//! Auto-Fix Capabilities for Schema Analyzer
//!
//! This module provides safe and semi-safe auto-fix capabilities for detected issues.
//!
//! ## Safety Levels
//!
//! - **Safe** (`--fix`): Additive changes only, no breaking changes
//!   - Add missing derives (ToSchema)
//!   - Delete stale generated files
//!   - Run cargo test to regenerate
//!
//! - **Unsafe** (`--fix-unsafe`): May cause breaking changes, requires confirmation
//!   - Replace Uuid with semantic primitive
//!   - Add #[serde(flatten)] for composition
//!   - Rename duplicate types

use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::issue_kinds::{Issue, IssueKind, OrphanRecommendation};
use crate::config::schema_lock;

/// Result of applying a fix
#[derive(Debug, Clone)]
pub enum FixResult {
    Applied {
        file: PathBuf,
        description: String,
    },
    Skipped {
        file: PathBuf,
        reason: String,
    },
    Failed {
        file: PathBuf,
        error: String,
    },
}

/// Summary of all fixes applied
#[derive(Debug, Default)]
pub struct FixSummary {
    pub safe_applied: usize,
    pub safe_skipped: usize,
    pub unsafe_applied: usize,
    pub unsafe_skipped: usize,
    pub failed: usize,
    pub details: Vec<FixResult>,
}

impl FixSummary {
    pub fn total_applied(&self) -> usize {
        self.safe_applied + self.unsafe_applied
    }
}

/// Auto-fixer for schema analysis issues
pub struct AutoFixer {
    root: PathBuf,
    dry_run: bool,
    /// Known primitive types that exist in the codebase
    known_primitives: HashSet<String>,
}

impl AutoFixer {
    pub fn new(root: PathBuf, dry_run: bool) -> Self {
        // Load known primitives from familiar-core
        let mut known_primitives = HashSet::new();
        known_primitives.insert("UserId".to_string());
        known_primitives.insert("TenantId".to_string());
        known_primitives.insert("ChannelId".to_string());
        known_primitives.insert("MessageId".to_string());
        known_primitives.insert("SessionId".to_string());
        known_primitives.insert("Email".to_string());
        
        Self {
            root,
            dry_run,
            known_primitives,
        }
    }

    /// Apply safe fixes only (--fix)
    pub fn apply_safe_fixes(&self, issues: &[Issue]) -> FixSummary {
        let mut summary = FixSummary::default();
        
        for issue in issues {
            let result = match &issue.kind {
                // Add missing derives (ToSchema)
                IssueKind::InconsistentDerives { missing, .. } => {
                    self.add_derives(&issue.file, issue.line, missing)
                }
                IssueKind::MissingOpenApiDerive { .. } => {
                    self.add_derives(&issue.file, issue.line, &["ToSchema".to_string()])
                }
                
                // Delete stale generated files
                IssueKind::StaleGeneration { file_name } => {
                    self.delete_stale_file(file_name)
                }
                
                // Run cargo test to regenerate
                IssueKind::MissingGeneration { .. } => {
                    self.run_cargo_test()
                }
                
                // Fix orphan schemas (safe: adds metadata, doesn't delete)
                IssueKind::OrphanSchema { 
                    schema_path, 
                    category,
                    recommendation,
                    schema_name,
                    ..
                } => {
                    self.fix_orphan_schema(schema_path, category, recommendation, schema_name)
                }
                
                _ => None,
            };
            
            if let Some(r) = result {
                match &r {
                    FixResult::Applied { .. } => summary.safe_applied += 1,
                    FixResult::Skipped { .. } => summary.safe_skipped += 1,
                    FixResult::Failed { .. } => summary.failed += 1,
                }
                summary.details.push(r);
            }
        }
        
        summary
    }

