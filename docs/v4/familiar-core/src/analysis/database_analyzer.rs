//! Database Schema Analysis - Validates SeaORM compliance in services
//!
//! Checks for:
//! - Direct sqlx::query() usage in services (should use TigerDataStore)
//! - Missing SeaORM entity usage (services should use entities from familiar-core)
//! - Direct database pool access (should use TigerDataStore methods)
//! - Legacy row mapping patterns
//! - Services bypassing familiar-core database layer
//!
//! This analyzer focuses on **familiar-services** (familiar-api, familiar-worker),
//! not familiar-core/primitives/contracts.

use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use super::issue_kinds::{Fix, Issue, IssueKind, Severity};

/// Services that should use SeaORM via TigerDataStore
const TARGET_SERVICES: &[&str] = &["familiar-api", "familiar-worker"];

/// Legacy row mapper types that should be migrated
const LEGACY_ROW_TYPES: &[&str] = &[
    "DbEntityRow",
    "DbFieldExcitationRow",
    "DbQuantumStateRow",
    "DbContentRow",
    "DbMediaRefRow",
];

/// Database-specific analyzer for SeaORM compliance in services
pub struct DatabaseAnalyzer {
    root: PathBuf,
}

impl DatabaseAnalyzer {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Run database-specific analysis on services
    pub fn analyze(&self, files: &[PathBuf]) -> Vec<Issue> {
        let issues = Mutex::new(Vec::new());

        // Only analyze files in target services
        let service_files: Vec<_> = files
            .iter()
            .filter(|f| self.is_service_file(f))
            .collect();

        // Phase 1: Check services for direct sqlx usage
        self.check_direct_sqlx_usage(&service_files, &issues);

        // Phase 2: Check for legacy row mapping patterns
        self.check_row_mapping_patterns(&service_files, &issues);

        // Phase 3: Check for direct pool access
        self.check_pool_access(&service_files, &issues);

        // Phase 4: Check for entity definitions in services
        self.check_entity_definitions(&service_files, &issues);

        // Phase 5: Check TigerDataStore usage
        self.check_store_usage(&service_files, &issues);

        issues.into_inner().unwrap()
    }

