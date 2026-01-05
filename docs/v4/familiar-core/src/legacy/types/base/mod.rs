//! Base Types
//!
//! Common base types for schema composition:
//! - `EntityMeta`: Common metadata for tenant-scoped entities
//! - `SystemEntityMeta`: Common metadata for system-level entities

pub mod entity_meta;

pub use self::entity_meta::{EntityMeta, SystemEntityMeta};




