//! Course Types
//!
//! Domain types for course (conversation) workflows.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Course Command Payloads
// =============================================================================

/// Start a new course from a weave
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseStart {
    /// The weave (user input) ID
    pub weave_id: Uuid,
    /// Raw text content
    pub content: String,
    /// Optional context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Multimodal blocks (images, audio, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<serde_json::Value>>,
    /// Prior conversation for context
    #[serde(default)]
    pub conversation_history: Vec<serde_json::Value>,
}

/// Continue an existing course with additional user input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseContinue {
    /// The new user message
    pub user_message: String,
    /// Optional multimodal blocks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<serde_json::Value>>,
}

/// Cancel a running course
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseCancel {
    /// Reason for cancellation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Retry a failed course
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseRetry {
    /// Original command to retry
    pub original_command_id: Uuid,
}

// =============================================================================
// Course Event Payloads
// =============================================================================

/// Course has started processing
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseStarted {
    /// The weave that started this course
    pub weave_id: Uuid,
}

/// Segmentation phase completed
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseSegmented {
    /// Number of segments/units identified
    pub unit_count: usize,
}

/// Classification phase completed
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseClassified {
    /// Entity types identified
    pub entity_types: Vec<String>,
    /// Physics hints computed (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physics_summary: Option<String>,
}

/// Course completed successfully
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseCompleted {
    /// The final response to send to user
    pub response: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Token usage (if tracked)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
}

/// Course failed
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseFailed {
    /// Error message (sanitized for storage)
    pub error: String,
    /// Error code for programmatic handling
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Whether this is retryable
    #[serde(default)]
    pub retryable: bool,
}

/// Course was cancelled
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseCancelled {
    /// Who cancelled (user, system, timeout)
    pub cancelled_by: String,
    /// Reason for cancellation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Course is being retried
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseRetrying {
    /// Retry attempt number
    pub attempt: u32,
    /// Original error that triggered retry
    pub original_error: String,
}