    /// Apply unsafe fixes (--fix-unsafe) - requires confirmation
    pub fn apply_unsafe_fixes(&self, issues: &[Issue], confirmed: bool) -> FixSummary {
        let mut summary = FixSummary::default();
        
        if !confirmed && !self.dry_run {
            println!("\n⚠️  The following unsafe changes will be made:");
            for issue in issues {
                if self.is_unsafe_fix(&issue.kind) {
                    println!("  - {}:{} - {}", 
                        issue.file.display(), 
                        issue.line,
                        issue.message
                    );
                }
            }
            println!("\nRun with --confirm to apply these changes.\n");
            return summary;
        }
        
        for issue in issues {
            let result = match &issue.kind {
                // Replace raw Uuid with semantic primitive
                IssueKind::RawPrimitive { suggested, .. } 
                    if self.known_primitives.contains(suggested) => {
                    self.replace_type(&issue.file, issue.line, suggested)
                }
                
                IssueKind::SuggestSemanticPrimitive { suggested_primitive, .. }
                    if self.known_primitives.contains(suggested_primitive) => {
                    self.replace_type(&issue.file, issue.line, suggested_primitive)
                }
                
                // Add serde(flatten) for timestamps
                IssueKind::InlineTimestamps { name } => {
                    self.migrate_to_timestamps(&issue.file, name)
                }
                
                _ => None,
            };
            
            if let Some(r) = result {
                match &r {
                    FixResult::Applied { .. } => summary.unsafe_applied += 1,
                    FixResult::Skipped { .. } => summary.unsafe_skipped += 1,
                    FixResult::Failed { .. } => summary.failed += 1,
                }
                summary.details.push(r);
            }
        }
        
        summary
    }

    /// Check if an issue requires unsafe fix
    fn is_unsafe_fix(&self, kind: &IssueKind) -> bool {
        matches!(kind,
            IssueKind::RawPrimitive { .. } |
            IssueKind::SuggestSemanticPrimitive { .. } |
            IssueKind::InlineTimestamps { .. } |
            IssueKind::MissingEntityMeta { .. } |
            IssueKind::DuplicateTypeName { .. }
        )
    }

    /// Resolve a file path - if relative, join with root
    fn resolve_path(&self, file: &Path) -> PathBuf {
        if file.is_absolute() {
            file.to_path_buf()
        } else {
            self.root.join(file)
        }
    }

    /// Add missing derives to a struct/enum
    fn add_derives(&self, file: &Path, line: usize, derives: &[String]) -> Option<FixResult> {
        let file = self.resolve_path(file);
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => return Some(FixResult::Failed { 
                file: file.to_path_buf(), 
                error: e.to_string() 
            }),
        };

        // Find the derive line near the target line
        let lines: Vec<&str> = content.lines().collect();
        let search_start = line.saturating_sub(5);
        let search_end = (line + 2).min(lines.len());
        
        let derive_re = Regex::new(r"#\[derive\(([^)]+)\)\]").unwrap();
        
