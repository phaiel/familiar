//! Invitation Types
//!
//! Types for family invitations (email and code-based).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::{TenantId, UserId, InvitationId};

/// Type of family invitation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InviteType {
    /// Sent to a specific email address
    Email,
    /// Shareable code (like Discord)
    Code,
}

impl InviteType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Code => "code",
        }
    }
}

/// Role to assign when invitation is accepted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InviteRole {
    Admin,
    Member,
    Guest,
}

impl Default for InviteRole {
    fn default() -> Self {
        Self::Member
    }
}

impl InviteRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }
}

/// A family invitation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FamilyInvitation {
    pub id: InvitationId,
    pub tenant_id: TenantId,
    #[serde(default)]
    pub invited_by: Option<UserId>,
    pub invite_type: InviteType,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub invite_code: Option<String>,
    pub role: InviteRole,
    pub max_uses: i32,
    pub use_count: i32,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating an email invitation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateEmailInviteInput {
    pub tenant_id: TenantId,
    pub email: String,
    #[serde(default)]
    pub role: Option<InviteRole>,
    /// Expiry in days (default 7)
    #[serde(default = "default_invite_days")]
    pub expires_in_days: i64,
}

/// Input for creating a code invitation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateCodeInviteInput {
    pub tenant_id: TenantId,
    #[serde(default)]
    pub role: Option<InviteRole>,
    /// Max number of uses (default 1, 0 = unlimited)
    #[serde(default = "default_one")]
    pub max_uses: i32,
    /// Expiry in days (None = never)
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

fn default_invite_days() -> i64 {
    7
}

fn default_one() -> i32 {
    1
}

/// Public invitation info (for showing what user is joining)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InvitationInfo {
    pub id: InvitationId,
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub role: InviteRole,
    pub is_valid: bool,
}

