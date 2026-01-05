//! Embedded schema registry.
//!
//! This module embeds the JSON schemas from familiar-schemas at compile time
//! using `include_dir!`. The embedded schemas are used for runtime validation
//! via the ContractEnforcer.
//!
//! Note: The path resolution for include_dir! requires a compile-time literal.
//! The path is: familiar-core-new (docs/v4/familiar-core-new) -> familiar-schemas (familiar/familiar-schemas)
//! Which is: up 3 levels to "familiar/", then into "familiar-schemas/"

pub mod generated_version;
pub mod graph;

pub use generated_version::{SCHEMA_HASH, SCHEMA_VERSION};
pub use graph::SchemaGraph;

// Get schemas from familiar-contracts
pub use familiar_contracts::SCHEMAS;

/// Get the version of a specific schema.
pub fn get_schema_version(_path: &str) -> &'static str {
    SCHEMA_VERSION
}

/// Get a schema file by path relative to the json-schema directory.
pub fn get_schema(path: &str) -> Option<&'static str> {
    SCHEMAS.get_file(path).and_then(|f| f.contents_utf8())
}

/// List all schema files in a directory.
pub fn list_schemas(dir_path: &str) -> Vec<&'static str> {
    SCHEMAS.get_dir(dir_path)
        .map(|dir| {
            dir.files()
                .filter_map(|f| f.path().to_str())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schemas_embedded() {
        // This test will fail if the schemas aren't properly embedded
        // It's OK if it fails during initial development before schemas exist
        let _schemas = &SCHEMAS;
    }
}