        for i in search_start..search_end {
            if let Some(caps) = derive_re.captures(lines[i]) {
                let existing = caps.get(1).unwrap().as_str();
                let existing_derives: HashSet<&str> = existing
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                
                // Filter out derives that already exist
                let new_derives: Vec<&String> = derives
                    .iter()
                    .filter(|d| !existing_derives.contains(d.as_str()))
                    .collect();
                
                if new_derives.is_empty() {
                    return Some(FixResult::Skipped {
                        file: file.clone(),
                        reason: "All derives already present".to_string(),
                    });
                }
                
                let all_derives = format!("{}, {}", existing, new_derives.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                let new_line = format!("#[derive({})]", all_derives);
                
                if self.dry_run {
                    return Some(FixResult::Applied {
                        file: file.clone(),
                        description: format!("[DRY RUN] Would add derives: {}", new_derives.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                    });
                }
                
                // Apply the fix
                let mut new_lines = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                new_lines[i] = new_line;
                
                if let Err(e) = fs::write(&file, new_lines.join("\n")) {
                    return Some(FixResult::Failed {
                        file: file.clone(),
                        error: e.to_string(),
                    });
                }
                
                return Some(FixResult::Applied {
                    file,
                    description: format!("Added derives: {}", new_derives.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                });
            }
        }
        
        Some(FixResult::Skipped {
            file,
            reason: "Could not find #[derive(...)] attribute".to_string(),
        })
    }

    /// Delete a stale generated file
    fn delete_stale_file(&self, file_name: &str) -> Option<FixResult> {
        // Handle running from v4/ or familiar-core/
        let ts_path = ["familiar-core/generated/typescript", "generated/typescript"]
            .iter()
            .map(|p| self.root.join(format!("{}/{}.ts", p, file_name)))
            .find(|p| p.exists())
            .unwrap_or_else(|| self.root.join(format!("familiar-core/generated/typescript/{}.ts", file_name)));
        
        if !ts_path.exists() {
            return Some(FixResult::Skipped {
                file: ts_path.clone(),
                reason: "File doesn't exist".to_string(),
            });
        }
        
        if self.dry_run {
            return Some(FixResult::Applied {
                file: ts_path,
                description: "[DRY RUN] Would delete stale file".to_string(),
            });
        }
        
        match fs::remove_file(&ts_path) {
            Ok(_) => Some(FixResult::Applied {
                file: ts_path,
                description: "Deleted stale generated file".to_string(),
            }),
            Err(e) => Some(FixResult::Failed {
                file: ts_path,
                error: e.to_string(),
            }),
        }
    }

    /// Run cargo test to regenerate TypeScript types
    fn run_cargo_test(&self) -> Option<FixResult> {
        // Find familiar-core directory (could be self.root or self.root/familiar-core)
        let cargo_dir = if self.root.join("Cargo.toml").exists() {
            self.root.clone()
        } else {
            self.root.join("familiar-core")
        };
        
        if self.dry_run {
            return Some(FixResult::Applied {
                file: cargo_dir.clone(),
                description: "[DRY RUN] Would run 'cargo test' to regenerate TypeScript".to_string(),
            });
        }
        
        let output = Command::new("cargo")
            .args(["test", "export_ts", "--", "--nocapture"])
            .current_dir(&cargo_dir)
            .output();
        
        match output {
            Ok(o) if o.status.success() => Some(FixResult::Applied {
                file: cargo_dir,
                description: "Ran cargo test to regenerate TypeScript types".to_string(),
            }),
            Ok(o) => Some(FixResult::Failed {
                file: cargo_dir,
                error: String::from_utf8_lossy(&o.stderr).to_string(),
            }),
            Err(e) => Some(FixResult::Failed {
                file: cargo_dir,
                error: e.to_string(),
            }),
        }
    }

    /// Replace a raw type (like Uuid) with a semantic primitive
    fn replace_type(&self, file: &Path, line: usize, new_type: &str) -> Option<FixResult> {
        let file = self.resolve_path(file);
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => return Some(FixResult::Failed {
                file: file.clone(),
                error: e.to_string(),
            }),
        };
        
        let lines: Vec<&str> = content.lines().collect();
        if line == 0 || line > lines.len() {
            return Some(FixResult::Skipped {
                file,
                reason: "Invalid line number".to_string(),
            });
        }
        
        let target_line = lines[line - 1];
        
        // Pattern: pub field_name: Uuid or pub field_name: Option<Uuid>
        let uuid_re = Regex::new(r":\s*(Option<)?Uuid(>)?").unwrap();
        
        if !uuid_re.is_match(target_line) {
            return Some(FixResult::Skipped {
                file,
                reason: "No Uuid found on target line".to_string(),
            });
        }
        
        let new_line = if target_line.contains("Option<Uuid>") {
            uuid_re.replace(target_line, format!(": Option<{}>", new_type).as_str())
        } else {
            uuid_re.replace(target_line, format!(": {}", new_type).as_str())
        };
        
        if self.dry_run {
            return Some(FixResult::Applied {
                file,
                description: format!("[DRY RUN] Would replace Uuid with {}", new_type),
            });
        }
        
        // Apply the fix
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines[line - 1] = new_line.to_string();
        
        // Ensure the import exists
        self.ensure_import(&file, new_type);
        
        if let Err(e) = fs::write(&file, new_lines.join("\n")) {
            return Some(FixResult::Failed {
                file,
                error: e.to_string(),
            });
        }
        
        Some(FixResult::Applied {
            file,
            description: format!("Replaced Uuid with {}", new_type),
        })
    }

    /// Add import for a primitive type if not already present
    fn ensure_import(&self, file: &Path, type_name: &str) {
        if let Ok(content) = fs::read_to_string(file) {
            // Check if import already exists
            if content.contains(&format!("use crate::primitives::{}", type_name)) 
                || content.contains(&format!("{}::", type_name))
                || content.contains(&format!("{} }}", type_name)) {
                return;
            }
            
            // Find the last use statement and add after it
            let import_line = format!("use crate::primitives::{};\n", type_name);
            let lines: Vec<&str> = content.lines().collect();
            
            let mut last_use_idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("use ") {
                    last_use_idx = i;
                }
            }
            
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            new_lines.insert(last_use_idx + 1, import_line);
            
            let _ = fs::write(file, new_lines.join("\n"));
        }
    }

