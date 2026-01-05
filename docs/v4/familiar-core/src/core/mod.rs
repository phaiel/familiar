//! # Core Business Logic (Placeholder)
//!
//! This module will contain hand-written business logic, implementations,
//! and infrastructure code. Unlike `contracts/`, this code is authored
//! manually and provides behavior for the generated types.
//!
//! ## Migration Status
//!
//! The core/ directory structure has been created and files have been copied.
//! However, enabling these modules requires updating all import paths.
//!
//! For now, continue using the legacy modules at the crate root:
//! - `crate::impl_/`
//! - `crate::infrastructure/`
//! - `crate::simulation/`
//! - etc.
//!
//! The contracts/ module is available with generated types from schemas.
//!
//! ## Future Structure (after import migration)
//!
//! - `impl_/` - impl blocks for generated types (behavior injection)
//! - `infrastructure/` - database layer (TigerDataStore, stores)
//! - `simulation/` - physics simulation, entity spawning
//! - `analysis/` - schema analysis tools
//! - `reports/` - report generation (askama templates)
//! - `context.rs` - simulation runtime context
//! - `runtime_config.rs` - runtime configuration (config.toml)

// NOTE: These modules are commented out until import paths are updated
// The files exist in the directory but are not compiled to avoid import errors.
// 
// pub mod impl_;
// pub mod infrastructure;
// pub mod simulation;
// pub mod analysis;
// pub mod reports;
// pub mod context;
// pub mod runtime_config;

// Re-exports will be added after migration
// pub use infrastructure::TigerDataStore;
// pub use context::SimulationContext;
