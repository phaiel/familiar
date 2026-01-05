//! Cross-File Analysis - Checks that require multi-file context
//!
//! These checks cannot be done with ast-grep because they require:
//! - Comparing across multiple files
//! - Pattern detection across types
//! - SQL parsing
//!
//! NOTE: Schema validation (unused schemas, generation sync, drift) is handled
//! by `schema-drift` in familiar-schemas, NOT this analyzer.
//! This analyzer checks that SERVICE CODE uses schemas correctly.

use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use super::issue_kinds::{
    AnalysisReport, DeriveCoverage, Fix, Issue, IssueKind, OrphanRecommendation, Severity, Stats,
};
use super::kafka_analyzer::{KafkaAnalyzer, check_worker_kafka_compliance};
use super::walker::FastWalker;
use crate::config::schema_lock;
use crate::schemas::graph::{OrphanInfo, SchemaGraph};

/// Cross-file analyzer for checks that require multi-file context
/// 
/// Focuses on checking SERVICE CODE patterns, not schema validation.
/// Schema validation is done by `schema-drift` in familiar-schemas.
pub struct CrossFileAnalyzer {
    root: PathBuf,
    known_entities: HashSet<&'static str>,
}

impl CrossFileAnalyzer {
    pub fn new(root: PathBuf) -> Self {
        let known_entities: HashSet<&'static str> = [
            "User",
            "Tenant",
            "TenantMember",
            "Channel",
            "Message",
            "AuthSession",
            "MagicLink",
            "Invitation",
            "JoinRequest",
            "AsyncTask",
            "OnboardingSession",
            "FamiliarEntity",
        ]
        .into_iter()
        .collect();

