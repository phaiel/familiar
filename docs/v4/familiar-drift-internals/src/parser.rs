//! Rust source code parser using syn.
//! Extracts struct definitions and their fields for comparison with JSON schemas.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{self, Fields, Item, Type, Attribute};
use walkdir::WalkDir;

use crate::compare::DriftError;

/// Information extracted from a Rust struct definition
#[derive(Debug, Clone)]
pub struct RustTypeInfo {
    /// The name of the struct
    pub name: String,
    /// Map of field name -> field type (as string)
    pub fields: HashMap<String, RustFieldInfo>,
    /// Whether this struct has #[serde(flatten)] fields
    pub has_flattened: bool,
    /// Names of flattened types (for recursive analysis)
    pub flattened_types: Vec<String>,
}

/// Information about a single Rust struct field
#[derive(Debug, Clone)]
pub struct RustFieldInfo {
    /// The field's type as a string
    pub type_string: String,
    /// Whether this field is optional (Option<T>)
    pub is_optional: bool,
    /// The serde rename if any
    pub serde_rename: Option<String>,
    /// Whether this field is flattened
    pub is_flattened: bool,
    /// Whether this field is skipped by serde
    pub is_skipped: bool,
}

/// Find and parse a Rust type from a directory of .rs files
pub fn find_and_parse_rust_type(
    rust_types_dir: &Path,
    type_name: &str,
) -> Result<RustTypeInfo, DriftError> {
    find_and_parse_rust_type_multi(&[rust_types_dir], type_name)
}

/// Find and parse a Rust type from multiple directories of .rs files
pub fn find_and_parse_rust_type_multi(
    rust_dirs: &[&Path],
    type_name: &str,
) -> Result<RustTypeInfo, DriftError> {
    for rust_dir in rust_dirs {
        if !rust_dir.exists() {
            continue;
        }
        
        // Walk through all .rs files looking for the type
        for entry in WalkDir::new(rust_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", entry.path().display(), e);
                    continue;
                }
            };
            
            if let Some(type_info) = parse_type_from_source(&content, type_name)? {
                return Ok(type_info);
            }
        }
    }
    
    Err(DriftError::TypeNotFound(type_name.to_string()))
}

/// Parse a specific type from Rust source code
pub fn parse_type_from_source(
    source: &str,
    type_name: &str,
) -> Result<Option<RustTypeInfo>, DriftError> {
    let syntax = syn::parse_file(source).map_err(|e| {
        DriftError::ParseError(format!("Failed to parse Rust source: {}", e))
    })?;
    
    for item in syntax.items {
        if let Item::Struct(item_struct) = item {
            if item_struct.ident == type_name {
                return Ok(Some(extract_struct_info(&item_struct)?));
            }
        }
    }
    
    Ok(None)
}

/// Extract type information from a syn struct
fn extract_struct_info(item: &syn::ItemStruct) -> Result<RustTypeInfo, DriftError> {
    let mut fields = HashMap::new();
    let mut has_flattened = false;
    let mut flattened_types = Vec::new();
    
    if let Fields::Named(named_fields) = &item.fields {
        for field in &named_fields.named {
            if let Some(field_name) = &field.ident {
                let field_info = extract_field_info(field)?;
                
                if field_info.is_skipped {
                    continue;
                }
                
                if field_info.is_flattened {
                    has_flattened = true;
                    flattened_types.push(field_info.type_string.clone());
                } else {
                    let name = field_info.serde_rename.clone()
                        .unwrap_or_else(|| field_name.to_string());
                    fields.insert(name, field_info);
                }
            }
        }
    }
    
    Ok(RustTypeInfo {
        name: item.ident.to_string(),
        fields,
        has_flattened,
        flattened_types,
    })
}

/// Extract field information including serde attributes
fn extract_field_info(field: &syn::Field) -> Result<RustFieldInfo, DriftError> {
    let type_string = type_to_string(&field.ty);
    let is_optional = is_option_type(&field.ty);
    let (serde_rename, is_flattened, is_skipped) = parse_serde_attrs(&field.attrs);
    
    Ok(RustFieldInfo {
        type_string,
        is_optional,
        serde_rename,
        is_flattened,
        is_skipped,
    })
}

/// Convert a syn Type to a string representation
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<String> = type_path.path.segments.iter()
                .map(|seg| {
                    let ident = seg.ident.to_string();
                    if seg.arguments.is_empty() {
                        ident
                    } else {
                        match &seg.arguments {
                            syn::PathArguments::AngleBracketed(args) => {
                                let inner: Vec<String> = args.args.iter()
                                    .filter_map(|arg| {
                                        if let syn::GenericArgument::Type(t) = arg {
                                            Some(type_to_string(t))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                format!("{}<{}>", ident, inner.join(", "))
                            }
                            _ => ident,
                        }
                    }
                })
                .collect();
            segments.join("::")
        }
        Type::Array(arr) => format!("[{}; N]", type_to_string(&arr.elem)),
        Type::Tuple(tup) => {
            let inner: Vec<String> = tup.elems.iter().map(type_to_string).collect();
            format!("({})", inner.join(", "))
        }
        _ => "unknown".to_string(),
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Parse serde attributes from field attributes
fn parse_serde_attrs(attrs: &[Attribute]) -> (Option<String>, bool, bool) {
    let mut rename = None;
    let mut is_flattened = false;
    let mut is_skipped = false;
    
    for attr in attrs {
        if attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<syn::LitStr>() {
                            rename = Some(lit.value());
                        }
                    }
                } else if meta.path.is_ident("flatten") {
                    is_flattened = true;
                } else if meta.path.is_ident("skip") {
                    is_skipped = true;
                }
                Ok(())
            });
        }
    }
    
    (rename, is_flattened, is_skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_struct() {
        let source = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct TestType {
                pub id: String,
                pub count: i32,
                #[serde(rename = "custom_name")]
                pub original: String,
            }
        "#;
        
        let result = parse_type_from_source(source, "TestType").unwrap();
        assert!(result.is_some());
        
        let info = result.unwrap();
        assert_eq!(info.name, "TestType");
        assert!(info.fields.contains_key("id"));
        assert!(info.fields.contains_key("count"));
        assert!(info.fields.contains_key("custom_name"));
        assert!(!info.fields.contains_key("original"));
    }
    
    #[test]
    fn test_parse_flattened_struct() {
        let source = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct Moment {
                #[serde(flatten)]
                pub identity: Identity,
                pub moment_type: MomentType,
            }
        "#;
        
        let result = parse_type_from_source(source, "Moment").unwrap();
        assert!(result.is_some());
        
        let info = result.unwrap();
        assert!(info.has_flattened);
        assert!(info.flattened_types.contains(&"Identity".to_string()));
        assert!(info.fields.contains_key("moment_type"));
    }
}

