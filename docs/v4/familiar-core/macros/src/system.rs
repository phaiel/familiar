//! System generation macro implementation.
//!
//! Generates System trait implementations from ECS system schema files.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::config;

pub fn generate(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let schema_path = input_str.trim_matches('"').trim();
    
    let full_path = config::get_schema_path(schema_path);
    
    let schema_content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => {
            let error_msg = format!(
                "Failed to read system schema at '{}': {}",
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
            let error_msg = format!("Failed to parse system schema JSON: {}", e);
            return quote! {
                compile_error!(#error_msg);
            }.into();
        }
    };
    
    generate_system_impl(&schema).into()
}

fn generate_system_impl(schema: &serde_json::Value) -> TokenStream2 {
    let name = schema.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("UnnamedSystem");
    
    let description = schema.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let input_type = schema.get("input_type")
        .and_then(|v| v.as_str())
        .unwrap_or("serde_json::Value");
    
    let output_type = schema.get("output_type")
        .and_then(|v| v.as_str())
        .unwrap_or("serde_json::Value");
    
    let resource_class = schema.get("resource_class")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    
    let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());
    let input_type_ident = syn::Ident::new(input_type, proc_macro2::Span::call_site());
    let output_type_ident = syn::Ident::new(output_type, proc_macro2::Span::call_site());
    let logic_fn_name = syn::Ident::new(
        &format!("{}_logic", to_snake_case(name)),
        proc_macro2::Span::call_site()
    );
    
    let doc_comment = format!("{}\n\nResource class: {}", description, resource_class);
    
    quote! {
        #[doc = #doc_comment]
        #[derive(Debug, Clone)]
        pub struct #struct_name {
            // Components will be injected by the Node
        }
        
        impl #struct_name {
            pub const NAME: &'static str = stringify!(#struct_name);
            pub const RESOURCE_CLASS: &'static str = #resource_class;
            
            pub fn new() -> Self {
                Self {}
            }
        }
        
        #[async_trait::async_trait]
        impl crate::runtime::System for #struct_name {
            type Input = #input_type_ident;
            type Output = #output_type_ident;
            
            fn name(&self) -> &'static str {
                Self::NAME
            }
            
            fn resource_class(&self) -> &'static str {
                Self::RESOURCE_CLASS
            }
            
            async fn execute(&self, input: Self::Input) -> Result<Self::Output, crate::runtime::SystemError> {
                // Call the hand-written logic function
                crate::logic::#logic_fn_name(self, input).await
            }
        }
    }
}

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