        Self {
            root,
            known_entities,
        }
    }

    pub fn analyze(&self) -> AnalysisReport {
        let start = Instant::now();
        let issues = Mutex::new(Vec::new());

        // Collect files using fast walker
        let walker = FastWalker::new(self.root.clone());
        let files: Vec<PathBuf> = walker.collect_files().into_iter().map(|f| f.path).collect();
        let files_count = files.len();

        // NOTE: Schema validation (unused schemas, generation sync, stale files)
        // is handled by `schema-drift` in familiar-schemas, NOT the analyzer.
        // The analyzer only checks that SERVICE CODE uses schemas correctly.

        // Phase 1: Shared field pattern detection
        self.detect_shared_field_patterns(&files, &issues);

        // Phase 2: ECS System detection (only in specific directories)
        self.detect_ecs_systems(&files, &issues);

        // Phase 3: Decomposition candidates (large structs with 8+ fields)
        self.detect_decomposition_candidates(&files, &issues);

        // Phase 4: Duplicate type name detection
        self.detect_duplicate_types(&files, &issues);

        // Phase 5: Kafka/Protobuf codegen compliance
        self.analyze_kafka_compliance(&files, &issues);

        let mut all_issues = issues.into_inner().unwrap();
        all_issues.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then_with(|| a.file.cmp(&b.file))
                .then_with(|| a.line.cmp(&b.line))
        });

        // Compute stats
        let entity_candidates = all_issues
            .iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestEntity { .. }))
            .count();
        let systems_detected = all_issues
            .iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestSystem { .. }))
            .count();
        let laws_detected = all_issues
            .iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestLaw { .. }))
            .count();
        let kafka_issues = all_issues
            .iter()
            .filter(|i| matches!(i.kind, 
                IssueKind::MissingProtobufSchema { .. } |
                IssueKind::MissingKafkaKey { .. } |
                IssueKind::MissingProtobufEnvelope { .. } |
                IssueKind::JsonSerializationInsteadOfProtobuf { .. } |
                IssueKind::CodegenPathViolation { .. }
            ))
            .count();
        let manual_kafka_impl = all_issues
            .iter()
            .filter(|i| matches!(i.kind, IssueKind::ManualKafkaImplementation { .. }))
            .count();

        AnalysisReport {
            stats: Stats {
                files_scanned: files_count,
                types_exported: 0, // Schema stats handled by familiar-schemas
                types_defined: 0,
                types_in_familiar_core: 0,
                types_in_services: 0,
                issues_found: all_issues.len(),
                duration_ms: start.elapsed().as_millis() as u64,
                derive_coverage: DeriveCoverage::default(),
                entity_candidates,
                systems_detected,
                laws_detected,
                decomposition_candidates: 0,
                shared_patterns: 0,
                trait_suggestions: 0,
                kafka_issues,
                manual_kafka_impl,
                communication_violations: 0,
                http_client_in_workers: 0,
                kafka_bypass_detected: 0,
                database_issues: 0,
                direct_sqlx_usage: 0,
                legacy_row_mapping: 0,
                orphan_schemas: 0,
                orphans_connect_graph: 0,
                orphans_delete: 0,
                orphans_deprecated: 0,
                missing_json_schemas: 0,
                isolated_schemas: 0,
            },
            issues: all_issues,
        }
    }

    // NOTE: Database table validation removed - schema validation is done by
    // `schema-drift` in familiar-schemas, not the analyzer.

    fn detect_shared_field_patterns(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let type_fields: Mutex<HashMap<String, (PathBuf, usize, Vec<String>)>> =
            Mutex::new(HashMap::new());
        let type_re = Regex::new(r"(?m)^(?:pub\s+)?struct\s+([A-Z][A-Za-z0-9_]*)").unwrap();

        for path in files {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            if !path.to_string_lossy().contains("/familiar-core/src/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for cap in type_re.captures_iter(&content) {
                    let name = cap.get(1).unwrap().as_str();
                    let match_start = cap.get(0).unwrap().start();
                    let line = content[..match_start].lines().count() + 1;
                    let fields = self.extract_fields(&content, match_start);

                    let field_names: Vec<String> = fields
                        .iter()
                        .filter_map(|f| f.split(':').next())
                        .map(|f| f.trim().trim_start_matches("pub ").to_string())
                        .filter(|f| !f.is_empty())
                        .collect();

                    if !field_names.is_empty() {
                        type_fields
                            .lock()
                            .unwrap()
                            .insert(name.to_string(), (path.clone(), line, field_names));
                    }
                }
            }
        }

        let known_patterns: Vec<(&str, Vec<&str>)> = vec![
            ("Timestamps", vec!["created_at", "updated_at"]),
            ("RequestContext", vec!["ip_address", "user_agent"]),
            ("EntityMeta", vec!["id", "tenant_id"]),
            ("Auditable", vec!["reviewed_by", "reviewed_at"]),
        ];

        let tf = type_fields.into_inner().unwrap();

        for (component_name, pattern_fields) in &known_patterns {
            let mut types_using_pattern: Vec<(String, PathBuf, usize)> = Vec::new();

            for (type_name, (path, line, fields)) in &tf {
                let has_all_fields = pattern_fields
                    .iter()
                    .all(|pf| fields.iter().any(|f| f.contains(pf)));
                let uses_component = fields.iter().any(|f| f.contains(*component_name));

                if has_all_fields && !uses_component {
                    types_using_pattern.push((type_name.clone(), path.clone(), *line));
                }
            }

            if types_using_pattern.len() >= 2 {
                let type_names: Vec<String> = types_using_pattern
                    .iter()
                    .map(|(n, _, _)| n.clone())
                    .collect();

                if let Some((_, path, line)) = types_using_pattern.first() {
                    issues.lock().unwrap().push(Issue {
                        file: path.clone(),
                        line: *line,
                        kind: IssueKind::SharedFieldPattern {
                            field_names: pattern_fields.iter().map(|s| s.to_string()).collect(),
                            types_using: type_names.clone(),
                            suggested_component: component_name.to_string(),
                        },
                        severity: Severity::Info,
                        message: format!(
                            "Types {} all have {} fields - extract to {} component",
                            type_names.join(", "),
                            pattern_fields.join("/"),
                            component_name
                        ),
                        fix: Some(Fix {
                            description: format!(
                                "Create {} component and use #[serde(flatten)] in these types",
                                component_name
                            ),
                            replacement: None,
                        }),
                    });
                }
            }
        }
    }

    fn detect_ecs_systems(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let world_fn_re =
            Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*(?:&\s*(?:mut\s+)?World|world\s*:\s*&)")
                .unwrap();
        let system_name_re =
            Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(\w+_system)\s*\(").unwrap();

        for path in files {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let path_str = path.to_string_lossy();
            let is_system_path = path_str.contains("/simulation/")
                || path_str.contains("/systems/")
                || path_str.contains("/physics");

            if !is_system_path {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                // Detect World functions
                for cap in world_fn_re.captures_iter(&content) {
                    let fn_name = cap.get(1).unwrap().as_str();
                    let fn_start = cap.get(0).unwrap().start();
                    let line = content[..fn_start].lines().count() + 1;

                    issues.lock().unwrap().push(Issue {
                        file: path.clone(),
                        line,
                        kind: IssueKind::SuggestSystem {
                            name: fn_name.to_string(),
                            components: vec![],
                            location: path_str.to_string(),
                            category: None,
                        },
                        severity: Severity::Info,
                        message: format!("ECS System '{}' [hecs] - takes &World", fn_name),
                        fix: None,
                    });
                }

                // Detect *_system functions
                for cap in system_name_re.captures_iter(&content) {
                    let fn_name = cap.get(1).unwrap().as_str();
                    let fn_start = cap.get(0).unwrap().start();
                    let line = content[..fn_start].lines().count() + 1;

                    // Skip if already detected as world function
                    if world_fn_re.is_match(&content[fn_start..fn_start.saturating_add(200).min(content.len())]) {
                        continue;
                    }

                    issues.lock().unwrap().push(Issue {
                        file: path.clone(),
                        line,
                        kind: IssueKind::SuggestSystem {
                            name: fn_name.to_string(),
                            components: vec![],
                            location: path_str.to_string(),
                            category: None,
                        },
                        severity: Severity::Info,
                        message: format!("ECS System '{}' [named convention]", fn_name),
                        fix: None,
                    });
                }
            }
        }
    }

    fn detect_decomposition_candidates(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let type_re = Regex::new(r"(?m)^(?:pub\s+)?struct\s+([A-Z][A-Za-z0-9_]*)").unwrap();
        const MIN_FIELDS_FOR_DECOMPOSITION: usize = 8;

        for path in files {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            // Only check familiar-core types
            if !path.to_string_lossy().contains("/familiar-core/src/types/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for cap in type_re.captures_iter(&content) {
                    let name = cap.get(1).unwrap().as_str();
                    let match_start = cap.get(0).unwrap().start();
                    let line = content[..match_start].lines().count() + 1;
                    let fields = self.extract_fields(&content, match_start);

                    if fields.len() >= MIN_FIELDS_FOR_DECOMPOSITION {
                        // Suggest components based on field patterns
                        let mut suggested = Vec::new();
                        let field_str = fields.join(" ");
                        
                        if field_str.contains("created_at") && field_str.contains("updated_at") {
                            suggested.push("Timestamps".to_string());
                        }
                        if field_str.contains("ip_address") || field_str.contains("user_agent") {
                            suggested.push("RequestContext".to_string());
                        }
                        if field_str.contains("tenant_id") && fields.iter().any(|f| f.contains("id:")) {
                            suggested.push("EntityMeta".to_string());
                        }

                        issues.lock().unwrap().push(Issue {
                            file: path.clone(),
                            line,
                            kind: IssueKind::SuggestDecompose {
                                name: name.to_string(),
                                field_count: fields.len(),
                                suggested_components: suggested,
                            },
                            severity: Severity::Info,
                            message: format!(
                                "Large struct '{}' has {} fields - consider decomposing",
                                name, fields.len()
                            ),
                            fix: Some(Fix {
                                description: "Extract common field groups into components with #[serde(flatten)]".to_string(),
                                replacement: None,
                            }),
                        });
                    }
                }
            }
        }
    }

    fn detect_duplicate_types(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        // Collect all struct/enum names and their locations
        let type_locations: Mutex<HashMap<String, Vec<String>>> = Mutex::new(HashMap::new());
        let type_re = Regex::new(r"(?m)^(?:pub\s+)?(?:struct|enum)\s+([A-Z][A-Za-z0-9_]*)").unwrap();

        for path in files {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            // Only check familiar-core to avoid flagging test files etc.
            if !path.to_string_lossy().contains("/familiar-core/src/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for cap in type_re.captures_iter(&content) {
                    let name = cap.get(1).unwrap().as_str().to_string();
                    let location = path.to_string_lossy().to_string();
                    
                    type_locations
                        .lock()
                        .unwrap()
                        .entry(name)
                        .or_insert_with(Vec::new)
                        .push(location);
                }
            }
        }

        // Report types defined in multiple files
        // Exclude common false positives:
        // - 'Args' in different binaries
        // - 'Model' and 'Relation' are SeaORM conventions (each entity has its own)
        // - Types in clearly different domains (ui/ vs entities/api/)
        let locations = type_locations.into_inner().unwrap();
        // SeaORM entities each define their own Model, Relation, ActiveModel, Entity structs
        // This is a framework convention, not code duplication
        let excluded_types = ["Args", "Model", "Relation", "ActiveModel", "Entity", "Column", "PrimaryKey"];
        
        // Helper to check if locations are in different, non-overlapping domains
        fn are_separate_domains(locs: &[String]) -> bool {
            // If one is in ui/block_kit and another in entities/api, they're different domains
            let has_ui = locs.iter().any(|l| l.contains("/ui/") || l.contains("/block_kit"));
            let has_api = locs.iter().any(|l| l.contains("/entities/api/") || l.contains("/multimodal"));
            if has_ui && has_api && locs.len() == 2 {
                return true;
            }
            
            // If one is in entities/db/ and another in types/, they're intentional DB vs domain separation
            let has_db_entity = locs.iter().any(|l| l.contains("/entities/db/"));
            let has_domain_type = locs.iter().any(|l| l.contains("/types/") && !l.contains("/entities/"));
            if has_db_entity && has_domain_type && locs.len() == 2 {
                return true;
            }
            
            false
        }
        
        for (name, locs) in &locations {
            // Skip known false positives
            if excluded_types.contains(&name.as_str()) {
                continue;
            }
            
            // Skip if all locations are in /bin/ (different CLI tools)
            let all_in_binaries = locs.iter().all(|l| l.contains("/bin/"));
            if all_in_binaries {
                continue;
            }
            
            // Skip if in clearly separate domains (different modules for different purposes)
            if are_separate_domains(locs) {
                continue;
            }
            
            if locs.len() > 1 {
                // Get line number for first occurrence
                issues.lock().unwrap().push(Issue {
                    file: PathBuf::from(&locs[0]),
                    line: 0,
                    kind: IssueKind::DuplicateTypeName {
                        name: name.clone(),
                        locations: locs.clone(),
                    },
                    severity: Severity::Error,
                    message: format!(
                        "Type '{}' is defined in {} files: {}",
                        name,
                        locs.len(),
                        locs.iter()
                            .map(|l| l.split('/').last().unwrap_or(l))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    fix: Some(Fix {
                        description: format!("Rename or consolidate duplicate '{}' definitions", name),
                        replacement: None,
                    }),
                });
            }
        }
    }

    fn extract_fields(&self, content: &str, struct_start: usize) -> Vec<String> {
        let mut fields = Vec::new();

        if let Some(brace_start) = content[struct_start..].find('{') {
            let body_start = struct_start + brace_start + 1;

            if let Some(brace_end) = content[body_start..].find('}') {
                let body = &content[body_start..body_start + brace_end];

                for line in body.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains(':')
                        && !trimmed.starts_with("//")
                        && !trimmed.starts_with('#')
                    {
                        fields.push(trimmed.to_string());
                    }
                }
            }
        }

        fields
    }

    /// Extract struct code from file at the given line
    fn extract_struct_code(&self, path: &PathBuf, line: usize) -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        let lines: Vec<&str> = content.lines().collect();
        
        if line == 0 || line > lines.len() {
            return None;
        }
        
        // Find the struct start and end
        let start_idx = line.saturating_sub(1);
        let mut brace_count = 0;
        let mut started = false;
        let mut end_idx = start_idx;
        
        for (i, line_content) in lines.iter().enumerate().skip(start_idx) {
            for ch in line_content.chars() {
                if ch == '{' {
                    brace_count += 1;
                    started = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
            end_idx = i;
            if started && brace_count == 0 {
                break;
            }
        }
        
        // Include a few lines before for attributes
        let attr_start = start_idx.saturating_sub(5);
        Some(lines[attr_start..=end_idx].join("\n"))
    }

    // TODO: LLM escalation functionality removed - types were never defined
    // Restore when LlmEscalation and LlmEscalationReason types are added to issue_kinds.rs

    /// Phase 9: Kafka/Protobuf codegen compliance analysis
    fn analyze_kafka_compliance(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        // Run the Kafka analyzer
        let kafka_analyzer = KafkaAnalyzer::new(self.root.clone());
        let kafka_issues = kafka_analyzer.analyze(files);
        
        // Add Kafka-specific issues
        issues.lock().unwrap().extend(kafka_issues);
        
        // Also check familiar-worker compliance
        let worker_issues = check_worker_kafka_compliance(&self.root, files);
        issues.lock().unwrap().extend(worker_issues);
    }

    /// Analyze orphan schemas from the schema graph.
    /// 
    /// This method:
    /// 1. Loads the schema graph from familiar-schemas
    /// 2. Finds all orphan schemas (no incoming edges)
    /// 3. Checks if each orphan type is used in Rust/TS code
    /// 4. Categorizes orphans with appropriate recommendations
    pub fn analyze_orphan_schemas(
        &self,
        filter: Option<&str>,
    ) -> (Vec<Issue>, OrphanStats) {
        let mut issues = Vec::new();
        let mut stats = OrphanStats::default();

        // Find schemas directory using schema.lock configuration
        let Some(ref schema_dir) = schema_lock::find_schema_dir(&self.root) else {
            eprintln!("Warning: Could not find familiar-schemas directory");
            return (issues, stats);
        };

        // Build the schema graph
        let graph = match SchemaGraph::from_directory_with_depth(&schema_dir, 10) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Warning: Could not build schema graph: {}", e);
                return (issues, stats);
            }
        };

        // Get orphan schemas grouped by category
        let orphans_by_cat = graph.orphans_by_category();
        
        // Collect all Rust files for code usage checking
        let walker = FastWalker::new(self.root.clone());
        let all_files: Vec<PathBuf> = walker.collect_files().into_iter().map(|f| f.path).collect();
        let rust_files: Vec<_> = all_files.iter()
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
            .collect();

        // Categories that are expected to be orphans (infrastructure roots)
        let expected_root_categories = ["ecs", "queues", "nodes", "systems", "resources"];
        
        // Categories that indicate deprecated/legacy code
        let deprecated_categories = ["windmill"];

        for (category, orphans) in orphans_by_cat {
            // Apply filter if specified
            if let Some(f) = filter {
                if !category.contains(f) {
                    continue;
                }
            }

            stats.total += orphans.len();
            
            let is_expected = expected_root_categories.iter().any(|c| category == *c);
            let is_deprecated = deprecated_categories.iter().any(|c| category == *c);

            for orphan in orphans {
                // Track truly isolated vs consumer-only
                if orphan.has_outgoing {
                    stats.consumer_only += 1;
                } else {
                    stats.truly_isolated += 1;
                }
                
                // Skip types/primitives as they're frequently orphans by design
                if category == "types" || category == "primitives" {
                    stats.skipped_primitives += 1;
                    continue;
                }
                
                // Consumer-only schemas (have outgoing refs) are expected leaf nodes
                // Only report truly isolated schemas as needing attention
                if orphan.has_outgoing {
                    // This is a consumer schema - it references others but isn't referenced
                    // This is normal for tools, contracts, etc.
                    continue;
                }

                // Derive Rust type name from schema path
                let type_name = extract_type_name(&orphan.schema_id);
                
                // Check if type is used in code
                let usage_locations = self.find_type_usage(&type_name, &rust_files);
                let used_in_code = !usage_locations.is_empty();

                // Determine recommendation
                let recommendation = if is_expected {
                    stats.expected_roots += 1;
                    OrphanRecommendation::ExpectedRoot
                } else if is_deprecated {
                    stats.deprecated += 1;
                    OrphanRecommendation::MarkDeprecated
                } else if used_in_code {
                    stats.connect_graph += 1;
                    OrphanRecommendation::ConnectGraph
                } else {
                    stats.delete += 1;
                    OrphanRecommendation::Delete
                };

                // Determine severity based on recommendation
                let severity = match recommendation {
                    OrphanRecommendation::Delete => Severity::Warning,
                    OrphanRecommendation::ConnectGraph => Severity::Info,
                    OrphanRecommendation::MarkDeprecated => Severity::Info,
                    OrphanRecommendation::ExpectedRoot => Severity::Info,
                };

                let message = match &recommendation {
                    OrphanRecommendation::Delete => {
                        format!("Truly isolated schema '{}' - not used in code, safe to delete", type_name)
                    }
                    OrphanRecommendation::ConnectGraph => {
                        format!(
                            "Truly isolated schema '{}' - used in {} locations, needs graph connection via $ref",
                            type_name, usage_locations.len()
                        )
                    }
                    OrphanRecommendation::MarkDeprecated => {
                        format!("Truly isolated schema '{}' - from deprecated {} system", type_name, category)
                    }
                    OrphanRecommendation::ExpectedRoot => {
                        format!("Truly isolated schema '{}' - expected root (infrastructure)", type_name)
                    }
                };

                issues.push(Issue {
                    file: PathBuf::from(&orphan.file_path),
                    line: 0,
                    kind: IssueKind::OrphanSchema {
                        schema_name: type_name.clone(),
                        schema_path: orphan.schema_id.clone(),
                        category: category.clone(),
                        used_in_code,
                        usage_locations: usage_locations.clone(),
                        recommendation: recommendation.clone(),
                    },
                    severity,
                    message,
                    fix: match recommendation {
                        OrphanRecommendation::ConnectGraph => Some(Fix {
                            description: format!(
                                "Add x-familiar-* extension to connect '{}' to the schema graph",
                                type_name
                            ),
                            replacement: None,
                        }),
                        OrphanRecommendation::MarkDeprecated => Some(Fix {
                            description: "Add x-familiar-deprecated: true to schema".to_string(),
                            replacement: None,
                        }),
                        OrphanRecommendation::Delete => Some(Fix {
                            description: format!("Delete schema file: {}", orphan.schema_id),
                            replacement: None,
                        }),
                        OrphanRecommendation::ExpectedRoot => None,
                    },
                });
            }
        }

        // Sort by severity, then category, then name
        issues.sort_by(|a, b| {
            a.severity.cmp(&b.severity)
                .then_with(|| {
                    if let (IssueKind::OrphanSchema { category: cat_a, .. }, 
                            IssueKind::OrphanSchema { category: cat_b, .. }) = (&a.kind, &b.kind) {
                        cat_a.cmp(cat_b)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .then_with(|| a.file.cmp(&b.file))
        });

        (issues, stats)
    }

    /// Find usages of a type name in Rust files.
    /// 
    /// This uses simple regex matching. For more sophisticated analysis,
    /// the ast-grep integration can be used separately.
    fn find_type_usage(&self, type_name: &str, rust_files: &[&PathBuf]) -> Vec<String> {
        let mut locations = Vec::new();
        
        // Build regex patterns for type usage
        let patterns = [
            // Struct/enum field: foo: TypeName
            format!(r":\s*{}\b", regex::escape(type_name)),
            // Generic parameter: Vec<TypeName>
            format!(r"<{}\b", regex::escape(type_name)),
            // Function parameter or return: fn foo(x: TypeName) -> TypeName
            format!(r"\b{}\b", regex::escape(type_name)),
            // Import: use x::TypeName
            format!(r"use\s+.*::{}\b", regex::escape(type_name)),
        ];
        
        let combined_pattern = patterns.join("|");
        let re = match Regex::new(&combined_pattern) {
            Ok(r) => r,
            Err(_) => return locations,
        };

        for path in rust_files {
            // Skip generated files
            let path_str = path.to_string_lossy();
            if path_str.contains("/generated/") || path_str.contains("/target/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        // Make sure it's not in a comment
                        let trimmed = line.trim();
                        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                            let relative_path = path.strip_prefix(&self.root)
                                .unwrap_or(path)
                                .to_string_lossy();
                            locations.push(format!("{}:{}", relative_path, line_num + 1));
                            // Limit to 10 locations per type
                            if locations.len() >= 10 {
                                return locations;
                            }
                        }
                    }
                }
            }
        }

        locations
    }
    
    /// Detect Rust types without corresponding JSON schemas.
    /// Scans familiar-core/src/types/ for struct/enum definitions and checks
    /// if they have a matching schema in familiar-schemas.
    pub fn detect_missing_json_schemas(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Find the types directory
        let types_dir = self.root.join("familiar-core/src/types");
        if !types_dir.exists() {
            return issues;
        }
        
        // Find schema directory using schema.lock configuration
        let schema_dir = match schema_lock::find_schema_dir(&self.root) {
            Some(p) => p,
            None => return issues,
        };
        
        // Collect all schema names
        let mut schema_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for entry in walkdir::WalkDir::new(schema_dir)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace(".schema", "");
                schema_names.insert(name);
            }
        }
        
        // Scan Rust files for type definitions
        let type_re = Regex::new(r"pub\s+(?:struct|enum)\s+(\w+)").unwrap();
        
        for entry in walkdir::WalkDir::new(&types_dir)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if path.file_name().map(|n| n == "mod.rs").unwrap_or(false) {
                    continue;
                }
                
                if let Ok(content) = fs::read_to_string(path) {
                    for cap in type_re.captures_iter(&content) {
                        let type_name = cap.get(1).unwrap().as_str().to_string();
                        
                        // Skip internal types
                        if type_name.ends_with("Builder") || 
                           type_name.ends_with("Error") ||
                           type_name.starts_with("_") {
                            continue;
                        }
                        
                        if !schema_names.contains(&type_name) {
                            // Suggest a schema path based on file location
                            let rel_path = path.strip_prefix(&types_dir)
                                .unwrap_or(path)
                                .to_string_lossy();
                            let category = rel_path.split('/').next().unwrap_or("types");
                            let suggested_path = format!("{}/{}.schema.json", category, type_name);
                            
                            issues.push(Issue {
                                file: path.to_path_buf(),
                                line: 0,
                                kind: IssueKind::MissingJsonSchema {
                                    type_name: type_name.clone(),
                                    source_file: path.display().to_string(),
                                    suggested_schema_path: suggested_path,
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "Rust type '{}' has no corresponding JSON schema",
                                    type_name
                                ),
                                fix: Some(Fix {
                                    description: "Create schema from Rust type using schema_export".to_string(),
                                    replacement: None,
                                }),
                            });
                        }
                    }
                }
            }
        }
        
        issues
    }
    
    /// Detect isolated types/primitives that should be referenced by other schemas.
    pub fn detect_isolated_schemas(&self, filter: Option<&str>) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Find schema directory using schema.lock configuration
        let Some(schema_dir) = schema_lock::find_schema_dir(&self.root) else {
            return issues;
        };
        
        // Build schema graph
        let graph = match crate::schemas::SchemaGraph::from_directory_with_depth(&schema_dir, 10) {
            Ok(g) => g,
            Err(_) => return issues,
        };
        
        // Get truly isolated schemas
        let isolated = graph.truly_isolated_schemas();
        
        // Collect Rust files for usage check
        let walker = crate::analysis::FastWalker::new(self.root.clone());
        let all_files: Vec<PathBuf> = walker.collect_files().into_iter().map(|f| f.path).collect();
        let rust_files: Vec<_> = all_files.iter()
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
            .collect();
        
        for orphan in isolated {
            // Only report types and primitives
            if orphan.category != "types" && orphan.category != "primitives" {
                continue;
            }
            
            // Apply filter if specified
            if let Some(f) = filter {
                if !orphan.category.contains(f) {
                    continue;
                }
            }
            
            let type_name = extract_type_name(&orphan.schema_id);
            let usage_locations = self.find_type_usage(&type_name, &rust_files);
            let used_in_code = !usage_locations.is_empty();
            
            // Find expected refs based on name patterns
            let expected_refs = suggest_expected_refs(&type_name, &orphan.category);
            
            issues.push(Issue {
                file: PathBuf::from(&orphan.file_path),
                line: 0,
                kind: IssueKind::IsolatedSchema {
                    schema_name: type_name.clone(),
                    schema_path: orphan.schema_id.clone(),
                    category: orphan.category.clone(),
                    used_in_code,
                    expected_refs: expected_refs.clone(),
                },
                severity: if used_in_code { Severity::Warning } else { Severity::Info },
                message: if used_in_code {
                    format!(
                        "Isolated {} '{}' - used in code but not referenced by any schema. Should be $ref'd from: {}",
                        orphan.category, type_name,
                        if expected_refs.is_empty() { "unknown".to_string() } else { expected_refs.join(", ") }
                    )
                } else {
                    format!(
                        "Isolated {} '{}' - not used in code or schemas, may be unused",
                        orphan.category, type_name
                    )
                },
                fix: Some(Fix {
                    description: format!(
                        "Add $ref to this {} from schemas that use it",
                        orphan.category
                    ),
                    replacement: None,
                }),
            });
        }
        
        issues
    }
}

