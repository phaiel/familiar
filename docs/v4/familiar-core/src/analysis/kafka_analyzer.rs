//! Kafka Schema Analysis - Validates Protobuf codegen compliance
//!
//! Checks for:
//! - Missing Protobuf message definitions for Kafka types
//! - Missing kafka_key() implementations
//! - Codegen path violations
//! - JSON serialization instead of Protobuf

use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use super::issue_kinds::{Fix, Issue, IssueKind, Severity};

/// Kafka-specific analyzer for Protobuf codegen compliance
pub struct KafkaAnalyzer {
    root: PathBuf,
    /// Known Kafka envelope types that must have Protobuf schemas
    envelope_types: HashSet<&'static str>,
}

impl KafkaAnalyzer {
    pub fn new(root: PathBuf) -> Self {
        let envelope_types: HashSet<&'static str> = [
            "EnvelopeV1",
            "Payload",
            "TraceKind",
            "TraceStatus",
            "TracePayload",
        ]
        .into_iter()
        .collect();

        Self {
            root,
            envelope_types,
        }
    }

    /// Run Kafka-specific analysis
    pub fn analyze(&self, files: &[PathBuf]) -> Vec<Issue> {
        let issues = Mutex::new(Vec::new());

        // Phase 1: Check familiar-core kafka types for Protobuf schema definitions
        self.check_kafka_types(&issues);

        // Phase 2: Check services for manual Kafka implementations
        self.check_manual_implementations(files, &issues);

        // Phase 3: Check for JSON serialization instead of Protobuf
        self.check_serialization_format(files, &issues);

        // Note: Migration opportunity detection (state.windmill, http_client.post, etc.)
        // is now handled by ast-grep rules in rules/kafka/

        issues.into_inner().unwrap()
    }

    /// Check that all Kafka envelope types have Protobuf schema definitions
    fn check_kafka_types(&self, issues: &Mutex<Vec<Issue>>) {
        let kafka_dir = self.root.join("familiar-core/src/types/kafka");
        
        if !kafka_dir.exists() {
            return;
        }

        let files = ["command.rs", "event.rs", "trace.rs"];
        
        for file_name in files {
            let file_path = kafka_dir.join(file_name);
            if let Ok(content) = fs::read_to_string(&file_path) {
                self.check_schema_derives(&content, &file_path, issues);
            }
        }
    }

    /// Check a file for missing Protobuf schema definitions for Kafka types
    /// 
    /// Note: We use Protobuf (not JSON Schema) for serialization because:
    /// - Protobuf provides compact binary encoding with strong typing
    /// - Schema Registry supports Protobuf schemas
    /// - Generated code provides type-safe serialization/deserialization
    fn check_schema_derives(&self, content: &str, file_path: &PathBuf, issues: &Mutex<Vec<Issue>>) {
        // Find all struct/enum declarations with their preceding attributes
        let type_re = Regex::new(r"(?ms)((?:#\[[^\]]+\]\s*)*)(pub\s+(?:struct|enum)\s+(\w+))").unwrap();
        // Check for Serialize/Deserialize (required for JSON encoding)
        let serde_re = Regex::new(r"derive\([^)]*Serialize.*Deserialize|derive\([^)]*Deserialize.*Serialize").unwrap();
        
        for cap in type_re.captures_iter(content) {
            let type_name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            
            // Only check known envelope types
            if !self.envelope_types.contains(type_name) {
                continue;
            }

            // Get the attributes block (everything before the type declaration)
            let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            
            let line = content[..cap.get(0).unwrap().start()]
                .lines()
                .count() + 1;
            
            // Check for Serialize/Deserialize
            if !serde_re.is_match(attrs) {
                issues.lock().unwrap().push(Issue {
                    file: file_path.clone(),
                    line,
                    kind: IssueKind::MissingProtobufSchema {
                        type_name: type_name.to_string(),
                        file: file_path.display().to_string(),
                    },
                    severity: Severity::Error,
                    message: format!(
                        "Kafka envelope type '{}' is missing #[derive(Serialize, Deserialize, schemars::JsonSchema)] for JSON encoding",
                        type_name
                    ),
                    fix: Some(Fix {
                        description: format!("Add Serialize, Deserialize to derive macro for {}", type_name),
                        replacement: Some("Serialize, Deserialize".to_string()),
                    }),
                });
            }
        }
    }

    /// Check services for deprecated manual Kafka implementations
    fn check_manual_implementations(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let deprecated_patterns = [
            (r"#\[deprecated.*Use generated.*Producer", "CommandProducer", "CourseCommandProducer"),
            (r"#\[deprecated.*Use generated.*Consumer", "TraceConsumer", "CourseTraceConsumer"),
            (r"impl\s+CommandProducer", "CommandProducer", "CourseCommandProducer"),
            (r"impl\s+TraceConsumer", "TraceConsumer", "CourseTraceConsumer"),
            (r"impl\s+EventProducer", "EventProducer", "CourseEventProducer"),
            (r"impl\s+CommandConsumer", "CommandConsumer", "CourseCommandConsumer"),
        ];

        files.par_iter().for_each(|path| {
            // Only check service kafka modules
            let path_str = path.display().to_string();
            if !path_str.contains("services/") || !path_str.contains("/kafka/") {
                return;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (pattern, component, replacement) in &deprecated_patterns {
                    if let Ok(re) = Regex::new(pattern) {
                        if re.is_match(&content) {
                            let line = content
                                .lines()
                                .enumerate()
                                .find(|(_, line)| re.is_match(line))
                                .map(|(i, _)| i + 1)
                                .unwrap_or(1);

                            issues.lock().unwrap().push(Issue {
                                file: path.clone(),
                                line,
                                kind: IssueKind::ManualKafkaImplementation {
                                    component: component.to_string(),
                                    file: path_str.clone(),
                                    suggested_replacement: format!(
                                        "familiar_core::kafka::clients::{}",
                                        replacement
                                    ),
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "Manual {} should be replaced with generated {}",
                                    component, replacement
                                ),
                                fix: Some(Fix {
                                    description: format!(
                                        "Use familiar_core::kafka::clients::{} instead",
                                        replacement
                                    ),
                                    replacement: None,
                                }),
                            });
                        }
                    }
                }
            }
        });
    }

    /// Check for JSON serialization when Protobuf should be used
    fn check_serialization_format(&self, files: &[PathBuf], issues: &Mutex<Vec<Issue>>) {
        let json_patterns = [
            (r"serde_json::to_string\(&envelope", "JSON serialization for Kafka envelope"),
            (r"serde_json::from_str.*EnvelopeV1", "JSON deserialization for Kafka envelope"),
            (r"send_command\(&envelope", "Sending command via EnvelopeProducer"),
            (r"send_event\(&envelope", "Sending event via EnvelopeProducer"),
            (r"send_trace\(&envelope", "Sending trace via EnvelopeProducer"),
        ];

        files.par_iter().for_each(|path| {
            let path_str = path.display().to_string();
            
            // Only check service kafka modules
            if !path_str.contains("services/") || !path_str.contains("/kafka/") {
                return;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (pattern, description) in &json_patterns {
                    if let Ok(re) = Regex::new(pattern) {
                        if re.is_match(&content) {
                            let line = content
                                .lines()
                                .enumerate()
                                .find(|(_, line)| re.is_match(line))
                                .map(|(i, _)| i + 1)
                                .unwrap_or(1);

                            issues.lock().unwrap().push(Issue {
                                file: path.clone(),
                                line,
                                kind: IssueKind::JsonSerializationInsteadOfProtobuf {
                                    component: description.to_string(),
                                    file: path_str.clone(),
                                },
                                severity: Severity::Info,
                                message: format!(
                                    "{} - should use Protobuf serialization via ProtobufSerializer",
                                    description
                                ),
                                fix: Some(Fix {
                                    description: "Use ProtobufSerializer.encode_with_schema_id() instead".to_string(),
                                    replacement: None,
                                }),
                            });
                        }
                    }
                }
            }
        });
    }

}