    /// Check if file is in a target service
    fn is_service_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        TARGET_SERVICES.iter().any(|service| {
            path_str.contains(&format!("services/{}", service))
        })
    }

    /// Extract service name from file path
    fn service_from_path(&self, path: &PathBuf) -> Option<String> {
        let path_str = path.to_string_lossy();
        for service in TARGET_SERVICES {
            if path_str.contains(&format!("services/{}", service)) {
                return Some(service.to_string());
            }
        }
        None
    }

    /// Check for direct sqlx::query() usage in services
    /// Services should use TigerDataStore methods instead
    fn check_direct_sqlx_usage(&self, files: &[&PathBuf], issues: &Mutex<Vec<Issue>>) {
        let sqlx_query_re = Regex::new(r"sqlx::query[^_]").unwrap();
        let sqlx_query_as_re = Regex::new(r"sqlx::query_as").unwrap();
        let sqlx_query_scalar_re = Regex::new(r"sqlx::query_scalar").unwrap();

        files.par_iter().for_each(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let service = self.service_from_path(path).unwrap_or_default();

                for (line_num, line) in content.lines().enumerate() {
                    // Check for sqlx::query
                    if sqlx_query_re.is_match(line) 
                        || sqlx_query_as_re.is_match(line) 
                        || sqlx_query_scalar_re.is_match(line) 
                    {
                        let mut locked = issues.lock().unwrap();
                        locked.push(Issue {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            kind: IssueKind::DirectSqlxUsage {
                                service: service.clone(),
                                query_type: self.extract_query_type(line),
                            },
                            severity: Severity::Error,
                            message: format!(
                                "Direct sqlx usage in service '{}' - use TigerDataStore methods instead",
                                service
                            ),
                            fix: Some(Fix {
                                description: "Use TigerDataStore.method() instead of direct sqlx query".to_string(),
                                replacement: None,
                            }),
                        });
                    }
                }
            }
        });
    }

    /// Extract query type from line
    fn extract_query_type(&self, line: &str) -> String {
        if line.contains("sqlx::query_as") {
            "query_as".to_string()
        } else if line.contains("sqlx::query_scalar") {
            "query_scalar".to_string()
        } else {
            "query".to_string()
        }
    }

    /// Check for legacy row mapping patterns (DbEntityRow, etc.)
    fn check_row_mapping_patterns(&self, files: &[&PathBuf], issues: &Mutex<Vec<Issue>>) {
        files.par_iter().for_each(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let service = self.service_from_path(path).unwrap_or_default();

                for (line_num, line) in content.lines().enumerate() {
                    for row_type in LEGACY_ROW_TYPES {
                        if line.contains(row_type) {
                            let mut locked = issues.lock().unwrap();
                            locked.push(Issue {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                kind: IssueKind::LegacyRowMapping {
                                    service: service.clone(),
                                    row_type: row_type.to_string(),
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "Legacy row mapper '{}' used in service '{}' - use SeaORM entities instead",
                                    row_type, service
                                ),
                            fix: Some(Fix {
                                description: format!(
                                    "Replace '{}' with SeaORM entity from familiar_core::entities::db",
                                    row_type
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

    /// Check for direct pool access (store.pool(), .pool())
    fn check_pool_access(&self, files: &[&PathBuf], issues: &Mutex<Vec<Issue>>) {
        let pool_access_re = Regex::new(r"\.pool\(\)").unwrap();
        let pool_type_re = Regex::new(r"Pool<Postgres>|PgPool").unwrap();

        files.par_iter().for_each(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let service = self.service_from_path(path).unwrap_or_default();

                for (line_num, line) in content.lines().enumerate() {
                    if pool_access_re.is_match(line) || pool_type_re.is_match(line) {
                        let mut locked = issues.lock().unwrap();
                        locked.push(Issue {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            kind: IssueKind::DirectPoolAccess {
                                service: service.clone(),
                            },
                            severity: Severity::Error,
                            message: format!(
                                "Direct pool access in service '{}' - use TigerDataStore methods instead",
                                service
                            ),
                            fix: Some(Fix {
                                description: "Remove .pool() access and use TigerDataStore methods".to_string(),
                                replacement: None,
                            }),
                        });
                    }
                }
            }
        });
    }

    /// Check for entity definitions in services (should be in familiar-core)
    fn check_entity_definitions(&self, files: &[&PathBuf], issues: &Mutex<Vec<Issue>>) {
        // Patterns that suggest entity definitions
        let derive_entity_re = Regex::new(r"DeriveEntityModel|sea_orm\(table_name").unwrap();
        let struct_entity_re = Regex::new(r"pub struct \w+Entity").unwrap();

        files.par_iter().for_each(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let service = self.service_from_path(path).unwrap_or_default();

                for (line_num, line) in content.lines().enumerate() {
                    if derive_entity_re.is_match(line) || struct_entity_re.is_match(line) {
                        let entity_name = self.extract_entity_name(&content, line_num);
                        let mut locked = issues.lock().unwrap();
                        locked.push(Issue {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            kind: IssueKind::EntityInService {
                                service: service.clone(),
                                entity_name: entity_name.clone(),
                            },
                            severity: Severity::Error,
                            message: format!(
                                "SeaORM entity '{}' defined in service '{}' - move to familiar-core/src/entities/db/",
                                entity_name, service
                            ),
                            fix: Some(Fix {
                                description: "Move entity to familiar_core::entities::db and import from there".to_string(),
                                replacement: None,
                            }),
                        });
                    }
                }
            }
        });
    }

    /// Extract entity name from context
    fn extract_entity_name(&self, content: &str, line_num: usize) -> String {
        let struct_re = Regex::new(r"pub struct (\w+)").unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        // Look for struct definition within 5 lines
        for i in line_num.saturating_sub(5)..=(line_num + 5).min(lines.len().saturating_sub(1)) {
            if let Some(caps) = struct_re.captures(lines.get(i).unwrap_or(&"")) {
                return caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            }
        }
        "Unknown".to_string()
    }

    /// Check that services properly use TigerDataStore
    fn check_store_usage(&self, files: &[&PathBuf], issues: &Mutex<Vec<Issue>>) {
        // Check for services that import sqlx but not TigerDataStore
        let sqlx_use_re = Regex::new(r"use sqlx").unwrap();
        let store_use_re = Regex::new(r"TigerDataStore|familiar_core::infrastructure").unwrap();

        files.par_iter().for_each(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let service = self.service_from_path(path).unwrap_or_default();
                let has_sqlx_import = sqlx_use_re.is_match(&content);
                let has_store_import = store_use_re.is_match(&content);

                if has_sqlx_import && !has_store_import {
                    // Find the sqlx import line
                    for (line_num, line) in content.lines().enumerate() {
                        if sqlx_use_re.is_match(line) {
                            let mut locked = issues.lock().unwrap();
                            locked.push(Issue {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                kind: IssueKind::BypassingStore {
                                    service: service.clone(),
                                    pattern: "sqlx import without TigerDataStore".to_string(),
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "Service '{}' imports sqlx directly but not TigerDataStore - consider using familiar-core's database layer",
                                    service
                                ),
                            fix: Some(Fix {
                                description: "Import TigerDataStore from familiar_core::infrastructure".to_string(),
                                replacement: None,
                            }),
                            });
                            break; // Only report once per file
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_service_file() {
        let analyzer = DatabaseAnalyzer::new(PathBuf::from("/test"));
        
        assert!(analyzer.is_service_file(&PathBuf::from("/path/to/services/familiar-api/src/main.rs")));
        assert!(analyzer.is_service_file(&PathBuf::from("/path/to/services/familiar-worker/src/mod.rs")));
        assert!(!analyzer.is_service_file(&PathBuf::from("/path/to/familiar-core/src/lib.rs")));
    }

    #[test]
    fn test_service_from_path() {
        let analyzer = DatabaseAnalyzer::new(PathBuf::from("/test"));
        
        assert_eq!(
            analyzer.service_from_path(&PathBuf::from("/services/familiar-api/src/main.rs")),
            Some("familiar-api".to_string())
        );
        assert_eq!(
            analyzer.service_from_path(&PathBuf::from("/services/familiar-worker/src/mod.rs")),
            Some("familiar-worker".to_string())
        );
    }

    #[test]
    fn test_extract_query_type() {
        let analyzer = DatabaseAnalyzer::new(PathBuf::from("/test"));
        
        assert_eq!(analyzer.extract_query_type("sqlx::query(\"SELECT\")"), "query");
        assert_eq!(analyzer.extract_query_type("sqlx::query_as::<_, User>"), "query_as");
        assert_eq!(analyzer.extract_query_type("sqlx::query_scalar"), "query_scalar");
    }
}

