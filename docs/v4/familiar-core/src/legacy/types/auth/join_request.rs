//! Join Request Types
//!
//! Types for family join requests (user-initiated).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::{UserId, TenantId};

/// Status of a join request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JoinRequestStatus {
    Pending,
    Approved,
    Rejected,
}

impl Default for JoinRequestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl JoinRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

/// A request to join a family
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JoinRequest {
    pub id: Uuid,  // JoinRequestId
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[serde(default)]
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    #[serde(default)]
    pub reviewed_by: Option<UserId>,
    #[serde(default)]
    pub reviewed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a join request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateJoinRequestInput {
    pub tenant_id: TenantId,
    #[serde(default)]
    pub message: Option<String>,
}

/// Input for reviewing a join request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ReviewJoinRequestInput {
    pub approved: bool,
    #[serde(default)]
    pub note: Option<String>,
}

