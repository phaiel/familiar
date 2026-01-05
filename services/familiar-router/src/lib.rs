//! Runtime routing logic for multi-node architecture
//!
//! This crate provides intelligent routing decisions based on CEL expressions
//! defined in schema constraints and routing policies.

pub mod context;
pub mod evaluator;
pub mod router;
pub mod resource_tracker;
pub mod telemetry;

pub use context::NodeContext;
pub use evaluator::CALEvaluator;
pub use router::{RouteDecision, Router, TelemetryProvider, TelemetrySnapshot};
pub use resource_tracker::{ResourceTracker, ResourceRequirements, ResourceLease, ResourceUtilization};
pub use telemetry::MockTelemetryProvider;

/// Errors that can occur during routing operations
#[derive(thiserror::Error, Debug)]
pub enum RouterError {
    #[error("CEL evaluation error: {0}")]
    Celeval(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Route not found for trigger: {0}")]
    RouteNotFound(String),

    #[error("Invalid routing policy: {0}")]
    InvalidPolicy(String),

    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    #[error("Node constraint violation: {0}")]
    ConstraintViolation(String),
}

/// Result type for routing operations
pub type RouterResult<T> = Result<T, RouterError>;
