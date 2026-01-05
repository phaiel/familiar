//! Generated types from JSON schemas.
//!
//! This module re-exports types from `contracts/` (generated from schemas)
//! and `types/` (manually maintained with behavior).
//!
//! ## Migration Note
//!
//! The `generate_type!` proc macro has been deprecated. Types are now generated
//! using `cargo xtask codegen generate` which outputs to `contracts/generated.rs`.
//! This approach properly handles external `$ref` paths in JSON schemas.
//!
//! ## Module Structure
//!
//! - `api` - API request/response types
//! - `auth` - Authentication types (sessions, users)
//! - `tools` - Tool input/output types

pub mod api;
pub mod auth;
pub mod tools;

// Re-export commonly used generated types
pub use api::*;
pub use auth::*;
pub use tools::*;
