//! Schema Cache - Loaded from schema.lock configuration
//!
//! Provides a cache of schema names that can be used across all analyzers
//! to check if a type has a canonical schema definition.
//!
//! Schema location is determined by `schema.lock` configuration, NOT hardcoded paths.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::analysis::SchemaCache;
//!
//! let cache = SchemaCache::new(workspace_root);
//!
//! // Check if a type has a schema
//! if cache.exists("AgentState") {
//!     println!("AgentState has a canonical schema");
//! }
//!
//! // Get schema path
//! if let Some(path) = cache.find_path("AgentState") {
//!     println!("Schema at: {}", path);
//! }
//! ```

use crate::config::schema_lock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;

/// Cache of schema names from familiar-schemas
/// 
/// This is a simple name-based cache. It does NOT validate schema content,
/// just checks if a schema file exists for a given type name.
/// 
/// Schema content validation is handled by `schema-drift` tool, not the analyzer.
#[derive(Debug, Clone)]
pub struct SchemaCache {
    /// Set of all schema names (e.g., "AgentState", "UserId")
    names: HashSet<String>,
    /// Map of schema name to its category/path (e.g., "AgentState" -> "agentic")
    paths: HashMap<String, String>,
    /// Root path where familiar-schemas was found
    schemas_root: Option<PathBuf>,
}

impl SchemaCache {
    /// Create a new schema cache by loading from familiar-schemas
    pub fn new(workspace_root: &PathBuf) -> Self {
        let (names, paths, schemas_root) = Self::load_schemas(workspace_root);
        Self { names, paths, schemas_root }
    }
    
    /// Check if a type name has a schema in familiar-schemas
    pub fn exists(&self, type_name: &str) -> bool {
        self.names.contains(type_name)
    }
    
    /// Get the category/path for a schema (e.g., "agentic", "primitives")
    pub fn get_category(&self, type_name: &str) -> Option<&str> {
        self.paths.get(type_name).map(|s| s.as_str())
    }
    
    /// Get the full path to a schema file
    pub fn find_path(&self, type_name: &str) -> Option<String> {
        let category = self.paths.get(type_name)?;
        let root = self.schemas_root.as_ref()?;
        Some(root.join(format!("{}/{}.schema.json", category, type_name)).to_string_lossy().to_string())
    }
    
    /// Get all schema names
    pub fn all_names(&self) -> &HashSet<String> {
        &self.names
    }
    
    /// Get count of loaded schemas
    pub fn len(&self) -> usize {
        self.names.len()
    }
    
    /// Check if cache is empty (no schemas found)
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
    
    /// Get the root path where schemas were loaded from
    pub fn schemas_root(&self) -> Option<&PathBuf> {
        self.schemas_root.as_ref()
    }
    
    /// Load all schema names using schema.lock configuration
    fn load_schemas(workspace_root: &PathBuf) -> (HashSet<String>, HashMap<String, String>, Option<PathBuf>) {
        let mut names = HashSet::new();
        let mut paths = HashMap::new();
        let mut found_root = None;
        
        // Use schema.lock to find schema directory
        let base_path = match schema_lock::find_schema_dir(workspace_root) {
            Some(p) => p,
            None => return (names, paths, found_root),
        };
        
        if base_path.exists() {
            found_root = Some(base_path.clone());
            
            // Scan all subdirectories (categories)
            if let Ok(entries) = fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let category = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Scan schema files in this category
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if let Some(name) = Self::extract_schema_name(&sub_path) {
                                    names.insert(name.clone());
                                    paths.insert(name, category.clone());
                                }
                            }
                        }
                    } else if let Some(name) = Self::extract_schema_name(&path) {
                        // Schema file at root level
                        names.insert(name.clone());
                        paths.insert(name, "root".to_string());
                    }
                }
            }
        }
        
        (names, paths, found_root)
    }
    
    /// Extract schema name from a .schema.json file path
    fn extract_schema_name(path: &PathBuf) -> Option<String> {
        let file_name = path.file_name()?.to_str()?;
        if file_name.ends_with(".schema.json") {
            Some(file_name.trim_end_matches(".schema.json").to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_schema_name() {
        assert_eq!(
            SchemaCache::extract_schema_name(&PathBuf::from("AgentState.schema.json")),
            Some("AgentState".to_string())
        );
        assert_eq!(
            SchemaCache::extract_schema_name(&PathBuf::from("foo.json")),
            None
        );
    }
}

