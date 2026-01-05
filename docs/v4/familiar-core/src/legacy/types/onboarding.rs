//! Onboarding Types for DAG-based Flow
//!
//! Schema-first types for Windmill onboarding flows.
//! These are the canonical definitions - TypeScript types are generated via ts-rs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::primitives::{UserId, TenantId, ChannelId};

// ============================================================================
// Onboarding State Machine
// ============================================================================

/// State of an onboarding session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum OnboardingState {
    Initial,
    EmailSubmitted,
    MagicLinkSent,
    CredentialsSet,
    SignupComplete,
    InviteCodeEntered,
    FamilyChoice,
    CreatingFamily,
    JoiningFamily,
    RequestSubmitted,
    Complete,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self::Initial
    }
}

/// An onboarding session tracking progress through the flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct OnboardingSession {
    pub id: Uuid,  // OnboardingSessionId
    pub session_key: String,
    pub email: Option<String>,
    pub user_id: Option<UserId>,
    pub state: OnboardingState,
    pub invite_code: Option<String>,
    pub invite_id: Option<Uuid>,  // InvitationId
    pub join_request_id: Option<Uuid>,  // JoinRequestId
    pub steps_completed: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown>"))]
    pub metadata: serde_json::Value,
}

// ============================================================================
// Async Task Types
// ============================================================================

/// Status of an async task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum AsyncTaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for AsyncTaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// An async task tracked in the database
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AsyncTask {
    pub id: Uuid,  // AsyncTaskId
    pub task_type: String,
    pub correlation_id: String,
    pub windmill_job_id: Option<String>,
    pub windmill_flow_path: Option<String>,
    pub status: AsyncTaskStatus,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown>"))]
    pub input: serde_json::Value,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown> | null"))]
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Create an async task
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreateAsyncTaskInput {
    pub task_type: String,
    pub correlation_id: String,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown>"))]
    pub input: serde_json::Value,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub max_attempts: Option<i32>,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown> | null"))]
    pub metadata: Option<serde_json::Value>,
}

/// Response when creating an async task
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AsyncTaskCreated {
    pub task_id: Uuid,  // AsyncTaskId
    pub correlation_id: String,
    pub status: AsyncTaskStatus,
    pub poll_url: String,
}

/// Polling response for async task status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AsyncTaskPollResponse {
    pub task_id: Uuid,  // AsyncTaskId
    pub status: AsyncTaskStatus,
    #[cfg_attr(feature = "ts", ts(type = "Record<string, unknown> | null"))]
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Windmill Flow Input/Output Types
// ============================================================================

/// Consent flags for signup
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SignupConsents {
    pub terms: bool,
    pub privacy: bool,
    pub marketing: Option<bool>,
    pub ai_processing: Option<bool>,
}

/// Input for the signup Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SignupFlowInput {
    pub email: String,
    pub password: Option<String>,
    pub name: String,
    pub invite_code: Option<String>,
    pub consents: SignupConsents,
    pub request_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Output from the signup Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SignupFlowOutput {
    pub user_id: UserId,
    pub session_token: String,
    pub session_expires_at: DateTime<Utc>,
    pub needs_family: bool,
    pub joined_family_id: Option<TenantId>,
    pub joined_family_name: Option<String>,
}

/// Magic link action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum MagicLinkAction {
    Request,
    Consume,
}

/// Input for the magic link Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MagicLinkFlowInput {
    pub action: MagicLinkAction,
    pub email: Option<String>,
    pub token: Option<String>,
    pub name: Option<String>,
    pub invite_code: Option<String>,
    pub request_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Output from magic link request action
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MagicLinkRequestOutput {
    pub success: bool,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    /// In dev mode, return the token for testing
    pub dev_token: Option<String>,
}

/// Output from magic link consume action
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MagicLinkConsumeOutput {
    pub user_id: UserId,
    pub email: String,
    pub is_new_user: bool,
    pub session_token: String,
    pub session_expires_at: DateTime<Utc>,
    pub needs_family: bool,
    pub invite_code: Option<String>,
}

/// Input for the create family Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreateFamilyFlowInput {
    pub user_id: UserId,
    pub family_name: String,
    pub request_id: String,
}

/// Output from the create family Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreateFamilyFlowOutput {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub channel_id: ChannelId,
    pub channel_name: String,
}

/// Input for the accept invitation Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AcceptInvitationFlowInput {
    pub user_id: UserId,
    pub invite_code: String,
    pub request_id: String,
}

/// Output from the accept invitation Windmill flow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AcceptInvitationFlowOutput {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub role: String,
    pub personal_channel_id: ChannelId,
}

// ============================================================================
// Domain Events (for Event Sourcing / Redpanda)
// ============================================================================

/// Domain event for onboarding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(tag = "type", content = "payload")]
pub enum OnboardingEvent {
    /// User started signup process
    #[serde(rename = "user.signup.started")]
    SignupStarted {
        email: String,
        has_invite_code: bool,
    },
    
    /// User signup completed successfully
    #[serde(rename = "user.signup.completed")]
    SignupCompleted {
        user_id: UserId,
        email: String,
        needs_family: bool,
        via_magic_link: bool,
    },
    
    /// Magic link requested
    #[serde(rename = "user.magic_link.requested")]
    MagicLinkRequested {
        email: String,
        purpose: String,
    },
    
    /// Magic link consumed
    #[serde(rename = "user.magic_link.consumed")]
    MagicLinkConsumed {
        user_id: UserId,
        email: String,
        is_new_user: bool,
    },
    
    /// Family created
    #[serde(rename = "family.created")]
    FamilyCreated {
        tenant_id: TenantId,
        tenant_name: String,
        admin_user_id: UserId,
    },
    
    /// User joined family via invitation
    #[serde(rename = "family.member.joined")]
    MemberJoined {
        tenant_id: TenantId,
        user_id: UserId,
        role: String,
        via: String,  // "invite_code" or "email_invite"
    },
    
    /// Join request submitted
    #[serde(rename = "family.member.request.submitted")]
    JoinRequestSubmitted {
        tenant_id: TenantId,
        user_id: UserId,
        request_id: Uuid,  // JoinRequestId
    },
    
    /// Join request reviewed
    #[serde(rename = "family.member.request.reviewed")]
    JoinRequestReviewed {
        tenant_id: TenantId,
        user_id: UserId,
        request_id: Uuid,  // JoinRequestId
        approved: bool,
        reviewer_id: UserId,
    },
}

/// Wrapper for domain events with metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct DomainEventEnvelope {
    pub id: Uuid,  // EventId
    pub event: OnboardingEvent,
    pub correlation_id: String,
    pub causation_id: Option<Uuid>,  // EventId
    pub user_id: Option<UserId>,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: i64,
}

// ============================================================================
// Task Type Constants
// ============================================================================

/// Well-known task types for onboarding
pub mod task_types {
    pub const SIGNUP: &str = "onboarding.signup";
    pub const MAGIC_LINK_REQUEST: &str = "onboarding.magic_link.request";
    pub const MAGIC_LINK_CONSUME: &str = "onboarding.magic_link.consume";
    pub const CREATE_FAMILY: &str = "onboarding.create_family";
    pub const ACCEPT_INVITATION: &str = "onboarding.accept_invitation";
    pub const SUBMIT_JOIN_REQUEST: &str = "onboarding.submit_join_request";
    pub const REVIEW_JOIN_REQUEST: &str = "onboarding.review_join_request";
}

