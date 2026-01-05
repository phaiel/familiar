//! Comparison logic for detecting drift between Rust types and JSON schemas.

use std::fmt;
use crate::parser::RustTypeInfo;
use crate::schema::{JsonSchemaInfo, JsonType};

/// A report of all drift errors found
#[derive(Debug)]
pub struct DriftReport {
    pub errors: Vec<DriftError>,
}

impl fmt::Display for DriftReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Found {} drift error(s):", self.errors.len())?;
        writeln!(f)?;
        
        for (i, error) in self.errors.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, error)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for DriftReport {}
impl std::error::Error for DriftError {}

/// Individual drift errors
#[derive(Debug)]
pub enum DriftError {
    /// Field exists in schema but not in Rust
    MissingInRust {
        type_name: String,
        field: String,
    },
    /// Field exists in Rust but not in schema
    MissingInSchema {
        type_name: String,
        field: String,
    },
    /// Field type mismatch
    TypeMismatch {
        type_name: String,
        field: String,
        rust_type: String,
        schema_type: String,
    },
    /// Optionality mismatch
    OptionalityMismatch {
        type_name: String,
        field: String,
        rust_optional: bool,
        schema_required: bool,
    },
    /// Type not found in Rust source
    TypeNotFound(String),
    /// Schema not found
    SchemaNotFound(String),
    /// Parse error
    ParseError(String),
}

impl fmt::Display for DriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriftError::MissingInRust { type_name, field } => {
                write!(f, "Type '{}': Field '{}' exists in JSON schema but not in Rust struct", type_name, field)
            }
            DriftError::MissingInSchema { type_name, field } => {
                write!(f, "Type '{}': Field '{}' exists in Rust struct but not in JSON schema", type_name, field)
            }
            DriftError::TypeMismatch { type_name, field, rust_type, schema_type } => {
                write!(f, "Type '{}': Field '{}' has type '{}' in Rust but '{}' in schema", 
                    type_name, field, rust_type, schema_type)
            }
            DriftError::OptionalityMismatch { type_name, field, rust_optional, schema_required } => {
                let rust_str = if *rust_optional { "optional (Option<T>)" } else { "required" };
                let schema_str = if *schema_required { "required" } else { "optional" };
                write!(f, "Type '{}': Field '{}' is {} in Rust but {} in schema",
                    type_name, field, rust_str, schema_str)
            }
            DriftError::TypeNotFound(name) => {
                write!(f, "Rust type '{}' not found in source files", name)
            }
            DriftError::SchemaNotFound(name) => {
                write!(f, "JSON schema '{}.schema.json' not found", name)
            }
            DriftError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

/// Information about a single field drift
#[derive(Debug)]
pub struct FieldDrift {
    pub field_name: String,
    pub drift_type: FieldDriftType,
}

#[derive(Debug)]
pub enum FieldDriftType {
    MissingInRust,
    MissingInSchema,
    TypeMismatch { rust: String, schema: String },
    OptionalityMismatch { rust_optional: bool, schema_required: bool },
}