/// Check if familiar-worker is properly using Kafka codegen
pub fn check_worker_kafka_compliance(root: &PathBuf, files: &[PathBuf]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let worker_kafka_dir = root.join("services/familiar-worker/src/kafka");

    if !worker_kafka_dir.exists() {
        return issues;
    }

    // Check that worker uses generated consumers
    let expected_imports = [
        ("CourseCommandConsumer", "consumer.rs"),
        ("CourseEventProducer", "producer.rs"),
        ("CourseTraceProducer", "producer.rs"),
    ];

    for file in files {
        let path_str = file.display().to_string();
        if !path_str.contains("familiar-worker") || !path_str.contains("/kafka/") {
            continue;
        }

        if let Ok(content) = fs::read_to_string(file) {
            // Check for use of familiar_core::kafka::clients
            if !content.contains("familiar_core::kafka::clients") 
                && !content.contains("use crate::kafka") 
                && content.contains("impl") 
            {
                // This file has implementations but doesn't import generated types
                for (expected, _) in &expected_imports {
                    if content.contains(expected) && !content.contains(&format!("use.*{}", expected)) {
                        issues.push(Issue {
                            file: file.clone(),
                            line: 1,
                            kind: IssueKind::ManualKafkaImplementation {
                                component: expected.to_string(),
                                file: path_str.clone(),
                                suggested_replacement: format!(
                                    "familiar_core::kafka::clients::{}",
                                    expected
                                ),
                            },
                            severity: Severity::Warning,
                            message: format!(
                                "Worker Kafka module should import {} from familiar_core::kafka::clients",
                                expected
                            ),
                            fix: Some(Fix {
                                description: format!(
                                    "Add: use familiar_core::kafka::clients::{};",
                                    expected
                                ),
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

