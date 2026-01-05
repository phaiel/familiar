//! GDPR Types
//!
//! Types for GDPR data export and deletion requests.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::UserId;

// ============================================================================
// Data Export
// ============================================================================

/// Status of a data export request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExportStatus {
    Pending,
    Processing,
    Ready,
    Expired,
    Failed,
}

impl ExportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Expired => "expired",
            Self::Failed => "failed",
        }
    }
}

/// A data export request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DataExportRequest {
    pub id: Uuid,  // ExportRequestId
    pub user_id: UserId,
    pub status: ExportStatus,
    #[serde(default)]
    pub export_url: Option<String>,
    #[serde(default)]
    pub export_format: Option<String>,
    #[serde(default)]
    pub file_size_bytes: Option<i64>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Deletion Request
// ============================================================================

/// Status of a deletion request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeletionStatus {
    Pending,
    Cancelled,
    Processing,
    Completed,
}

impl DeletionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Cancelled => "cancelled",
            Self::Processing => "processing",
            Self::Completed => "completed",
        }
    }
}

/// A deletion request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeletionRequest {
    pub id: Uuid,  // DeletionRequestId
    #[serde(default)]
    pub user_id: Option<UserId>,
    pub email: String,
    pub status: DeletionStatus,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub scheduled_for: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for requesting deletion
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RequestDeletionInput {
    #[serde(default)]
    pub reason: Option<String>,
}

