//! # Implementation Blocks for Generated Types
//!
//! This module contains all `impl` blocks for types generated from `familiar-schemas`.
//! The types themselves are defined in `familiar-contracts`, but their behavior
//! (methods, trait implementations) lives here in `familiar-core`.
//!
//! ## Organization
//!
//! - `enums.rs` - Display, as_str(), is_* methods for enum types
//! - `entities.rs` - Entity construction and manipulation
//! - `auth.rs` - Authentication type helpers
//! - `tools.rs` - Tool input/output helpers
//! - `physics.rs` - Physics and VAE space calculations
//! - `status.rs` - Status enum helpers (ShuttleStatus, CourseStatus, etc.)

pub mod enums;
pub mod entities;
pub mod auth;
pub mod tools;
pub mod physics;
pub mod status;

// Entity initialization traits (workaround for Orphan Rule)
mod traits;
pub use traits::*;

// Re-export all impl'd types for convenience
pub use enums::*;
pub use entities::*;
pub use auth::*;
pub use tools::*;
pub use physics::*;
pub use status::*;
