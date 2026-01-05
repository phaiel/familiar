//! Auth API Types
//!
//! Request and response types for authentication API endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::TenantId;
use super::invitation::InviteRole;
use super::session::SessionCreated;
use super::user::User;

/// Request for email+password signup
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    /// Optional invite code to join a family
    #[serde(default)]
    pub invite_code: Option<String>,
    /// Consent to required terms
    pub accept_terms: bool,
    pub accept_privacy: bool,
}

/// Request for email+password login
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Request for magic link
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MagicLinkRequest {
    pub email: String,
    /// Optional invite code for signup flow
    #[serde(default)]
    pub invite_code: Option<String>,
}

/// Response after successful authentication
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AuthResponse {
    pub user: User,
    pub session: SessionCreated,
    /// True if this is a new user (just signed up)
    pub is_new_user: bool,
    /// True if user needs to create/join a family
    pub needs_family: bool,
}

/// Current user info (for /api/auth/me)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CurrentUser {
    pub user: User,
    /// User's family memberships
    pub memberships: Vec<UserMembership>,
}

/// A user's membership in a family
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UserMembership {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub role: InviteRole,
    pub is_primary: bool,
    pub joined_at: DateTime<Utc>,
}

