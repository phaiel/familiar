//! Communication Pattern Analyzer
//!
//! Validates that all internal service communication uses Redpanda/Kafka.
//! Detects antipatterns like:
//! - HTTP clients in worker services
//! - Direct inter-service calls bypassing Kafka
//! - Cargo.toml dependencies that suggest HTTP usage in workers
//!
//! ## Architecture Rules
//!
//! ```text
//! External → familiar-api (HTTP OK) → DB, Redpanda
//!                                   ↓
//!                              Redpanda/Kafka
//!                                   ↓
//!            familiar-worker (NO HTTP) ← Consume/Produce
//! ```
//!
//! Only `familiar-api` may use HTTP clients for external API calls.
//! All other services must use Kafka for inter-service communication.

use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use toml::Value;

use super::issue_kinds::{Fix, Issue, IssueKind, Severity};

/// Services that are allowed to use HTTP clients
const ALLOWED_HTTP_SERVICES: &[&str] = &["familiar-api"];

/// Services that must NOT use HTTP clients (internal workers)
const WORKER_SERVICES: &[&str] = &["familiar-worker"];

/// Cargo dependencies that indicate HTTP client usage
const HTTP_DEPENDENCIES: &[&str] = &["reqwest", "hyper", "surf", "ureq", "isahc"];

/// Communication pattern analyzer for Redpanda compliance
pub struct CommunicationAnalyzer {
    root: PathBuf,
    /// Paths to service directories
    service_paths: Vec<PathBuf>,
}

impl CommunicationAnalyzer {
    pub fn new(root: PathBuf) -> Self {
        let services_dir = root.join("services");
        let service_paths = if services_dir.exists() {
            fs::read_dir(&services_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { root, service_paths }
    }

    /// Run communication pattern analysis
    pub fn analyze(&self, files: &[PathBuf]) -> Vec<Issue> {
        let issues = Mutex::new(Vec::new());

        // Phase 1: Check Cargo.toml dependencies for HTTP clients in workers
        self.check_cargo_dependencies(&issues);

        // Phase 2: Check source files for HTTP patterns in worker services
        self.check_source_files(files, &issues);

        // Phase 3: Validate service architecture (future enhancement)
        // self.validate_service_topology(&issues);

        issues.into_inner().unwrap()
    }

    /// Check Cargo.toml files for HTTP dependencies in worker services
    fn check_cargo_dependencies(&self, issues: &Mutex<Vec<Issue>>) {
        for service_path in &self.service_paths {
            let service_name = service_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Skip services that are allowed to use HTTP
            if ALLOWED_HTTP_SERVICES.contains(&service_name) {
                continue;
            }

            let cargo_path = service_path.join("Cargo.toml");
            if !cargo_path.exists() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&cargo_path) {
                if let Ok(toml) = content.parse::<Value>() {
                    self.check_dependencies_in_toml(
                        &toml,
                        &cargo_path,
                        service_name,
                        issues,
                    );
                }
            }
        }
    }

    /// Check a Cargo.toml for HTTP-related dependencies
    fn check_dependencies_in_toml(
        &self,
        toml: &Value,
        cargo_path: &PathBuf,
        service_name: &str,
        issues: &Mutex<Vec<Issue>>,
    ) {
        let check_deps = |deps: Option<&Value>| {
            if let Some(Value::Table(deps_table)) = deps {
                for (dep_name, _) in deps_table.iter() {
                    if HTTP_DEPENDENCIES.iter().any(|d| *d == dep_name) {
                        // Check if it's a worker service
                        if WORKER_SERVICES.contains(&service_name) {
                            issues.lock().unwrap().push(Issue {
                                file: cargo_path.clone(),
                                line: 1,
                                kind: IssueKind::DirectHttpInWorker {
                                    component: format!("{} dependency", dep_name),
                                    file: cargo_path.display().to_string(),
                                    suggested_alternative: "Remove HTTP dependency - use Kafka EnvelopeProducer/Consumer".to_string(),
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "Worker service '{}' has HTTP dependency '{}' - workers should use Kafka for communication",
                                    service_name, dep_name
                                ),
                                fix: Some(Fix {
                                    description: format!(
                                        "Remove '{}' from Cargo.toml and migrate to Kafka-based communication",
                                        dep_name
                                    ),
                                    replacement: None,
                                }),
                            });
                        }
                    }
                }
            }
        };

        // Check [dependencies]
        check_deps(toml.get("dependencies"));
        // Check [dev-dependencies] - might be acceptable for testing
        // check_deps(toml.get("dev-dependencies"));
    }

    /// Check source files for HTTP patterns
    fn check_source_files(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let http_patterns = [
            ("reqwest::Client", "reqwest HTTP client"),
            ("hyper::Client", "hyper HTTP client"),
            ("http_client.post", "HTTP POST call"),
            ("http_client.get", "HTTP GET call"),
            (".send().await", "async HTTP request"),
        ];

        files.par_iter().for_each(|path| {
            let path_str = path.display().to_string();

            // Determine which service this file belongs to
            let service_name = self.service_from_path(&path_str);

            // Skip if in allowed service
            if let Some(name) = &service_name {
                if ALLOWED_HTTP_SERVICES.contains(&name.as_str()) {
                    return;
                }
            }

            // Only check worker service files
            if !path_str.contains("familiar-worker") {
                return;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (pattern, description) in &http_patterns {
                    if content.contains(pattern) {
                        let line = content
                            .lines()
                            .enumerate()
                            .find(|(_, line)| line.contains(pattern))
                            .map(|(i, _)| i + 1)
                            .unwrap_or(1);

                        issues.lock().unwrap().push(Issue {
                            file: path.clone(),
                            line,
                            kind: IssueKind::DirectHttpInWorker {
                                component: description.to_string(),
                                file: path_str.clone(),
                                suggested_alternative: "EnvelopeProducer.send_command()".to_string(),
                            },
                            severity: Severity::Error,
                            message: format!(
                                "{} in worker service - use Kafka EnvelopeProducer instead",
                                description
                            ),
                            fix: Some(Fix {
                                description: "Replace HTTP call with Kafka message".to_string(),
                                replacement: None,
                            }),
                        });
                    }
                }
            }
        });
    }

    /// Extract service name from file path
    fn service_from_path(&self, path: &str) -> Option<String> {
        if path.contains("familiar-api") {
            Some("familiar-api".to_string())
        } else if path.contains("familiar-worker") {
            Some("familiar-worker".to_string())
        } else if path.contains("familiar-ui") {
            Some("familiar-ui".to_string())
        } else {
            None
        }
    }

    /// Get a summary of communication patterns found
    pub fn get_summary(&self, issues: &[Issue]) -> CommunicationSummary {
        let mut summary = CommunicationSummary::default();

        for issue in issues {
            match &issue.kind {
                IssueKind::DirectHttpInWorker { .. } => {
                    summary.http_in_workers += 1;
                }
                IssueKind::HttpClientInWorkerStruct { .. } => {
                    summary.http_client_fields += 1;
                }
                IssueKind::InterServiceBypassKafka { .. } => {
                    summary.kafka_bypasses += 1;
                }
                _ => {}
            }
        }

        summary.total_violations = summary.http_in_workers + summary.http_client_fields + summary.kafka_bypasses;
        summary.compliant = summary.total_violations == 0;

        summary
    }
}

/// Summary of communication pattern analysis
#[derive(Debug, Default)]
pub struct CommunicationSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// HTTP clients found in worker services
    pub http_in_workers: usize,
    /// HTTP client fields in worker structs
    pub http_client_fields: usize,
    /// Direct calls bypassing Kafka
    pub kafka_bypasses: usize,
    /// Whether the codebase is fully compliant
    pub compliant: bool,
}