/// Suggest which schemas should reference this type based on naming patterns.
fn suggest_expected_refs(type_name: &str, category: &str) -> Vec<String> {
    let mut expected = Vec::new();
    
    // ID primitives should be used by entities with that name
    if type_name.ends_with("Id") {
        let entity_name = type_name.trim_end_matches("Id");
        expected.push(format!("entities/{}.schema.json", entity_name));
    }
    
    // Status types should be used by entities
    if type_name.ends_with("Status") {
        let entity_name = type_name.trim_end_matches("Status");
        expected.push(format!("entities/{}.schema.json", entity_name));
    }
    
    // Type enums should be used by entities
    if type_name.ends_with("Type") {
        let entity_name = type_name.trim_end_matches("Type");
        expected.push(format!("entities/{}.schema.json", entity_name));
    }
    
    // Input/Output types should be used by tools/systems
    if type_name.ends_with("Input") || type_name.ends_with("Output") {
        let base = type_name.trim_end_matches("Input").trim_end_matches("Output");
        expected.push(format!("tools/{}.schema.json", base));
        expected.push(format!("systems/{}.system.json", base));
    }
    
    // Request/Response types should be used by auth/api
    if type_name.ends_with("Request") || type_name.ends_with("Response") {
        expected.push(format!("auth/{}.schema.json", type_name));
        expected.push(format!("api/{}.schema.json", type_name));
    }
    
    expected
}

/// Statistics for orphan schema analysis.
#[derive(Debug, Default)]
pub struct OrphanStats {
    pub total: usize,
    /// Truly isolated - no incoming AND no outgoing edges (real orphans)
    pub truly_isolated: usize,
    /// Consumer-only - have outgoing edges but no incoming (leaf schemas)
    pub consumer_only: usize,
    pub expected_roots: usize,
    pub deprecated: usize,
    pub connect_graph: usize,
    pub delete: usize,
    pub skipped_primitives: usize,
}

/// Extract a Rust-friendly type name from a schema path.
/// 
/// e.g., "auth/CreateUserInput.schema.json" -> "CreateUserInput"
fn extract_type_name(schema_id: &str) -> String {
    // Get the filename part
    let filename = schema_id
        .rsplit('/')
        .next()
        .unwrap_or(schema_id);
    
    // Remove common suffixes
    let name = filename
        .trim_end_matches(".json")
        .trim_end_matches(".schema")
        .trim_end_matches(".component")
        .trim_end_matches(".node")
        .trim_end_matches(".system")
        .trim_end_matches(".resource")
        .trim_end_matches(".queue")
        .trim_end_matches(".meta");
    
    name.to_string()
}

