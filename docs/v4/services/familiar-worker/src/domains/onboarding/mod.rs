//! Onboarding Domain - User Provisioning
//!
//! Handles user signup, family creation, and invitation acceptance.
//! All operations are atomic (wrapped in SeaORM transactions).
//!
//! ## Module Structure
//!
//! - `signup`, `create_family`, `accept_invitation`: High-level workflow steps
//! - `router`: Evaluator pattern routing
//! - `db_ops`: Direct database operations for Windmill script replacement

pub mod signup;
pub mod create_family;
pub mod accept_invitation;
pub mod router;
pub mod db_ops;

use serde::{Deserialize, Serialize};

/// Request for new user signup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    #[serde(default)]
    pub invite_code: Option<String>,
    #[serde(default)]
    pub consents: SignupConsents,
}

/// Consent flags for signup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignupConsents {
    #[serde(default)]
    pub terms_accepted: bool,
    #[serde(default)]
    pub privacy_accepted: bool,
    #[serde(default)]
    pub marketing_opt_in: bool,
}

/// Response from signup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupResponse {
    pub user_id: String,
    pub tenant_id: String,
    pub email: String,
    pub session_token: Option<String>,
}

/// Request to create a new family/tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFamilyRequest {
    pub user_id: String,
    pub family_name: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Response from family creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFamilyResponse {
    pub tenant_id: String,
    pub family_name: String,
    pub owner_id: String,
}

/// Request to accept a family invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationRequest {
    pub invitation_code: String,
    pub user_id: String,
}

/// Response from invitation acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationResponse {
    pub tenant_id: String,
    pub member_role: String,
    pub accepted_at: String,
}

