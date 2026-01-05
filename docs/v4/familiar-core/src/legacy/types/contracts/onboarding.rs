//! Onboarding Types
//!
//! Domain types for user signup and family creation flows.

use serde::{Deserialize, Serialize};

/// User consent flags for signup
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SignupConsents {
    /// User accepted terms of service
    #[serde(default)]
    pub terms: bool,
    /// User accepted privacy policy
    #[serde(default)]
    pub privacy: bool,
}

/// Request context for audit logging
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RequestContext {
    /// Client IP address (may be behind proxy)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User agent string
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Request ID for correlation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

// =============================================================================
// Onboarding Command Payloads
// =============================================================================

/// Signup request payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SignupRequest {
    /// User's email address
    pub email: String,
    /// User's password (will be hashed by worker)
    pub password: String,
    /// User's display name
    pub name: String,
    /// User consent flags
    #[serde(default)]
    pub consents: SignupConsents,
    /// Request context for audit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_context: Option<RequestContext>,
    /// Invite code if signing up via invitation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_code: Option<String>,
}

/// Create family request payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateFamilyRequest {
    /// Name for the new family
    pub family_name: String,
}

/// Accept invitation request payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AcceptInvitationRequest {
    /// The invitation code to accept
    pub invitation_code: String,
}

// =============================================================================
// Onboarding Event Payloads
// =============================================================================

/// Signup completed event payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SignupCompleted {
    /// Session token for the new user
    pub session_token: String,
    /// Whether the user needs to create/join a family
    pub needs_family: bool,
}

/// Family created event payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FamilyCreated {
    /// Name of the created tenant
    pub tenant_name: String,
}

/// Invitation accepted event payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InvitationAccepted {
    /// Name of the family joined
    pub family_name: String,
}

/// Onboarding failed event payload
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OnboardingFailed {
    /// Error code for programmatic handling
    pub error_code: String,
    /// Human-readable error message
    pub message: String,
}






