//! # Familiar Router
//!
//! **🎯 ARCHITECTURAL IMPROVEMENT: Now uses proper config resolution**
//!
//! This router implementation has been significantly improved:
//! - ✅ Uses schema-inspired type definitions
//! - ✅ Provides telemetry interfaces
//! - ✅ **NEW**: CEL expressions are validated with real config resolution (no more dummy values)
//! - ✅ **NEW**: Located in familiar-architecture with proper dependencies
//!
//! What still needs improvement for true schema-driven routing:
//! - Generate actual routing algorithms from CEL expressions in system schemas
//! - Build decision trees from constraint hierarchies in node schemas
//! - Create complete routing state machines from schema definitions
//!
//! Current status: Solid foundation with proper config integration.

pub mod context;
pub mod decision;
pub mod trace;
pub mod router;
pub mod telemetry;

// Re-export the generated routing table
include!("generated_routing_table.rs");

pub use router::Router;
pub use context::RoutingContext;
pub use decision::RoutingDecision;
pub use trace::RoutingTrace;
pub use telemetry::TelemetryProvider;

/// Result type for routing operations
pub type Result<T> = std::result::Result<T, RouterError>;

/// Errors that can occur during routing operations
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("CEL evaluation failed: {0}")]
    CelEvaluation(#[from] cel_interpreter::ExecutionError),

    #[error("CEL parsing failed: {0}")]
    CelParsing(#[from] cel_interpreter::ParseError),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Routing table not loaded")]
    RoutingTableNotLoaded,

    #[error("No suitable nodes found for request")]
    NoSuitableNodes,

    #[error("Telemetry provider error: {0}")]
    Telemetry(#[from] anyhow::Error),
}