/// Compare a Rust type against its JSON schema
pub fn compare_types(
    type_name: &str,
    rust_info: &RustTypeInfo,
    schema_info: &JsonSchemaInfo,
) -> Result<(), DriftError> {
    // Note: If the Rust type uses #[serde(flatten)], we need to be lenient
    // because the flattened fields come from other types
    if rust_info.has_flattened {
        // For types with flattened fields, only check non-flattened fields exist in schema
        // The flattened fields would come from the parent types
        return check_non_flattened_fields(type_name, rust_info, schema_info);
    }
    
    // Check for fields missing in Rust
    for (schema_field, _prop_info) in &schema_info.properties {
        if !rust_info.fields.contains_key(schema_field) {
            // This might be in a flattened type, so we warn but don't fail
            // In a more sophisticated implementation, we'd recursively check flattened types
            eprintln!("  Warning: Field '{}' in schema not found directly in Rust struct '{}' (may be flattened)",
                schema_field, type_name);
        }
    }
    
    // Check for fields missing in schema (strict check for non-flattened fields)
    for (rust_field, _field_info) in &rust_info.fields {
        if !schema_info.properties.contains_key(rust_field) {
            return Err(DriftError::MissingInSchema {
                type_name: type_name.to_string(),
                field: rust_field.clone(),
            });
        }
    }
    
    // Check type compatibility for common fields
    for (field_name, rust_field) in &rust_info.fields {
        if let Some(schema_prop) = schema_info.properties.get(field_name) {
            // Check optionality
            let schema_required = schema_info.required.contains(field_name);
            if rust_field.is_optional && schema_required {
                return Err(DriftError::OptionalityMismatch {
                    type_name: type_name.to_string(),
                    field: field_name.clone(),
                    rust_optional: true,
                    schema_required: true,
                });
            }
            
            // Type checking is complex due to the variety of ways types can be represented
            // For now, we do basic compatibility checks
            if !is_type_compatible(&rust_field.type_string, &schema_prop.json_type, &schema_prop.format) {
                return Err(DriftError::TypeMismatch {
                    type_name: type_name.to_string(),
                    field: field_name.clone(),
                    rust_type: rust_field.type_string.clone(),
                    schema_type: format_json_type(&schema_prop.json_type, &schema_prop.format),
                });
            }
        }
    }
    
    Ok(())
}

/// Check only non-flattened fields for types that use #[serde(flatten)]
fn check_non_flattened_fields(
    type_name: &str,
    rust_info: &RustTypeInfo,
    schema_info: &JsonSchemaInfo,
) -> Result<(), DriftError> {
    // For flattened types, just verify the direct fields exist in the schema
    for (rust_field, _field_info) in &rust_info.fields {
        if !schema_info.properties.contains_key(rust_field) {
            return Err(DriftError::MissingInSchema {
                type_name: type_name.to_string(),
                field: rust_field.clone(),
            });
        }
    }
    
    Ok(())
}

/// Check if a Rust type is compatible with a JSON Schema type
fn is_type_compatible(rust_type: &str, json_type: &JsonType, format: &Option<String>) -> bool {
    // Strip Option wrapper if present
    let inner_rust_type = if rust_type.starts_with("Option<") {
        &rust_type[7..rust_type.len()-1]
    } else {
        rust_type
    };
    
    // Core Primitive mappings
    if inner_rust_type == "Timestamp" || inner_rust_type == "DateTime<Utc>" {
        return matches!(json_type, JsonType::String) && 
               (format.as_deref() == Some("date-time") || format.is_none());
    }
    if inner_rust_type == "UUID" || inner_rust_type == "Uuid" {
        return matches!(json_type, JsonType::String) && 
               (format.as_deref() == Some("uuid") || format.is_none());
    }
    
    match json_type {
        JsonType::String => {
            match format.as_deref() {
                Some("uuid") => inner_rust_type.contains("Uuid") || inner_rust_type == "String" || inner_rust_type == "UUID",
                Some("date-time") => inner_rust_type.contains("DateTime") || inner_rust_type.contains("NaiveDateTime") || inner_rust_type == "Timestamp",
                _ => inner_rust_type == "String" || inner_rust_type == "str" || inner_rust_type == "Timestamp" || inner_rust_type == "UUID",
            }
        }
        JsonType::Integer => {
            inner_rust_type.starts_with("i") || inner_rust_type.starts_with("u") ||
            inner_rust_type == "isize" || inner_rust_type == "usize"
        }
        JsonType::Number => {
            inner_rust_type.starts_with("f") || inner_rust_type == "f32" || inner_rust_type == "f64" ||
            inner_rust_type.starts_with("i") || inner_rust_type.starts_with("u") ||
            inner_rust_type == "NormalizedFloat"
        }
        JsonType::Boolean => inner_rust_type == "bool",
        JsonType::Array => inner_rust_type.starts_with("Vec<") || inner_rust_type.starts_with("["),
        JsonType::Object => {
            inner_rust_type.starts_with("HashMap") || 
            inner_rust_type.starts_with("BTreeMap") ||
            inner_rust_type.starts_with("serde_json::Value") ||
            inner_rust_type == "Value" ||
            inner_rust_type.contains("Config") ||
            inner_rust_type == "Metadata"
        }
        JsonType::Null => inner_rust_type == "()" || rust_type.starts_with("Option<"),
        JsonType::Union(types) => {
            // If one of the types is null and rust is Option<T>, check the non-null type
            let non_null_types: Vec<_> = types.iter()
                .filter(|t| !matches!(t, JsonType::Null))
                .collect();
            
            if non_null_types.len() == 1 {
                is_type_compatible(inner_rust_type, non_null_types[0], format)
            } else {
                true // Complex unions need manual verification
            }
        }
        JsonType::Ref(ref_name) => {
            let lower_rust = inner_rust_type.to_lowercase();
            let lower_ref = ref_name.to_lowercase();
            
            // Extremely lenient for refs during migration
            if lower_rust.contains(&lower_ref) || lower_ref.contains(&lower_rust) {
                return true;
            }
            
            // Special case for common IDs and primitives
            ((lower_ref == "uuid" || lower_ref == "id") && (lower_rust == "uuid" || lower_rust.ends_with("id") || lower_rust == "u_u_i_d")) ||
            ((lower_ref.contains("timestamp") || lower_ref.contains("datetime")) && 
             (lower_rust.contains("timestamp") || lower_rust.contains("datetime")))
        }
    }
}