    /// Migrate a struct to use Timestamps component
    fn migrate_to_timestamps(&self, file: &Path, struct_name: &str) -> Option<FixResult> {
        let file = self.resolve_path(file);
        if self.dry_run {
            return Some(FixResult::Applied {
                file,
                description: format!("[DRY RUN] Would migrate '{}' to use Timestamps component", struct_name),
            });
        }
        
        // This is complex - we need to:
        // 1. Remove created_at and updated_at fields
        // 2. Add #[serde(flatten)] pub timestamps: Timestamps
        // 3. Add import
        
        // For now, just skip with a helpful message
        Some(FixResult::Skipped {
            file,
            reason: format!(
                "Migration to Timestamps requires manual review. Edit '{}' to use:\n\
                 #[serde(flatten)]\n\
                 pub timestamps: Timestamps",
                struct_name
            ),
        })
    }

    /// Fix an orphan schema by adding x-familiar-* extensions.
    /// 
    /// This is a SAFE fix - it only adds metadata to schema files,
    /// it doesn't delete schemas or modify the schema structure.
    fn fix_orphan_schema(
        &self,
        schema_path: &str,
        category: &str,
        recommendation: &OrphanRecommendation,
        schema_name: &str,
    ) -> Option<FixResult> {
        // Find the schema file using schema.lock configuration
        let schema_dir = schema_lock::find_schema_dir(&self.root);
        let schema_file = schema_dir.map(|dir| dir.join(schema_path));
        let Some(schema_file) = schema_file.filter(|p| p.exists()) else {
            return Some(FixResult::Skipped {
                file: PathBuf::from(schema_path),
                reason: "Could not find schema file".to_string(),
            });
        };

        match recommendation {
            OrphanRecommendation::ConnectGraph => {
                self.add_graph_connection(&schema_file, category, schema_name)
            }
            OrphanRecommendation::MarkDeprecated => {
                self.add_deprecated_extension(&schema_file)
            }
            OrphanRecommendation::Delete => {
                // Deletion is NOT a safe fix - skip
                Some(FixResult::Skipped {
                    file: schema_file.clone(),
                    reason: "Deletion requires manual review (not a safe fix)".to_string(),
                })
            }
            OrphanRecommendation::ExpectedRoot => {
                // Expected roots don't need fixing
                Some(FixResult::Skipped {
                    file: schema_file.clone(),
                    reason: "Expected root node - no fix needed".to_string(),
                })
            }
        }
    }

