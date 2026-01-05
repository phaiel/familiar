//! System trait for zero-copy activity execution.
//!
//! The System trait is the core abstraction for tasks/activities in the ECS architecture.
//! It supports:
//! - Zero-copy JSON parsing via `handle_raw` with SIMD-JSON
//! - Integration with Temporal as Activities
//! - Integration with Rig as Tools
//! - Resource class routing for task queue assignment

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Errors that can occur during system execution
#[derive(Error, Debug)]
pub enum SystemError {
    /// Failed to deserialize input
    #[error("Input deserialization failed: {0}")]
    DeserializationError(String),
    
    /// Failed to serialize output
    #[error("Output serialization failed: {0}")]
    SerializationError(String),
    
    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    ValidationError(String),
    
    /// Business logic error
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    /// System is not available
    #[error("System unavailable: {0}")]
    Unavailable(String),
}

impl From<serde_json::Error> for SystemError {
    fn from(err: serde_json::Error) -> Self {
        SystemError::DeserializationError(err.to_string())
    }
}

impl From<anyhow::Error> for SystemError {
    fn from(err: anyhow::Error) -> Self {
        SystemError::ExecutionError(err.to_string())
    }
}

/// The System trait defines a unit of work that can be executed.
///
/// Systems are the "S" in ECS (Entity-Component-System). Each System:
/// - Has strongly typed Input and Output
/// - Has a name for registration and logging
/// - Has a resource class for task queue routing
/// - Supports zero-copy parsing via `handle_raw`
///
/// # Resource Classes
///
/// Systems declare their resource requirements via `resource_class()`:
/// - `"llm"` - Requires LLM/GPU resources, routed to daemon-queue
/// - `"batch"` - Batch processing, routed to worker-queue
/// - `"io"` - IO-bound operations, routed to worker-queue
/// - `"default"` - No special requirements
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use familiar_core::runtime::{System, SystemError};
///
/// struct FatesGate {
///     // injected components
/// }
///
/// #[async_trait]
/// impl System for FatesGate {
///     type Input = GateInput;
///     type Output = GateOutput;
///
///     fn name(&self) -> &'static str { "FatesGate" }
///     fn resource_class(&self) -> &'static str { "llm" }
///
///     async fn execute(&self, input: Self::Input) -> Result<Self::Output, SystemError> {
///         // Business logic here
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait System: Send + Sync + 'static {
    /// The input type for this system (must be deserializable)
    type Input: DeserializeOwned + Send;
    
    /// The output type for this system (must be serializable)
    type Output: Serialize + Send;
    
    /// The name of this system (used for registration and logging)
    fn name(&self) -> &'static str;
    
    /// The resource class for task queue routing.
    /// 
    /// Returns one of: "llm", "batch", "io", "default"
    fn resource_class(&self) -> &'static str {
        "default"
    }
    
    /// Zero-copy entry point for high-performance execution.
    ///
    /// This method accepts raw bytes and uses SIMD-JSON for zero-copy parsing.
    /// The default implementation delegates to `execute` after parsing.
    ///
    /// # Arguments
    /// * `bytes` - Mutable byte slice containing JSON input (modified in-place by simd-json)
    ///
    /// # Returns
    /// * Serialized JSON output as bytes
    async fn handle_raw(&self, bytes: &mut [u8]) -> Result<Vec<u8>, SystemError> {
        // Use simd-json for zero-copy parsing
        // Note: simd_json::from_slice requires &mut [u8] and modifies in place
        let input: Self::Input = simd_json::from_slice(bytes)
            .map_err(|e| SystemError::DeserializationError(e.to_string()))?;
        
        // Execute the business logic
        let output = self.execute(input).await?;
        
        // Serialize the output
        serde_json::to_vec(&output)
            .map_err(|e| SystemError::SerializationError(e.to_string()))
    }
    
    /// Execute the system's business logic.
    ///
    /// This is the method you implement with your actual logic.
    /// It receives typed input and returns typed output.
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, SystemError>;
}

/// Extension trait for System that provides Rig Tool compatibility
pub trait SystemAsTool: System {
    /// Convert this system to a Rig-compatible tool description
    fn tool_description(&self) -> String {
        format!("System: {}", self.name())
    }
}

// Blanket implementation for all Systems
impl<T: System> SystemAsTool for T {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_error_display() {
        let err = SystemError::DeserializationError("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
    
    #[test]
    fn test_system_error_from_serde() {
        let json_err: Result<i32, _> = serde_json::from_str("not a number");
        let system_err: SystemError = json_err.unwrap_err().into();
        assert!(matches!(system_err, SystemError::DeserializationError(_)));
    }
}

