//! # Internal Types
//!
//! Implementation-specific types that should NOT be generated from schemas.
//! These include error types, Rust-specific wrappers, and internal utilities.
//!
//! ## When to Use This Module
//!
//! Types belong here if they:
//! - Have Rust-specific impl blocks (From, Display, Error traits)
//! - Are internal implementation details not exposed via API
//! - Contain secrets or sensitive data that shouldn't be in schemas
//! - Are tightly coupled to specific Rust crates (SeaORM, sqlx, etc.)

pub mod auth_inputs;
pub mod db_types;
pub mod errors;
pub mod evaluation;
pub mod provider_adapters;

// Re-export commonly used types
pub use auth_inputs::*;
pub use db_types::*;
pub use errors::*;
pub use evaluation::*;
pub use provider_adapters::*;