    /// Add x-familiar-* extensions to connect a schema to the graph.
    fn add_graph_connection(
        &self,
        schema_file: &Path,
        category: &str,
        schema_name: &str,
    ) -> Option<FixResult> {
        let content = match fs::read_to_string(schema_file) {
            Ok(c) => c,
            Err(e) => return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: e.to_string(),
            }),
        };

        // Parse JSON
        let mut json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(e) => return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: format!("Invalid JSON: {}", e),
            }),
        };

        // Determine what extensions to add based on category
        let extensions = self.infer_extensions_for_category(category, schema_name);
        
        if extensions.is_empty() {
            return Some(FixResult::Skipped {
                file: schema_file.to_path_buf(),
                reason: format!("Cannot infer extensions for category '{}'", category),
            });
        }

        if self.dry_run {
            return Some(FixResult::Applied {
                file: schema_file.to_path_buf(),
                description: format!(
                    "[DRY RUN] Would add extensions: {}",
                    extensions.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ")
                ),
            });
        }

        // Add extensions
        if let Some(obj) = json.as_object_mut() {
            for (key, value) in &extensions {
                obj.insert(key.clone(), value.clone());
            }
        }

        // Write back with pretty formatting
        let new_content = serde_json::to_string_pretty(&json).unwrap_or(content);
        
        if let Err(e) = fs::write(schema_file, new_content) {
            return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: e.to_string(),
            });
        }

        Some(FixResult::Applied {
            file: schema_file.to_path_buf(),
            description: format!(
                "Added x-familiar-* extensions: {}",
                extensions.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ")
            ),
        })
    }

    /// Infer appropriate x-familiar-* extensions based on schema category.
    fn infer_extensions_for_category(
        &self,
        category: &str,
        schema_name: &str,
    ) -> Vec<(String, serde_json::Value)> {
        let mut extensions = Vec::new();
        
        match category {
            "tools" => {
                // Tool schemas should be connected via x-familiar-kind
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("tool".to_string()),
                ));
                
                // If it looks like an input/output schema
                if schema_name.ends_with("Input") || schema_name.ends_with("Request") {
                    extensions.push((
                        "x-familiar-role".to_string(),
                        serde_json::Value::String("input".to_string()),
                    ));
                } else if schema_name.ends_with("Output") || schema_name.ends_with("Response") {
                    extensions.push((
                        "x-familiar-role".to_string(),
                        serde_json::Value::String("output".to_string()),
                    ));
                }
            }
            "components" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("component".to_string()),
                ));
            }
            "auth" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("auth".to_string()),
                ));
            }
            "entities" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("entity".to_string()),
                ));
            }
            "database" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("database".to_string()),
                ));
                extensions.push((
                    "x-familiar-persistence".to_string(),
                    serde_json::Value::String("postgres".to_string()),
                ));
            }
            "conversation" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("conversation".to_string()),
                ));
            }
            "agentic" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("agentic".to_string()),
                ));
            }
            "contracts" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("contract".to_string()),
                ));
            }
            "config" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("config".to_string()),
                ));
            }
            "ui" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("ui".to_string()),
                ));
            }
            "api" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("api".to_string()),
                ));
            }
            "tenant" => {
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String("tenant".to_string()),
                ));
            }
            _ => {
                // Generic fallback - just add the kind
                extensions.push((
                    "x-familiar-kind".to_string(),
                    serde_json::Value::String(category.to_string()),
                ));
            }
        }
        
        extensions
    }

    /// Add x-familiar-deprecated extension to mark a schema as deprecated.
    fn add_deprecated_extension(&self, schema_file: &Path) -> Option<FixResult> {
        let content = match fs::read_to_string(schema_file) {
            Ok(c) => c,
            Err(e) => return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: e.to_string(),
            }),
        };

        // Parse JSON
        let mut json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(e) => return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: format!("Invalid JSON: {}", e),
            }),
        };

        if self.dry_run {
            return Some(FixResult::Applied {
                file: schema_file.to_path_buf(),
                description: "[DRY RUN] Would add x-familiar-deprecated: true".to_string(),
            });
        }

        // Add deprecated extension
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "x-familiar-deprecated".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        // Write back with pretty formatting
        let new_content = serde_json::to_string_pretty(&json).unwrap_or(content);
        
        if let Err(e) = fs::write(schema_file, new_content) {
            return Some(FixResult::Failed {
                file: schema_file.to_path_buf(),
                error: e.to_string(),
            });
        }

        Some(FixResult::Applied {
            file: schema_file.to_path_buf(),
            description: "Added x-familiar-deprecated: true".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_known_primitives() {
        let fixer = AutoFixer::new(PathBuf::from("."), true);
        assert!(fixer.known_primitives.contains("UserId"));
        assert!(fixer.known_primitives.contains("TenantId"));
    }
}

