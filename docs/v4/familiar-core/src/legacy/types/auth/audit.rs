//! Audit Log Types
//!
//! Types for security audit logging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::UserId;

/// An audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AuditLogEntry {
    pub id: Uuid,  // AuditLogId
    #[serde(default)]
    pub user_id: Option<UserId>,
    #[serde(default)]
    pub user_email: Option<String>,
    pub action: String,
    #[serde(default)]
    pub resource_type: Option<String>,
    #[serde(default)]
    pub resource_id: Option<Uuid>,  // Could be any resource type
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub success: bool,
    #[serde(default)]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating an audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateAuditLogInput {
    pub user_id: Option<UserId>,
    pub user_email: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,  // Could be any resource type
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub success: bool,
    pub error_message: Option<String>,
}

