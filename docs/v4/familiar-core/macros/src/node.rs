//! Node generation macro implementation.
//!
//! Generates Node structs from ECS node schema files.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::config;

pub fn generate(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let schema_path = input_str.trim_matches('"').trim();
    
    let full_path = config::get_schema_path(schema_path);
    
    // Read and parse the schema file at compile time
    let schema_content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => {
            let error_msg = format!(
                "Failed to read node schema at '{}': {}",
                full_path.display(),
                e
            );
            return quote! {
                compile_error!(#error_msg);
            }.into();
        }
    };
    
    let schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(v) => v,
        Err(e) => {
            let error_msg = format!("Failed to parse node schema JSON: {}", e);
            return quote! {
                compile_error!(#error_msg);
            }.into();
        }
    };
    
    // Extract node information from schema
    let generated = generate_node_struct(&schema);
    
    generated.into()
}

fn generate_node_struct(schema: &serde_json::Value) -> TokenStream2 {
    let name = schema.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("UnnamedNode");
    
    let queue = schema.get("queue")
        .and_then(|v| v.as_str())
        .unwrap_or("default-queue");
    
    let systems = schema.get("systems")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>())
        .unwrap_or_default();
    
    let components = schema.get("components")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>())
        .unwrap_or_default();
    
    let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());
    let queue_lit = queue;
    
    // Generate component field names (snake_case)
    let component_fields: Vec<_> = components.iter()
        .map(|c| to_snake_case(c))
        .collect();
    
    let component_types: Vec<_> = components.iter()
        .map(|c| syn::Ident::new(c, proc_macro2::Span::call_site()))
        .collect();
    
    let component_field_idents: Vec<_> = component_fields.iter()
        .map(|f| syn::Ident::new(f, proc_macro2::Span::call_site()))
        .collect();
    
    // Generate system field names (snake_case)
    let system_fields: Vec<_> = systems.iter()
        .map(|s| to_snake_case(s))
        .collect();
    
    let system_types: Vec<_> = systems.iter()
        .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
        .collect();
    
    let system_field_idents: Vec<_> = system_fields.iter()
        .map(|f| syn::Ident::new(f, proc_macro2::Span::call_site()))
        .collect();
    
    quote! {
        /// Generated Node struct from ECS schema
        #[derive(Debug)]
        pub struct #struct_name {
            #(pub #component_field_idents: std::sync::Arc<#component_types>,)*
            #(pub #system_field_idents: #system_types,)*
        }
        
        impl #struct_name {
            /// The Temporal task queue this node processes
            pub const QUEUE: &'static str = #queue_lit;
            
            /// Register all systems with a Temporal worker
            pub fn register<W>(&self, _worker: &mut W) {
                // Registration logic will be implemented by the consumer
                // This is a placeholder for the pattern
            }
        }
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

