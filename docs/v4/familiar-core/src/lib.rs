//! # familiar-core
//!
//! Schema-first enforcement engine for Familiar.
//!
//! This crate provides the core types, entities, and infrastructure for the Familiar
//! system. All types are generated from JSON schemas in `familiar-schemas`.
//!
//! ## Architecture
//!
//! ### From familiar-contracts (generated types)
//! Pure data types automatically generated from `familiar-schemas`:
//! - Tool inputs/outputs
//! - API request/response types
//! - Entity structs
//!
//! ### Business Logic Modules
//! - `impl_/` - impl blocks for generated types
//! - `infrastructure/` - database layer
//! - `simulation/` - physics, entity spawning
//! - `analysis/` - schema analysis tools
//!
//! ## Regeneration
//!
//! ```bash
//! cargo xtask codegen generate
//! ```
//!
//! ## Feature Flags
//!
//! - `schema-registry` - Enable Schema Registry integration
//! - `protobuf` - Enable Protobuf serialization for Kafka
//! - `password-hashing` - Enable password hashing
//! - `kafka-codegen` - Enable Kafka codegen CLI

// =============================================================================
// RE-EXPORTS FROM CRATES
// =============================================================================

/// Re-export all types from familiar-contracts (generated from schemas)
pub use familiar_contracts::*;

/// Re-export all primitives from familiar-primitives
pub use familiar_primitives::*;

// =============================================================================
// IMPL BLOCKS: Behavior for generated types
// =============================================================================

/// Impl blocks for generated contract types (methods, trait impls)
pub mod impl_;

// =============================================================================
// INTERNAL: Implementation-specific types (not schema-generated)
// =============================================================================

/// Internal types with Rust-specific impl blocks (errors, adapters, secrets)
pub mod internal;

/// SeaORM entity models for database tables  
pub mod entities;

// =============================================================================
// LEGACY: Kept until schema structure matches expected nested patterns
// =============================================================================
//
// BLOCKED BY: Schema structural differences
// - Generated types use flat fields (created_at, updated_at)
// - Code expects nested meta structures (meta.timestamps.created_at)
// - Missing input types (CreateAuditLogInput, etc.)
//
// TO FIX: Update schemas to include nested meta/timestamps structures
// See: familiar-schemas/versions/latest/json-schema/conversation/Tenant.schema.json

/// Legacy hand-written types - kept until schemas match structure
#[doc(hidden)]
pub mod legacy;

// Temporary re-exports for files still using legacy imports
pub use legacy::types;
pub use legacy::components;

// =============================================================================
// BUSINESS LOGIC MODULES
// =============================================================================

/// Hand-written business logic and infrastructure
pub mod core;

/// Primitives (mostly re-exports from familiar-primitives + core-specific)
pub mod primitives;
pub mod context; // Simulation Runtime
pub mod infrastructure; // Database Layer
pub mod simulation; // Entity spawning, physics, heddle processing
pub mod config; // Schema-driven configuration
pub mod runtime_config; // Runtime configuration (config.toml)
pub mod analysis; // Schema usage analysis tools
pub mod reports; // Report generation (askama + tabled)
pub mod validation; // Contract validation (simd-json + jsonschema)

// Schema-first modules
pub mod schemas;
pub mod runtime;

// MCP (uses familiar_schemas::graph)
pub mod mcp;

// Codegen - re-export from familiar-schemas
// The local codegen module has been moved to familiar-schemas::codegen
#[cfg(feature = "codegen")]
pub mod codegen {
    //! Re-export codegen from familiar-schemas
    //! 
    //! This module is deprecated - use familiar_schemas::codegen directly.
    pub use familiar_schemas::codegen::*;
    pub use familiar_schemas::graph::*;
}

// Kafka integration (requires protobuf feature)
#[cfg(feature = "protobuf")]
pub mod kafka;

// =============================================================================
// RE-EXPORTS: Commonly used types
// =============================================================================

// Re-export internal types (errors, adapters)
pub use internal::{
    DbStoreError, ObserverError, OptimisticLockError,
    EvaluationResult, EvaluationStep, WindmillSecrets,
    OpenAIMessage, AnthropicConversation, GoogleConversation,
    to_openai_messages,
};

// Re-export validation (ContractEnforcer)
pub use validation::{ContractEnforcer, ContractError};

// Re-export config (includes SystemManifest)
pub use config::*;

// Re-export context
pub use context::SimulationContext;

// Re-export infrastructure (TigerData)
pub use infrastructure::TigerDataStore;

// Re-export simulation (Spawning + Physics + Heddle)
pub use simulation::{
    spawn_from_weave_unit, spawn_from_weave_units,
    entity_id, entity_type_name, entity_content, entity_physics, entity_physics_values,
    EntitySpawn, get_heddle_manifest, DEFAULT_COLLAPSE_THRESHOLD,
    get_field_potential_manifest, minimize_action, calculate_field_potential,
};

// Re-export runtime (System trait)
pub use runtime::{System, SystemError};
