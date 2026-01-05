//! Auth input types without schemas
//!
//! These are internal implementation types used by the infrastructure layer.

use serde::{Deserialize, Serialize};
use familiar_primitives::UserId;

/// Input for creating a new session
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
    24 * 7
}

