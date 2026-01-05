//! Schema-driven code generation for Familiar
//!
//! This crate provides code generation capabilities that were moved out of
//! familiar-schemas to keep the schema library pure and immutable.

pub mod config;
pub mod names;
pub mod rust;

pub use config::{CodegenConfig, RenderProfile, NamingConfig, Language};
pub use names::{NameResolver, ResolvedName, TypeOrigin, NameResolverStats};
pub use rust::generate_rust;
