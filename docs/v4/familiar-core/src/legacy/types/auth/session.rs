//! Session Types
//!
//! Types for authentication session management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::{SessionId, UserId};

/// An authenticated session
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AuthSession {
    pub id: SessionId,
    pub user_id: UserId,
    /// Token hash (the actual token is only returned once on creation)
    pub token_hash: String,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a session
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateSessionInput {
    pub user_id: UserId,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    /// Session duration in hours (default 24 * 7 = 1 week)
    #[serde(default = "default_session_hours")]
    pub expires_in_hours: i64,
}

fn default_session_hours() -> i64 {
    24 * 7 // 1 week
}

/// Result of creating a session (includes the raw token)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SessionCreated {
    pub session_id: SessionId,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

