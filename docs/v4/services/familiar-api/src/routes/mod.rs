//! API Routes
//!
//! Common types and route modules for the Familiar API.

use utoipa::ToSchema;
use serde::Serialize;

pub mod health;
pub mod models;
pub mod weave;
pub mod media;
pub mod ws;
pub mod agentic;
pub mod channels;
pub mod entities;
pub mod auth;
pub mod invitations;
pub mod tenants;
pub mod async_tasks;

// ============================================================================
// Common Response Types
// ============================================================================

/// Standard error response returned by all API endpoints
///
/// Uses Rust struct literal syntax with `..Default::default()` for optional fields:
/// ```
/// ErrorResponse { error: "...", code: "...", ..Default::default() }
/// ```
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
    /// Machine-readable error code
    pub code: String,
}

impl ErrorResponse {
    /// Create a new error response with just a message
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: "ERROR".to_string(),
        }
    }

    /// Create a new error response with message and code
    pub fn with_code(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }
}

/// Standard success response for operations without specific return data
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl SuccessResponse {
    pub fn ok() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
        }
    }
}