impl std::fmt::Display for CommunicationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.compliant {
            writeln!(f, "✅ Communication patterns compliant - all internal traffic uses Redpanda/Kafka")
        } else {
            writeln!(f, "❌ Communication pattern violations found:")?;
            if self.http_in_workers > 0 {
                writeln!(f, "  - HTTP clients in workers: {}", self.http_in_workers)?;
            }
            if self.http_client_fields > 0 {
                writeln!(f, "  - HTTP client struct fields: {}", self.http_client_fields)?;
            }
            if self.kafka_bypasses > 0 {
                writeln!(f, "  - Kafka bypass calls: {}", self.kafka_bypasses)?;
            }
            writeln!(f, "  Total: {} violations", self.total_violations)
        }
    }
}

/// Migration guidance for fixing communication violations
pub struct MigrationGuide;

impl MigrationGuide {
    /// Generate migration steps for a given issue
    pub fn for_issue(issue: &Issue) -> Vec<String> {
        match &issue.kind {
            IssueKind::DirectHttpInWorker { component, .. } => {
                vec![
                    format!("1. Remove {} from worker code", component),
                    "2. Add EnvelopeProducer to your struct instead".to_string(),
                    "3. Emit commands via producer.send_command(&envelope)".to_string(),
                    "4. Create a Kafka consumer on the target service".to_string(),
                    "5. Process results via Kafka events".to_string(),
                ]
            }
            IssueKind::HttpClientInWorkerStruct { struct_name, .. } => {
                vec![
                    format!("1. Remove http_client field from {}", struct_name),
                    "2. Add envelope_producer: EnvelopeProducer field".to_string(),
                    "3. Update constructor to create EnvelopeProducer".to_string(),
                    "4. Replace HTTP calls with Kafka messages".to_string(),
                ]
            }
            IssueKind::InterServiceBypassKafka { target_service, method, .. } => {
                vec![
                    format!("1. Remove direct {} call to {}", method, target_service),
                    format!("2. Create {}Request envelope payload type", target_service),
                    "3. Emit request via EnvelopeProducer".to_string(),
                    format!("4. Set up Kafka consumer in {}", target_service),
                    "5. Return results via Kafka events".to_string(),
                ]
            }
            _ => vec!["See documentation for migration guidance".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_detection() {
        let analyzer = CommunicationAnalyzer::new(PathBuf::from("."));
        
        assert_eq!(
            analyzer.service_from_path("/path/to/services/familiar-api/src/main.rs"),
            Some("familiar-api".to_string())
        );
        assert_eq!(
            analyzer.service_from_path("/path/to/services/familiar-worker/src/orchestrator/mod.rs"),
            Some("familiar-worker".to_string())
        );
    }

    #[test]
    fn test_allowed_services() {
        assert!(ALLOWED_HTTP_SERVICES.contains(&"familiar-api"));
        assert!(!ALLOWED_HTTP_SERVICES.contains(&"familiar-worker"));
    }
}

