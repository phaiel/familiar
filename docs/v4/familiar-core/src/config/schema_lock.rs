//! Schema lock configuration utilities
//!
//! Reads configuration from schema.lock to locate the schema source.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Schema source configuration from schema.lock [source] section
#[derive(Debug, Deserialize, Default, Clone)]
pub struct SchemaSource {
    /// Local filesystem path (relative to familiar-core)
    pub path: Option<String>,
    /// GitHub repository URL
    pub github: Option<String>,
    /// Git branch or tag (defaults to "main")
    #[serde(default = "default_branch")]
    pub branch: Option<String>,
}

fn default_branch() -> Option<String> {
    Some("main".to_string())
}

/// Schema lock file structure (subset for runtime use)
#[derive(Debug, Deserialize)]
pub struct SchemaLock {
    /// Locked schema version
    pub version: String,
    /// Content hash for integrity verification
    #[serde(default)]
    pub hash: String,
    /// Schema source configuration
    #[serde(default)]
    pub source: SchemaSource,
}

impl SchemaLock {
    /// Load schema.lock from the workspace
    ///
    /// Searches common locations for schema.lock relative to the given root.
    pub fn load(workspace_root: &Path) -> Option<Self> {
        let lock_paths = [
            workspace_root.join("familiar-core/schema.lock"),
            workspace_root.join("../familiar-core/schema.lock"),
            workspace_root.join("schema.lock"),
        ];
        
        for path in &lock_paths {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(lock) = toml::from_str(&content) {
                    return Some(lock);
                }
            }
        }
        
        None
    }
    
    /// Resolve the schema directory path
    ///
    /// Returns the path to the json-schema directory for the locked version.
    /// For GitHub sources, expects the repo to be checked out at build time.
    pub fn resolve_schema_dir(&self, workspace_root: &Path) -> Option<PathBuf> {
        if let Some(github_url) = &self.source.github {
            // For GitHub sources, Cargo puts the dependency in target/
            // The exact path depends on how the build script is set up
            let target_dir = workspace_root.join("target");
            let dep_name = github_url
                .split('/')
                .last()
                .unwrap_or("familiar-schemas")
                .replace(".git", "");

            let schema_dir = target_dir
                .join("familiar-schemas")  // This is how Cargo names git deps
                .join("versions")
                .join(&self.version)
                .join("json-schema");

            if schema_dir.exists() {
                return Some(schema_dir);
            }
        } else if let Some(source_path) = &self.source.path {
            // Try relative paths from workspace root (legacy local path support)
            let base_paths = [
                workspace_root.join("familiar-core").join(source_path),
                workspace_root.join(source_path),
            ];

            for base in &base_paths {
                let schema_dir = base.join("versions").join(&self.version).join("json-schema");
                if schema_dir.exists() {
                    return Some(schema_dir);
                }
            }
        }

        None
    }
}

/// Find the schema directory using schema.lock configuration
///
/// This is the preferred way to locate schemas at runtime for development tools.
/// For the MCP and library code, use embedded schemas instead.
pub fn find_schema_dir(workspace_root: &Path) -> Option<PathBuf> {
    let lock = SchemaLock::load(workspace_root)?;
    lock.resolve_schema_dir(workspace_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_schema_lock() {
        let toml = r#"
version = "v1.1.0-alpha"
hash = "sha256:abc123"

[source]
path = "../../../familiar-schemas"
"#;
        let lock: SchemaLock = toml::from_str(toml).unwrap();
        assert_eq!(lock.version, "v1.1.0-alpha");
        assert_eq!(lock.source.path, Some("../../../familiar-schemas".to_string()));
    }
}

