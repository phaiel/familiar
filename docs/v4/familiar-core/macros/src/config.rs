//! Configuration for schema paths using schema.lock.
//!
//! Schema path is determined by reading `schema.lock` from familiar-core.
//! No hardcoded paths - configuration is centralized in schema.lock.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Schema source from schema.lock [source] section
#[derive(Debug, Deserialize, Default)]
struct SchemaSource {
    path: Option<String>,
}

/// Schema lock file structure
#[derive(Debug, Deserialize)]
struct SchemaLock {
    version: String,
    #[serde(default)]
    source: SchemaSource,
}

/// Cached schema root path
static SCHEMA_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Load schema.lock and resolve the schema directory path
fn resolve_schema_root() -> PathBuf {
    let macros_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // Try to find schema.lock (macros is at familiar-core/macros/)
    let lock_paths = [
        macros_dir.join("../schema.lock"),      // familiar-core/schema.lock
        macros_dir.join("../../schema.lock"),   // workspace/schema.lock
    ];
    
    for lock_path in &lock_paths {
        if let Ok(content) = fs::read_to_string(lock_path) {
            if let Ok(lock) = toml::from_str::<SchemaLock>(&content) {
                if let Some(source_path) = lock.source.path {
                    // Resolve relative to familiar-core directory
                    let base = lock_path.parent().unwrap_or(&macros_dir);
                    let schema_dir = base.join(&source_path)
                        .join("versions")
                        .join(&lock.version)
                        .join("json-schema");
                    
                    if schema_dir.exists() {
                        return schema_dir;
                    }
                }
            }
        }
    }
    
    // Panic with helpful message if schema.lock not found
    panic!(
        "Could not find schema.lock or resolve schema directory.\n\
         Expected schema.lock at: {:?}\n\
         Make sure familiar-core/schema.lock exists with [source].path configured.",
        lock_paths
    );
}

/// Get the path to the JSON schemas directory.
///
/// Reads from schema.lock [source].path configuration.
pub fn get_schemas_root() -> PathBuf {
    SCHEMA_ROOT.get_or_init(resolve_schema_root).clone()
}

/// Get the path to a specific schema file.
pub fn get_schema_path(relative: &str) -> PathBuf {
    get_schemas_root().join(relative)
}
