//! Database Infrastructure Module
//!
//! Contains the database layer (TigerData/TimescaleDB + pgvector).
//! All DB-specific primitives, types, and components are in their respective
//! top-level modules with `Db` prefix (e.g., DbConnectionString, DbStoreError).

pub mod store;
pub mod media_store;

// Re-export store
pub use store::TigerDataStore;
pub use media_store::{MediaStore, MediaStoreConfig};

// Re-export DB types from their canonical locations
pub use crate::primitives::{DbConnectionString, DbPoolSize};
pub use crate::internal::{DbEntityTable, DbComponentTable, DbPoolConfig, DbStoreError};
