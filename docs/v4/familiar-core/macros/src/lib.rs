//! # familiar-core-new-macros
//!
//! Procedural macros for generating Rust types from JSON schemas.
//! These macros read JSON schema files at compile time and generate
//! corresponding Rust structs with serde derive attributes.
//!
//! ## Macros
//!
//! - `generate_node!` - Generate a Node struct from an ECS node schema
//! - `generate_system!` - Generate a System implementation from a system schema
//! - `generate_type!` - Generate a simple data type from a JSON schema
//!
//! ## Schema Path Configuration
//!
//! Set `FAMILIAR_SCHEMAS_PATH` environment variable to override the default
//! schema location. This can be configured in `.cargo/config.toml`:
//!
//! Schema path is configured via `schema.lock` in familiar-core.
//! See `familiar-core/schema.lock` [source] section.

use proc_macro::TokenStream;

mod config;
mod node;
mod system;
mod types;

/// Generate a Node struct from a JSON schema file.
///
/// The path is relative to the schema directory configured in `schema.lock`.
///
/// # Example
///
/// ```ignore
/// generate_node!("ecs/nodes/fates_daemon.node.json");
/// ```
#[proc_macro]
pub fn generate_node(input: TokenStream) -> TokenStream {
    node::generate(input)
}

/// Generate a System implementation from a JSON schema file.
///
/// # Example
///
/// ```ignore
/// generate_system!("ecs/systems/fates_gate.system.json");
/// ```
#[proc_macro]
pub fn generate_system(input: TokenStream) -> TokenStream {
    system::generate(input)
}

/// Generate a simple type from a JSON schema file.
///
/// This is for "simple" types - data structures without complex business logic.
///
/// # Example
///
/// ```ignore
/// generate_type!("tools/GateInput.schema.json");
/// ```
#[proc_macro]
pub fn generate_type(input: TokenStream) -> TokenStream {
    types::generate(input)
}

