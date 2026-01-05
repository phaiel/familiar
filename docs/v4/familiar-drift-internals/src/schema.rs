//! JSON Schema parser.
//! Extracts type information from JSON Schema files for comparison with Rust types.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::compare::DriftError;

/// Information extracted from a JSON Schema
#[derive(Debug, Clone)]
pub struct JsonSchemaInfo {
    /// The schema title (type name)
    pub name: String,
    /// Map of property name -> property info
    pub properties: HashMap<String, JsonPropertyInfo>,
    /// List of required field names
    pub required: Vec<String>,
}

/// Information about a single JSON Schema property
#[derive(Debug, Clone)]
pub struct JsonPropertyInfo {
    /// The JSON type(s) - can be array for nullable types
    pub json_type: JsonType,
    /// Format hint (e.g., "uuid", "date-time")
    pub format: Option<String>,
    /// Reference to another schema ($ref)
    pub ref_type: Option<String>,
    /// Description from schema
    pub description: Option<String>,
}

/// Represents JSON Schema types
#[derive(Debug, Clone)]
pub enum JsonType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    Null,
    /// Multiple types allowed (e.g., ["string", "null"])
    Union(Vec<JsonType>),
    /// Reference to a definition
    Ref(String),
}

/// Raw JSON Schema structure for parsing
#[derive(Debug, Deserialize, Serialize)]
struct RawSchema {
    #[serde(rename = "$schema")]
    schema: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "type")]
    schema_type: Option<serde_json::Value>,
    properties: Option<HashMap<String, serde_json::Value>>,
    required: Option<Vec<String>>,
    definitions: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "allOf")]
    all_of: Option<Vec<serde_json::Value>>,
}

/// Find and parse a JSON Schema file for a given type name
pub fn find_and_parse_schema(
    schemas_dir: &Path,
    type_name: &str,
) -> Result<JsonSchemaInfo, DriftError> {
    let schema_filename = format!("{}.schema.json", type_name);
    
    // Walk through all directories looking for the schema file
    for entry in WalkDir::new(schemas_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if entry.file_name().to_string_lossy() == schema_filename {
            let content = fs::read_to_string(entry.path()).map_err(|e| {
                DriftError::ParseError(format!(
                    "Failed to read schema {}: {}",
                    entry.path().display(),
                    e
                ))
            })?;
            
            return parse_schema(&content, type_name);
        }
    }
    
    Err(DriftError::SchemaNotFound(type_name.to_string()))
}

/// Parse a JSON Schema string into JsonSchemaInfo
pub fn parse_schema(content: &str, type_name: &str) -> Result<JsonSchemaInfo, DriftError> {
    let raw: RawSchema = serde_json::from_str(content).map_err(|e| {
        DriftError::ParseError(format!("Failed to parse JSON schema for {}: {}", type_name, e))
    })?;
    
    let name = raw.title.unwrap_or_else(|| type_name.to_string());
    let required = raw.required.unwrap_or_default();
    
    let mut properties = HashMap::new();
    
    if let Some(props) = raw.properties {
        for (prop_name, prop_value) in props {
            let prop_info = parse_property(&prop_value)?;
            properties.insert(prop_name, prop_info);
        }
    }
    
    Ok(JsonSchemaInfo {
        name,
        properties,
        required,
    })
}

/// Parse a single property from its JSON value
fn parse_property(value: &serde_json::Value) -> Result<JsonPropertyInfo, DriftError> {
    let obj = value.as_object().ok_or_else(|| {
        DriftError::ParseError("Property is not an object".to_string())
    })?;
    
    // Check for $ref first
    if let Some(ref_val) = obj.get("$ref") {
        let ref_str = ref_val.as_str().unwrap_or("");
        return Ok(JsonPropertyInfo {
            json_type: JsonType::Ref(extract_ref_name(ref_str)),
            format: None,
            ref_type: Some(ref_str.to_string()),
            description: obj.get("description").and_then(|v| v.as_str()).map(String::from),
        });
    }
    
    // Check for allOf, anyOf, oneOf (often used for refs with descriptions)
    for combinator in &["allOf", "anyOf", "oneOf"] {
        if let Some(arr_val) = obj.get(*combinator) {
            if let Some(arr) = arr_val.as_array() {
                for item in arr {
                    if let Some(ref_val) = item.get("$ref") {
                        let ref_str = ref_val.as_str().unwrap_or("");
                        return Ok(JsonPropertyInfo {
                            json_type: JsonType::Ref(extract_ref_name(ref_str)),
                            format: None,
                            ref_type: Some(ref_str.to_string()),
                            description: obj.get("description").and_then(|v| v.as_str()).map(String::from),
                        });
                    }
                }
            }
        }
    }
    
    let json_type = parse_type_field(obj.get("type"));
    let format = obj.get("format").and_then(|v| v.as_str()).map(String::from);
    let description = obj.get("description").and_then(|v| v.as_str()).map(String::from);
    
    Ok(JsonPropertyInfo {
        json_type,
        format,
        ref_type: None,
        description,
    })
}

/// Parse the "type" field which can be a string or array
fn parse_type_field(type_val: Option<&serde_json::Value>) -> JsonType {
    match type_val {
        None => JsonType::Object,
        Some(val) => {
            if let Some(s) = val.as_str() {
                string_to_json_type(s)
            } else if let Some(arr) = val.as_array() {
                let types: Vec<JsonType> = arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(string_to_json_type)
                    .collect();
                if types.len() == 1 {
                    types.into_iter().next().unwrap()
                } else {
                    JsonType::Union(types)
                }
            } else {
                JsonType::Object
            }
        }
    }
}

/// Convert a type string to JsonType
fn string_to_json_type(s: &str) -> JsonType {
    match s {
        "string" => JsonType::String,
        "integer" => JsonType::Integer,
        "number" => JsonType::Number,
        "boolean" => JsonType::Boolean,
        "array" => JsonType::Array,
        "object" => JsonType::Object,
        "null" => JsonType::Null,
        _ => JsonType::String,
    }
}

/// Extract the type name from a $ref string
/// e.g., "#/definitions/MomentType" -> "MomentType"
fn extract_ref_name(ref_str: &str) -> String {
    ref_str.rsplit('/').next().unwrap_or(ref_str).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_schema() {
        let schema = r#"
        {
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "TestType",
            "type": "object",
            "properties": {
                "id": { "type": "string", "format": "uuid" },
                "count": { "type": "integer" }
            },
            "required": ["id", "count"]
        }
        "#;
        
        let info = parse_schema(schema, "TestType").unwrap();
        assert_eq!(info.name, "TestType");
        assert!(info.properties.contains_key("id"));
        assert!(info.properties.contains_key("count"));
        assert!(info.required.contains(&"id".to_string()));
    }
    
    #[test]
    fn test_parse_nullable_type() {
        let schema = r#"
        {
            "title": "NullableTest",
            "type": "object",
            "properties": {
                "value": { "type": ["string", "null"] }
            }
        }
        "#;
        
        let info = parse_schema(schema, "NullableTest").unwrap();
        let value_prop = info.properties.get("value").unwrap();
        assert!(matches!(value_prop.json_type, JsonType::Union(_)));
    }
}

