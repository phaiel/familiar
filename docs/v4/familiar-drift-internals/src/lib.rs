//! # familiar-drift-internals
//!
//! Drift checking library for schema-first architecture.
//! Compares manually-maintained Rust structs against JSON Schema definitions
//! to detect drift and ensure type safety at compile time.
//!
//! Used by `familiar-core-new/build.rs` to fail compilation when manual types
//! don't match their corresponding JSON schemas.

pub mod parser;
pub mod schema;
pub mod compare;

pub use parser::RustTypeInfo;
pub use schema::JsonSchemaInfo;
pub use compare::{DriftReport, DriftError, FieldDrift};

use std::path::Path;
use thiserror::Error;

/// Errors that can occur during drift checking
#[derive(Error, Debug)]
pub enum CheckError {
    #[error("Failed to read Rust source: {0}")]
    RustReadError(#[from] std::io::Error),
    
    #[error("Failed to parse Rust source: {0}")]
    RustParseError(String),
    
    #[error("Failed to parse JSON schema: {0}")]
    SchemaParseError(#[from] serde_json::Error),
    
    #[error("Schema file not found for type: {0}")]
    SchemaNotFound(String),
    
    #[error("Rust type not found: {0}")]
    TypeNotFound(String),
}

/// Check for drift between manually-maintained Rust types and JSON schemas.
/// (Legacy single-directory version - use check_drift_multi for multiple directories)
///
/// # Arguments
/// * `schemas_dir` - Path to the directory containing JSON schemas
/// * `rust_types_dir` - Path to the directory containing Rust type definitions
/// * `manual_types` - List of type names that should be drift-checked
///
/// # Returns
/// * `Ok(())` if no drift detected
/// * `Err(DriftReport)` containing all detected drift issues
pub fn check_drift(
    schemas_dir: &Path,
    rust_types_dir: &Path,
    manual_types: &[&str],
) -> Result<(), DriftReport> {
    check_drift_multi(schemas_dir, &[rust_types_dir], manual_types)
}

/// Check for drift between manually-maintained Rust types and JSON schemas.
/// Searches multiple Rust source directories for type definitions.
///
/// # Arguments
/// * `schemas_dir` - Path to the directory containing JSON schemas
/// * `rust_dirs` - Multiple directories to search for Rust type definitions
/// * `manual_types` - List of type names that should be drift-checked
///
/// # Returns
/// * `Ok(())` if no drift detected
/// * `Err(DriftReport)` containing all detected drift issues
pub fn check_drift_multi(
    schemas_dir: &Path,
    rust_dirs: &[&Path],
    manual_types: &[&str],
) -> Result<(), DriftReport> {
    let mut drift_errors: Vec<DriftError> = Vec::new();
    
    for type_name in manual_types {
        match check_single_type_multi(schemas_dir, rust_dirs, type_name) {
            Ok(()) => {}
            Err(e) => drift_errors.push(e),
        }
    }
    
    if drift_errors.is_empty() {
        Ok(())
    } else {
        Err(DriftReport { errors: drift_errors })
    }
}

/// Check a single type for drift between Rust and JSON schema
fn check_single_type_multi(
    schemas_dir: &Path,
    rust_dirs: &[&Path],
    type_name: &str,
) -> Result<(), DriftError> {
    // Find the JSON schema file
    let schema_info = schema::find_and_parse_schema(schemas_dir, type_name)?;
    
    // Find and parse the Rust type - search all directories
    let rust_info = parser::find_and_parse_rust_type_multi(rust_dirs, type_name)?;
    
    // Compare the two
    compare::compare_types(type_name, &rust_info, &schema_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_drift_report_display() {
        let report = DriftReport {
            errors: vec![
                DriftError::MissingInRust {
                    type_name: "Moment".to_string(),
                    field: "new_field".to_string(),
                },
            ],
        };
        let display = format!("{}", report);
        assert!(display.contains("Moment"));
        assert!(display.contains("new_field"));
    }
}