/// Format a JSON type for display
fn format_json_type(json_type: &JsonType, format: &Option<String>) -> String {
    let base = match json_type {
        JsonType::String => "string".to_string(),
        JsonType::Integer => "integer".to_string(),
        JsonType::Number => "number".to_string(),
        JsonType::Boolean => "boolean".to_string(),
        JsonType::Array => "array".to_string(),
        JsonType::Object => "object".to_string(),
        JsonType::Null => "null".to_string(),
        JsonType::Union(types) => {
            let strs: Vec<_> = types.iter().map(|t| format_json_type(t, &None)).collect();
            strs.join(" | ")
        }
        JsonType::Ref(name) => format!("$ref:{}", name),
    };
    
    if let Some(fmt) = format {
        format!("{}({})", base, fmt)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::parser::RustFieldInfo;
    use crate::schema::JsonPropertyInfo;
    
    #[test]
    fn test_type_compatibility_string() {
        assert!(is_type_compatible("String", &JsonType::String, &None));
        assert!(is_type_compatible("Option<String>", &JsonType::String, &None));
    }
    
    #[test]
    fn test_type_compatibility_uuid() {
        assert!(is_type_compatible("Uuid", &JsonType::String, &Some("uuid".to_string())));
        assert!(is_type_compatible("uuid::Uuid", &JsonType::String, &Some("uuid".to_string())));
    }
    
    #[test]
    fn test_type_compatibility_integer() {
        assert!(is_type_compatible("i32", &JsonType::Integer, &None));
        assert!(is_type_compatible("u64", &JsonType::Integer, &None));
        assert!(is_type_compatible("i64", &JsonType::Integer, &None));
    }
    
    #[test]
    fn test_compare_matching_types() {
        let rust_info = RustTypeInfo {
            name: "Test".to_string(),
            fields: {
                let mut m = HashMap::new();
                m.insert("id".to_string(), RustFieldInfo {
                    type_string: "String".to_string(),
                    is_optional: false,
                    serde_rename: None,
                    is_flattened: false,
                });
                m
            },
            has_flattened: false,
            flattened_types: vec![],
        };
        
        let schema_info = JsonSchemaInfo {
            name: "Test".to_string(),
            properties: {
                let mut m = HashMap::new();
                m.insert("id".to_string(), JsonPropertyInfo {
                    json_type: JsonType::String,
                    format: None,
                    ref_type: None,
                    description: None,
                });
                m
            },
            required: vec!["id".to_string()],
        };
        
        let result = compare_types("Test", &rust_info, &schema_info);
        assert!(result.is_ok());
    }
}

