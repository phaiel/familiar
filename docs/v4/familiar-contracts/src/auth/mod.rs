//! auth - GENERATED from familiar-schemas/auth/
//!
//! This file is auto-generated. Do not edit manually.
//! Regenerate with: cargo xtask codegen generate

#![allow(dead_code)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Import primitives from familiar-primitives
use familiar_primitives::{
    AuditLogId, ConsentRecordId, DeletionRequestId, Email, ExportRequestId,
    InvitationId, InviteCode, InviteRole, JoinRequestId, MagicLinkId,
    PasswordHash, SessionId, SessionToken, TenantId, UserId,
};

/// An audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditLogEntry {
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub id: AuditLogId,
    pub ip_address: Option<String>,
    pub metadata: Value,
    pub resource_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub success: bool,
    pub user_agent: Option<String>,
    pub user_email: Option<Email>,
    pub user_id: Option<Uuid>,
}

/// Response after successful authentication
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthResponse {
    pub is_new_user: bool,
    pub needs_family: bool,
    pub session: SessionCreated,
    pub user: User,
}

/// An authenticated session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthSession {
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub id: SessionId,
    pub ip_address: Option<String>,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub user_id: UserId,
}

/// A consent record
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsentRecord {
    pub consent_type: ConsentType,
    pub created_at: DateTime<Utc>,
    pub granted: bool,
    pub id: ConsentRecordId,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: UserId,
    pub version: Option<String>,
}

/// User's current consent status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsentStatus {
    pub ai_processing: Option<DateTime<Utc>>,
    pub analytics: Option<DateTime<Utc>>,
    pub data_sharing: Option<DateTime<Utc>>,
    pub marketing_emails: Option<DateTime<Utc>>,
    pub privacy_policy: Option<DateTime<Utc>>,
    pub terms_of_service: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsentType(pub String);

/// Input for creating a code invitation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCodeInviteInput {
    pub expires_in_days: Option<i64>,
    pub max_uses: i32,
    pub role: Option<InviteRole>,
    pub tenant_id: TenantId,
}

/// Input for creating an email invitation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateEmailInviteInput {
    pub email: Email,
    pub expires_in_days: i64,
    pub role: Option<InviteRole>,
    pub tenant_id: TenantId,
}

/// Input for creating a join request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJoinRequestInput {
    pub message: Option<String>,
    pub tenant_id: TenantId,
}

/// Input for creating a magic link
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMagicLinkInput {
    pub email: Email,
    pub expires_in_minutes: i64,
    pub metadata: Option<Value>,
    pub purpose: MagicLinkPurpose,
}

/// Input for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserInput {
    pub avatar_url: Option<String>,
    pub email: Email,
    pub name: String,
    pub password_hash: Option<PasswordHash>,
    pub primary_tenant_id: Option<Uuid>,
}

/// Current user info (for /api/auth/me)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurrentUser {
    pub memberships: Vec<UserMembership>,
    pub user: User,
}

/// A data export request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataExportRequest {
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub export_format: Option<String>,
    pub export_url: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub id: ExportRequestId,
    pub started_at: Option<DateTime<Utc>>,
    pub status: ExportStatus,
    pub user_id: UserId,
}

/// A deletion request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeletionRequest {
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub email: Email,
    pub id: DeletionRequestId,
    pub reason: Option<String>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub status: DeletionStatus,
    pub user_id: Option<Uuid>,
}

/// Status of a deletion request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeletionStatus(pub String);

/// Status of a data export request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportStatus(pub String);

/// A family invitation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FamilyInvitation {
    pub created_at: DateTime<Utc>,
    pub email: Option<Email>,
    pub expires_at: Option<DateTime<Utc>>,
    pub id: InvitationId,
    pub invite_code: Option<InviteCode>,
    pub invite_type: InviteType,
    pub invited_by: Option<Uuid>,
    pub max_uses: i32,
    pub role: InviteRole,
    pub tenant_id: TenantId,
    pub use_count: i32,
}

/// Public invitation info (for showing what user is joining)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InvitationInfo {
    pub id: InvitationId,
    pub is_valid: bool,
    pub role: InviteRole,
    pub tenant_id: TenantId,
    pub tenant_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InviteType(pub String);

/// A request to join a family
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JoinRequest {
    pub created_at: DateTime<Utc>,
    pub id: JoinRequestId,
    pub message: Option<String>,
    pub review_note: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub status: JoinRequestStatus,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JoinRequestStatus(pub String);

/// Request for email+password login
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub email: Email,
    pub password: String,
}

/// A magic link for passwordless auth
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MagicLink {
    pub created_at: DateTime<Utc>,
    pub email: Email,
    pub expires_at: DateTime<Utc>,
    pub id: MagicLinkId,
    pub metadata: Value,
    pub purpose: MagicLinkPurpose,
    pub used_at: Option<DateTime<Utc>>,
}

/// Result of creating a magic link (includes raw token)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MagicLinkCreated {
    pub expires_at: DateTime<Utc>,
    pub link_id: Uuid,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MagicLinkPurpose(pub String);

/// Request for magic link
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MagicLinkRequest {
    pub email: Email,
    pub invite_code: Option<InviteCode>,
}

/// Public user info (safe to expose to other users)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PublicUser {
    pub avatar_url: Option<String>,
    pub id: UserId,
    pub name: String,
}

/// Input for recording consent
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecordConsentInput {
    pub consent_type: ConsentType,
    pub granted: bool,
    pub version: Option<String>,
}

/// Input for requesting deletion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestDeletionInput {
    pub reason: Option<String>,
}

/// Input for reviewing a join request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReviewJoinRequestInput {
    pub approved: bool,
    pub note: Option<String>,
}

/// Result of creating a session (includes the raw token)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionCreated {
    pub expires_at: DateTime<Utc>,
    pub session_id: SessionId,
    pub token: SessionToken,
}

/// Request for email+password signup
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SignupRequest {
    pub accept_privacy: bool,
    pub accept_terms: bool,
    pub consents: SignupConsents,
    pub email: Email,
    pub invite_code: Option<InviteCode>,
    pub name: String,
    pub password: String,
    pub request_context: Option<RequestContext>,
}

/// Signup consents
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SignupConsents {
    pub ai_processing: bool,
    pub analytics: bool,
    pub data_sharing: bool,
    pub marketing_emails: bool,
}

/// Request context for authentication
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Input for updating user profile
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserInput {
    pub avatar_url: Option<String>,
    pub name: Option<String>,
    pub primary_tenant_id: Option<Uuid>,
    pub settings: Option<Value>,
}

/// A user's identity that can belong to multiple families
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deletion_requested_at: Option<DateTime<Utc>>,
    pub email: Email,
    pub email_verified: bool,
    pub gdpr_consents: Value,
    pub id: UserId,
    pub name: String,
    pub primary_tenant_id: Option<Uuid>,
    pub settings: Value,
    pub updated_at: DateTime<Utc>,
}

/// A user's membership in a family
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserMembership {
    pub is_primary: bool,
    pub joined_at: DateTime<Utc>,
    pub role: InviteRole,
    pub tenant_id: TenantId,
    pub tenant_name: String,
}
