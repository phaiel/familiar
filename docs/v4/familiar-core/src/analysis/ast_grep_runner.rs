//! ast-grep Integration for Schema Analysis
//!
//! Runs ast-grep rules and converts results to our Issue format.
//! This module replaces most of the manual Rust parsing with declarative YAML rules.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use super::issue_kinds::{AnalysisReport, Issue, IssueKind, Severity, Stats, Fix};
use super::schema_cache::SchemaCache;

/// Range information from ast-grep
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AstGrepRange {
    start: Position,
    #[allow(dead_code)]
    end: Position,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Position {
    line: usize,
    #[allow(dead_code)]
    column: usize,
}

/// A single match from ast-grep JSON output
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AstGrepMatch {
    file: String,
    range: AstGrepRange,
    rule_id: String,
    severity: String,
    message: String,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    meta_variables: Option<MetaVariables>,
}

#[derive(Debug, Deserialize, Default, schemars::JsonSchema)]
struct MetaVariables {
    #[serde(default)]
    single: HashMap<String, MetaVar>,
    #[serde(default)]
    multi: HashMap<String, Vec<MetaVar>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MetaVar {
    text: String,
    #[allow(dead_code)]
    range: AstGrepRange,
}

/// Runs ast-grep analysis on the codebase
pub struct AstGrepRunner {
    root: PathBuf,
    config_path: PathBuf,
    /// Shared cache of schema names from familiar-schemas
    schema_cache: SchemaCache,
}

impl AstGrepRunner {
    pub fn new(root: PathBuf) -> Self {
        let root = root.canonicalize().unwrap_or(root);
        let config_path = ["familiar-core/sgconfig.yml", "sgconfig.yml"]
            .iter()
            .map(|p| root.join(p))
            .find(|p| p.exists())
            .unwrap_or_else(|| root.join("familiar-core/sgconfig.yml"));
        
        // Load schema cache (shared across analyzers)
        let schema_cache = SchemaCache::new(&root);
        
        Self { root, config_path, schema_cache }
    }
    
    /// Create with a pre-loaded schema cache (for sharing across analyzers)
    pub fn with_cache(root: PathBuf, schema_cache: SchemaCache) -> Self {
        let root = root.canonicalize().unwrap_or(root);
        let config_path = ["familiar-core/sgconfig.yml", "sgconfig.yml"]
            .iter()
            .map(|p| root.join(p))
            .find(|p| p.exists())
            .unwrap_or_else(|| root.join("familiar-core/sgconfig.yml"));
        
        Self { root, config_path, schema_cache }
    }
    
    /// Get reference to the schema cache
    pub fn schema_cache(&self) -> &SchemaCache {
        &self.schema_cache
    }

    /// Run ast-grep and return an AnalysisReport
    pub fn run(&self) -> Result<AnalysisReport, AstGrepError> {
        let start = Instant::now();

        // Check if config exists
        if !self.config_path.exists() {
            return Err(AstGrepError::CommandFailed(
                format!("Config file not found: {}", self.config_path.display())
            ));
        }

        // Run ast-grep with JSON output
        // Use absolute paths and run from workspace root
        let output = Command::new("sg")
            .args([
                "scan", 
                "-c", 
                self.config_path.to_str().unwrap(), 
                "--json"
            ])
            .current_dir(&self.root)
            .output()
            .map_err(|e| AstGrepError::CommandFailed(e.to_string()))?;

        // Check for errors in stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() && !output.status.success() && output.status.code() != Some(1) {
            // Exit code 1 means matches found, which is expected
            return Err(AstGrepError::CommandFailed(stderr.to_string()));
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let matches: Vec<AstGrepMatch> = if stdout.trim().is_empty() || stdout.trim() == "[]" {
            Vec::new()
        } else {
            serde_json::from_str(&stdout)
                .map_err(|e| AstGrepError::ParseError(format!("Failed to parse ast-grep output: {} - stdout was: {}", e, &stdout[..stdout.len().min(500)])))?
        };

        // Convert to our Issue format
        let issues = self.convert_matches(matches);
        
        // Compute stats
        let stats = self.compute_stats(&issues, start.elapsed().as_millis() as u64);

        Ok(AnalysisReport { issues, stats })
    }

    fn convert_matches(&self, matches: Vec<AstGrepMatch>) -> Vec<Issue> {
        matches
            .into_iter()
            .filter_map(|m| self.convert_match(m))
            .collect()
    }

    fn convert_match(&self, m: AstGrepMatch) -> Option<Issue> {
        let severity = match m.severity.to_lowercase().as_str() {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => Severity::Info,
        };

        let kind = self.map_rule_to_kind(&m.rule_id, &m.message, m.meta_variables.as_ref())?;

        let fix = m.note.map(|note| Fix {
            description: note,
            replacement: None,
        });

        Some(Issue {
            file: PathBuf::from(&m.file),
            line: m.range.start.line,
            kind,
            severity,
            message: m.message,
            fix,
        })
    }

    fn map_rule_to_kind(&self, rule_id: &str, message: &str, meta: Option<&MetaVariables>) -> Option<IssueKind> {
        match rule_id {
            "raw-uuid-field" => Some(IssueKind::RawPrimitive {
                raw: "Uuid".to_string(),
                suggested: self.extract_suggested_primitive(message),
            }),
            
            "missing-ts-derive" => Some(IssueKind::InconsistentDerives {
                name: self.extract_type_name(meta),
                has: vec!["Serialize".to_string()],
                missing: vec!["TS".to_string()],
            }),
            
            "missing-openapi-derive" => Some(IssueKind::MissingOpenApiDerive {
                name: self.extract_type_name(meta),
                has_serialize: true,
            }),
            
            "centralize-type" => {
                let name = meta
                    .and_then(|m| m.single.get("NAME"))
                    .map(|v| v.text.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                Some(IssueKind::SuggestCentralize {
                    name,
                    category: self.infer_category_from_message(message),
                })
            }
            
            "local-pydantic-class" => {
                let name = meta
                    .and_then(|m| m.single.get("NAME"))
                    .map(|v| v.text.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                // Check if this type exists in familiar-schemas
                let schema_exists = self.schema_cache.exists(&name);
                let schema_path = self.schema_cache.find_path(&name);
                
                Some(IssueKind::LocalTypeNotInSchema {
                    type_name: name,
                    language: "python".to_string(),
                    schema_exists,
                    schema_path,
                })
            }
            
            "duplicate-ts-interface" => {
                let name = meta
                    .and_then(|m| m.single.get("NAME"))
                    .map(|v| v.text.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                Some(IssueKind::DuplicateType {
                    name,
                    generated_path: "familiar-core/generated/typescript/".to_string(),
                })
            }
            
            // NEW: Core entity in services (error)
            "core-entity-in-services" => {
                let name = self.extract_type_name(meta);
                Some(IssueKind::MissingSchema {
                    name: format!("{} (should be in familiar-core)", name),
                })
            }
            
            // NEW: DB type outside core (warning)
            "db-type-outside-core" => {
                let name = self.extract_type_name(meta);
                Some(IssueKind::SuggestCentralize {
                    name,
                    category: "database_entity".to_string(),
                })
            }
            
            "suggest-timestamps" => Some(IssueKind::SuggestTimestamps {
                name: self.extract_type_name(meta),
                has_created: true,
                has_updated: true, // Assume if created_at exists, updated_at likely does too
            }),
            
            "suggest-entity-meta" => Some(IssueKind::SuggestEntityMeta {
                name: self.extract_type_name(meta),
                id_field: "id".to_string(),
                has_tenant_id: true,
                has_timestamps: true,
            }),
            
            "suggest-request-context" => Some(IssueKind::SuggestSchema {
                name: self.extract_type_name(meta),
                reason: "Has ip_address/user_agent - use RequestContext component".to_string(),
            }),
            
            "suggest-has-physics" => Some(IssueKind::SuggestHasTrait {
                type_name: self.extract_type_name(meta),
                trait_name: "HasPhysics".to_string(),
                fields_covered: vec!["valence".to_string(), "arousal".to_string(), "significance".to_string()],
            }),
            
            "suggest-expirable" => Some(IssueKind::SuggestHasTrait {
                type_name: self.extract_type_name(meta),
                trait_name: "Expirable".to_string(),
                fields_covered: vec!["expires_at".to_string()],
            }),
            
            "large-struct-candidate" | "large-struct" => Some(IssueKind::SuggestDecompose {
                name: self.extract_type_name(meta),
                field_count: 0, // Unknown - ast-grep doesn't expose field counts directly
                suggested_components: vec![],
            }),
            
            "familiar-core-import" => Some(IssueKind::BrokenImport {
                import_path: meta
                    .and_then(|m| m.single.get("PATH"))
                    .map(|v| format!("@familiar-core/{}", v.text))
                    .unwrap_or_else(|| "@familiar-core/unknown".to_string()),
                resolved_to: "verify type exists".to_string(),
            }),
            
            // === NEW RULES ===
            
            // direct-env-access: Using std::env::var outside config modules
            "direct-env-access" => Some(IssueKind::SuggestSchema {
                name: "direct-env-access".to_string(),
                reason: message.to_string(),
            }),
            
            // suggest-entity: Structs with id + timestamps
            "suggest-entity" => Some(IssueKind::SuggestEntity {
                name: self.extract_type_name(meta),
                reason: "Has id + timestamps - could be SeaORM entity".to_string(),
                table_name: self.extract_type_name(meta).to_lowercase() + "s",
            }),
            
            // missing-entity-derive: SeaORM entities missing Serialize
            "missing-entity-derive" => Some(IssueKind::MissingEntityDerive {
                name: self.extract_type_name(meta),
                missing: vec!["Serialize".to_string(), "Deserialize".to_string()],
            }),
            
            // suggest-api-result: Response types with success/error pattern
            "suggest-api-result" => Some(IssueKind::SuggestApiResult {
                name: self.extract_type_name(meta),
                has_success: true,
                has_error: true,
            }),
            
            // inconsistent-derives: TS but not ToSchema
            "inconsistent-derives" => Some(IssueKind::InconsistentDerives {
                name: self.extract_type_name(meta),
                has: vec!["TS".to_string(), "Serialize".to_string()],
                missing: vec!["ToSchema".to_string()],
            }),
            
            // duplicate-core-type: Core types duplicated in services
            "duplicate-core-type" => Some(IssueKind::DuplicateType {
                name: self.extract_type_name(meta),
                generated_path: "familiar_core::types".to_string(),
            }),
            
            // suggest-primitive: String fields that could use primitives
            "suggest-primitive" => Some(IssueKind::SuggestPrimitive {
                field: self.extract_field_name(meta),
                suggested: self.infer_primitive_from_field(meta),
            }),
            
            // === ECS RULES ===
            
            // ecs-system-world: Functions taking &World or &mut World
            "ecs-system-world" => Some(IssueKind::SuggestSystem {
                name: self.extract_fn_name(meta),
                components: vec![],
                location: String::new(), // Will be filled from file path
                category: Some("hecs".to_string()),
            }),
            
            // ecs-system-named: Functions named *_system
            "ecs-system-named" => Some(IssueKind::SuggestSystem {
                name: self.extract_fn_name(meta),
                components: vec![],
                location: String::new(),
                category: Some("convention".to_string()),
            }),
            
            // law-candidate: Trait-bounded single-item operators
            "law-candidate" => Some(IssueKind::SuggestLaw {
                name: self.extract_fn_name(meta),
                trait_bounds: vec![],
                target_type: None,
                location: String::new(),
                category: None,
            }),
            
            // === COMPOSITION RULES ===
            
            // inline-timestamps: Struct defines timestamps inline
            "inline-timestamps" => Some(IssueKind::InlineTimestamps {
                name: self.extract_type_name(meta),
            }),
            
            // missing-entity-meta: Entity should use EntityMeta
            "missing-entity-meta" => Some(IssueKind::MissingEntityMeta {
                name: self.extract_type_name(meta),
            }),
            
            // === COMMUNICATION PATTERN RULES ===
            
            // direct-http-in-worker: HTTP client usage in worker services
            "direct-http-in-worker" => Some(IssueKind::DirectHttpInWorker {
                component: self.extract_http_component(message),
                file: String::new(), // Will be filled from file path
                suggested_alternative: "EnvelopeProducer.send_command()".to_string(),
            }),
            
            // http-client-struct-field: HTTP client stored as struct field
            "http-client-struct-field" => Some(IssueKind::HttpClientInWorkerStruct {
                struct_name: self.extract_type_name(meta),
                file: String::new(),
            }),
            
            // windmill-state-usage: Direct Windmill access should use Kafka
            "windmill-state-usage" | "state-windmill-access" => Some(IssueKind::InterServiceBypassKafka {
                caller_service: "familiar-api".to_string(),
                target_service: "windmill".to_string(),
                method: "state.windmill".to_string(),
                file: String::new(),
            }),
            
            // http-windmill-post: Direct HTTP to Windmill
            "http-windmill-post" => Some(IssueKind::InterServiceBypassKafka {
                caller_service: self.infer_service_from_file(message),
                target_service: "windmill".to_string(),
                method: "http_client.post".to_string(),
                file: String::new(),
            }),
            
            // windmill-client-new: WindmillClient instantiation (banned)
            "windmill-client-new" => Some(IssueKind::InterServiceBypassKafka {
                caller_service: "unknown".to_string(),
                target_service: "windmill".to_string(),
                method: "WindmillClient::new".to_string(),
                file: String::new(),
            }),
            
            // deprecated-windmill-field: Windmill field in struct (banned)
            "deprecated-windmill-field" => Some(IssueKind::HttpClientInWorkerStruct {
                struct_name: "AppState".to_string(),
                file: String::new(),
            }),
            
            _ => {
                // Unknown rule - skip silently to avoid noise
                // If you need to handle a new rule, add it explicitly above
                None
            }
        }
    }

    fn extract_type_name(&self, meta: Option<&MetaVariables>) -> String {
        meta.and_then(|m| m.single.get("NAME"))
            .map(|v| v.text.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn extract_fn_name(&self, meta: Option<&MetaVariables>) -> String {
        // Try NAME first (for consistent patterns), then FN for function-specific captures
        meta.and_then(|m| m.single.get("NAME").or_else(|| m.single.get("FN")))
            .map(|v| v.text.clone())
            .unwrap_or_else(|| "unknown_fn".to_string())
    }

    fn extract_suggested_primitive(&self, message: &str) -> String {
        // Try to extract the field name from the message to suggest a primitive
        if message.contains("tenant_id") {
            "TenantId".to_string()
        } else if message.contains("user_id") || message.contains("owner_id") {
            "UserId".to_string()
        } else if message.contains("channel_id") {
            "ChannelId".to_string()
        } else if message.contains("message_id") {
            "MessageId".to_string()
        } else if message.contains("session_id") {
            "SessionId".to_string()
        } else {
            "SemanticPrimitive".to_string()
        }
    }

    fn extract_field_name(&self, meta: Option<&MetaVariables>) -> String {
        meta.and_then(|m| m.single.get("FIELD"))
            .map(|v| v.text.clone())
            .unwrap_or_else(|| "unknown_field".to_string())
    }

    fn infer_primitive_from_field(&self, meta: Option<&MetaVariables>) -> String {
        let field = self.extract_field_name(meta);
        let field_lower = field.to_lowercase();
        
        if field_lower.contains("email") {
            "Email".to_string()
        } else if field_lower.ends_with("_url") || field_lower == "url" {
            "Url".to_string()
        } else if field_lower.contains("password") || field_lower.ends_with("_hash") {
            "PasswordHash".to_string()
        } else if field_lower.ends_with("_key") {
            "ApiKey".to_string()
        } else if field_lower.ends_with("_code") {
            "InviteCode".to_string()
        } else if field_lower.ends_with("_token") {
            "SessionToken".to_string()
        } else {
            // Generate PascalCase suggestion from field name
            field.split('_')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str()
                    }
                })
                .collect()
        }
    }

    fn infer_category_from_message(&self, message: &str) -> String {
        if message.contains("Request") || message.contains("Response") {
            "api_contract".to_string()
        } else if message.contains("Config") || message.contains("Options") {
            "configuration".to_string()
        } else if message.contains("Input") || message.contains("Output") {
            "flow_interface".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    fn extract_http_component(&self, message: &str) -> String {
        if message.contains("reqwest::Client::new") {
            "reqwest::Client::new()".to_string()
        } else if message.contains("reqwest::Client::builder") {
            "reqwest::Client::builder()".to_string()
        } else if message.contains("http_client.post") {
            "http_client.post()".to_string()
        } else if message.contains("http_client.get") {
            "http_client.get()".to_string()
        } else {
            "HTTP client".to_string()
        }
    }
    
    fn infer_service_from_file(&self, _message: &str) -> String {
        // This will be enhanced when we have file path context
        "unknown".to_string()
    }

    fn compute_stats(&self, issues: &[Issue], duration_ms: u64) -> Stats {
        let mut stats = Stats::default();
        stats.duration_ms = duration_ms;
        stats.issues_found = issues.len();

        // Count by issue kind
        for issue in issues {
            match &issue.kind {
                IssueKind::SuggestEntity { .. } => stats.entity_candidates += 1,
                IssueKind::SuggestSystem { .. } => stats.systems_detected += 1,
                IssueKind::SuggestLaw { .. } => stats.laws_detected += 1,
                IssueKind::SuggestDecompose { .. } => stats.decomposition_candidates += 1,
                IssueKind::SharedFieldPattern { .. } => stats.shared_patterns += 1,
                IssueKind::SuggestHasTrait { .. } => stats.trait_suggestions += 1,
                // Communication pattern violations
                IssueKind::DirectHttpInWorker { .. } => {
                    stats.communication_violations += 1;
                    stats.http_client_in_workers += 1;
                }
                IssueKind::HttpClientInWorkerStruct { .. } => {
                    stats.communication_violations += 1;
                    stats.http_client_in_workers += 1;
                }
                IssueKind::InterServiceBypassKafka { .. } => {
                    stats.communication_violations += 1;
                    stats.kafka_bypass_detected += 1;
                }
                _ => {}
            }
        }

        // Count types in familiar-core vs services
        for issue in issues {
            let path = issue.file.to_string_lossy();
            if path.contains("familiar-core/src") {
                stats.types_in_familiar_core += 1;
            } else if path.contains("services/") {
                stats.types_in_services += 1;
            }
        }

        stats
    }
}

/// Error type for ast-grep operations
#[derive(Debug)]
pub enum AstGrepError {
    CommandFailed(String),
    ParseError(String),
}

impl std::fmt::Display for AstGrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstGrepError::CommandFailed(e) => write!(f, "ast-grep command failed: {}", e),
            AstGrepError::ParseError(e) => write!(f, "Failed to parse ast-grep output: {}", e),
        }
    }
}

impl std::error::Error for AstGrepError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_mapping() {
        // This would test the severity mapping logic
    }

    #[test]
    fn test_rule_mapping() {
        // This would test the rule_id to IssueKind mapping
    }
}

