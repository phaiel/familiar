//! Type generation macro implementation using `typify`.
//!
//! Uses the `typify` crate (by Oxide Computer) to generate professional-grade
//! Rust structs from JSON Schema files. This handles edge cases like:
//! - `oneOf` with discriminators (tagged unions)
//! - Complex validation constraints
//! - Nested type references
//!
//! ## Ontological Filter
//!
//! NOT all schemas can be generated. The following patterns require manual
//! maintenance with drift checking:
//! - `anyOf` - Ambiguous logic; requires hand-written validation
//! - Untagged `oneOf` - Performance killer; requires custom Visitor
//! - Complex `allOf` - Better handled with `#[serde(flatten)]` in Rust
//!
//! If a schema uses these patterns, the macro emits `compile_error!` to force
//! the type into the `MANUAL_TYPES` list.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::config;

pub fn generate(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let schema_path = input_str.trim_matches('"').trim();
    
    let full_path = config::get_schema_path(schema_path);
    
    // Read the schema file
    let schema_content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => {
            let error_msg = format!(
                "Failed to read type schema at '{}': {}. Set FAMILIAR_SCHEMAS_PATH env var to override.",
                full_path.display(),
                e
            );
            return quote! {
                compile_error!(#error_msg);
            }.into();
        }
    };
    
    // Parse the JSON schema
    let schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("Failed to parse type schema JSON: {}", e);
            return quote! {
                compile_error!(#error_msg);
            }.into();
        }
    };
    
    // Ontological Filter: Check for complex patterns that require manual maintenance
    if let Some(rejection_reason) = check_ontological_complexity(&schema) {
        let type_name = schema.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let error_msg = format!(
            "Schema '{}' ({}) is too complex for generation: {}. Add to MANUAL_TYPES list instead.",
            type_name, schema_path, rejection_reason
        );
        return quote! {
            compile_error!(#error_msg);
        }.into();
    }
    
    // Use typify to generate the Rust type
    generate_with_typify(&schema, schema_path).into()
}

/// Check if a schema has patterns that are too complex for automatic generation.
/// Returns Some(reason) if the schema should be manual, None if it can be generated.
fn check_ontological_complexity(schema: &serde_json::Value) -> Option<String> {
    // Check for anyOf - always requires manual handling
    if schema.get("anyOf").is_some() {
        return Some("Uses 'anyOf' which creates ambiguous Rust types".to_string());
    }
    
    // Check for oneOf without discriminator (untagged unions)
    if let Some(one_of) = schema.get("oneOf") {
        if one_of.is_array() && schema.get("discriminator").is_none() {
            // Check if any variant has a "type" or "kind" const field (informal discriminator)
            let has_informal_discriminator = one_of.as_array()
                .map(|variants| variants.iter().any(|v| {
                    v.get("properties")
                        .and_then(|p| p.get("type").or_else(|| p.get("kind")))
                        .and_then(|t| t.get("const"))
                        .is_some()
                }))
                .unwrap_or(false);
            
            if !has_informal_discriminator {
                return Some("Uses untagged 'oneOf' - add discriminator or move to MANUAL_TYPES".to_string());
            }
        }
    }
    
    // Check for complex allOf with potential conflicts
    if let Some(all_of) = schema.get("allOf") {
        if let Some(arr) = all_of.as_array() {
            // If there are multiple objects with "properties", there could be conflicts
            let property_counts: Vec<usize> = arr.iter()
                .filter_map(|v| v.get("properties"))
                .filter_map(|p| p.as_object())
                .map(|o| o.len())
                .collect();
            
            if property_counts.len() > 2 || property_counts.iter().sum::<usize>() > 10 {
                return Some("Complex 'allOf' with many properties - use #[serde(flatten)] manually".to_string());
            }
        }
    }
    
    // Recursively check nested definitions
    if let Some(definitions) = schema.get("definitions").or_else(|| schema.get("$defs")) {
        if let Some(defs) = definitions.as_object() {
            for (name, def) in defs {
                if let Some(reason) = check_ontological_complexity(def) {
                    return Some(format!("Definition '{}': {}", name, reason));
                }
            }
        }
    }
    
    None
}

fn generate_with_typify(schema: &serde_json::Value, schema_path: &str) -> TokenStream2 {
    // Configure typify settings for our codebase conventions
    // Note: typify adds derives internally, so we just configure what we want
    let mut settings = typify::TypeSpaceSettings::default();
    settings.with_derive("Debug".to_string());
    settings.with_derive("Clone".to_string());
    
    // Create a typify TypeSpace with our settings
    let mut type_space = typify::TypeSpace::new(&settings);
    
    // Convert serde_json::Value to schemars::schema::RootSchema
    let root_schema: schemars::schema::RootSchema = match serde_json::from_value(schema.clone()) {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("Failed to convert schema to RootSchema: {}", e);
            return quote! {
                compile_error!(#error_msg);
            };
        }
    };
    
    // Add the schema to typify's type space
    if let Err(e) = type_space.add_root_schema(root_schema) {
        let error_msg = format!("typify failed to process schema '{}': {}", schema_path, e);
        return quote! {
            compile_error!(#error_msg);
        };
    }
    
    // Generate the Rust code
    // typify already generates Serialize/Deserialize derives
    let generated = type_space.to_stream();
    
    generated
}
