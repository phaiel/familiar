//! Base Tool Schema Types
//!
//! Core types for all tool definitions including parameters, results, and errors.

use serde::{Deserialize, Serialize};

use crate::primitives::TokenUsage;

// ============================================================================
// Tool Definition
// ============================================================================

/// Base schema for all tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolDefinition {
    /// Unique identifier for the tool
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Version string (semver)
    pub version: String,
    /// Input parameters schema
    pub parameters: Vec<ToolParameter>,
    /// Category for organization
    pub category: ToolCategory,
    /// Whether this tool can run in parallel with others
    pub parallelizable: bool,
    /// Estimated latency in milliseconds
    #[serde(default)]
    pub estimated_latency_ms: Option<u32>,
}

/// Categories of tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// Segmentation tools (break input into units)
    Segmentation,
    /// Classification tools (label content)
    Classification,
    /// Spawn tools (create entities)
    Spawn,
    /// Hint generation tools
    Hints,
    /// Search and retrieval tools
    Retrieval,
    /// Orchestration tools
    Orchestration,
}

// ============================================================================
// Tool Parameters
// ============================================================================

/// Parameter definition with type and validation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Description of what this parameter does
    pub description: String,
    /// Data type
    pub param_type: ParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    /// Validation constraints
    #[serde(default)]
    pub constraints: Option<ParameterConstraints>,
}

/// Supported parameter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    /// Reference to another entity
    EntityRef,
    /// Base64 encoded binary data
    Binary,
}

/// Validation constraints for parameters
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ParameterConstraints {
    /// Minimum value (for numbers) or length (for strings/arrays)
    #[serde(default)]
    pub min: Option<f64>,
    /// Maximum value (for numbers) or length (for strings/arrays)
    #[serde(default)]
    pub max: Option<f64>,
    /// Allowed values (enum constraint)
    #[serde(default)]
    pub allowed_values: Option<Vec<String>>,
    /// Regex pattern for string validation
    #[serde(default)]
    pub pattern: Option<String>,
}

// ============================================================================
// Tool Results
// ============================================================================

/// Standardized result wrapper for all tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolResult<T> {
    /// Whether the tool executed successfully
    pub success: bool,
    /// The result data (if successful)
    #[serde(default)]
    pub data: Option<T>,
    /// Error information (if failed)
    #[serde(default)]
    pub error: Option<ToolError>,
    /// Execution metadata
    pub metadata: ToolExecutionMetadata,
}

/// Tool execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolExecutionMetadata {
    /// Tool name that was executed
    pub tool_name: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when execution started
    pub started_at: String,
    /// Request/trace ID for debugging
    #[serde(default)]
    pub trace_id: Option<String>,
    /// Model used (if LLM-based)
    #[serde(default)]
    pub model: Option<String>,
    /// Token usage (if LLM-based)
    #[serde(default)]
    pub token_usage: Option<TokenUsage>,
}

// ============================================================================
// Tool Errors
// ============================================================================

/// Error types for tool failures
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolError {
    /// Error code for programmatic handling
    pub code: ToolErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    /// Whether this error is retryable
    pub retryable: bool,
}

/// Error codes for tool failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolErrorCode {
    /// Invalid input parameters
    InvalidInput,
    /// Missing required parameter
    MissingParameter,
    /// Parameter validation failed
    ValidationFailed,
    /// Tool execution timed out
    Timeout,
    /// Rate limit exceeded
    RateLimited,
    /// External service unavailable
    ServiceUnavailable,
    /// LLM API error
    LlmError,
    /// Internal tool error
    InternalError,
    /// Resource not found
    NotFound,
    /// Permission denied
    PermissionDenied,
}

impl ToolError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::InvalidInput,
            message: message.into(),
            details: None,
            retryable: false,
        }
    }

    pub fn llm_error(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::LlmError,
            message: message.into(),
            details: None,
            retryable: true,
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::Timeout,
            message: message.into(),
            details: None,
            retryable: true,
        }
    }
}

// ============================================================================
// Modality Types
// ============================================================================

/// Supported input modalities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Audio,
    Vision,
    Video,
}

/// Input container for different modalities
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "modality", rename_all = "snake_case")]
pub enum ModalityInput {
    Text {
        content: String,
        #[serde(default)]
        language: Option<String>,
    },
    Audio {
        /// Base64 encoded audio data
        data: String,
        /// Audio format (mp3, wav, etc.)
        format: String,
        /// Duration in seconds
        #[serde(default)]
        duration_seconds: Option<f64>,
    },
    Vision {
        /// Base64 encoded image data
        data: String,
        /// Image format (png, jpg, etc.)
        format: String,
        /// Image dimensions
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },
    Video {
        /// Base64 encoded video data or URL
        data: String,
        /// Video format (mp4, webm, etc.)
        format: String,
        /// Duration in seconds
        #[serde(default)]
        duration_seconds: Option<f64>,
        /// Frame rate
        #[serde(default)]
        fps: Option<f64>,
    },
}

impl ModalityInput {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            language: None,
        }
    }

    pub fn modality(&self) -> Modality {
        match self {
            Self::Text { .. } => Modality::Text,
            Self::Audio { .. } => Modality::Audio,
            Self::Vision { .. } => Modality::Vision,
            Self::Video { .. } => Modality::Video,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition {
            name: "text_segmentation".to_string(),
            description: "Segment text into semantic units".to_string(),
            version: "1.0.0".to_string(),
            parameters: vec![],
            category: ToolCategory::Segmentation,
            parallelizable: true,
            estimated_latency_ms: Some(500),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("text_segmentation"));
    }

    #[test]
    fn test_modality_input() {
        let input = ModalityInput::text("Hello world");
        assert_eq!(input.modality(), Modality::Text);
    }
}
