//! # familiar-contracts
//!
//! **GENERATED CRATE** - Do not add business logic here.
//!
//! This crate contains pure data types generated from `familiar-schemas`.
//! All behavior (impl blocks, trait implementations) belongs in
//! `familiar-core/src/impl_/` modules.
//!
//! ## Regeneration
//!
//! Run `cargo xtask codegen generate` to regenerate from schemas.
//!
//! Never manually edit generated files - changes will be lost.

// Include embedded schemas for runtime access
include!(concat!(env!("OUT_DIR"), "/embedded_schemas.rs"));

// Re-export common dependencies used by generated code
pub use chrono::{DateTime, Utc};
pub use schemars::JsonSchema;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use uuid::Uuid;

// Re-export primitives FIRST - these provide NormalizedFloat, QuantizedCoord, etc.
// that are used by generated types as field types
pub use familiar_primitives::*;

// Generated types from schemas
// These import primitives via `use super::*` or Rust's name resolution
mod generated;
pub use generated::*;

/// Prelude module for convenient imports in impl blocks
/// 
/// Usage: `use familiar_contracts::prelude::*;`
pub mod prelude {
    // Re-export everything for impl blocks
    pub use super::*;
    pub use familiar_primitives::*;
}
