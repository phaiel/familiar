//! Authentication and Onboarding Types
//!
//! Discrete type modules for user authentication, sessions, and onboarding flows:
//! - `user` - User identity and profile
//! - `session` - Authentication sessions
//! - `magic_link` - Passwordless auth
//! - `invitation` - Family invitations
//! - `join_request` - Join request workflow
//! - `consent` - GDPR consent management
//! - `gdpr` - Data export and deletion
//! - `audit` - Security audit logging
//! - `api` - API request/response types

pub mod user;
pub mod session;
pub mod magic_link;
pub mod invitation;
pub mod join_request;
pub mod consent;
pub mod gdpr;
pub mod audit;
pub mod api;

// Re-exports for backwards compatibility
pub use self::user::{User, PublicUser, CreateUserInput, UpdateUserInput};
pub use self::session::{AuthSession, CreateSessionInput, SessionCreated};
pub use self::magic_link::{MagicLink, MagicLinkPurpose, CreateMagicLinkInput, MagicLinkCreated};
pub use self::invitation::{
    FamilyInvitation, InviteType, InviteRole,
    CreateEmailInviteInput, CreateCodeInviteInput, InvitationInfo,
};
pub use self::join_request::{JoinRequest, JoinRequestStatus, CreateJoinRequestInput, ReviewJoinRequestInput};
pub use self::consent::{ConsentRecord, ConsentType, RecordConsentInput, ConsentStatus};
pub use self::gdpr::{DataExportRequest, ExportStatus, DeletionRequest, DeletionStatus, RequestDeletionInput};
pub use self::audit::{AuditLogEntry, CreateAuditLogInput};
pub use self::api::{SignupRequest, LoginRequest, MagicLinkRequest, AuthResponse, CurrentUser, UserMembership};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_type_serialization() {
        let t = InviteType::Code;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"code\"");
    }

    #[test]
    fn test_consent_type_serialization() {
        let c = ConsentType::PrivacyPolicy;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"privacy_policy\"");
    }

    #[test]
    fn test_magic_link_purpose_serialization() {
        let p = MagicLinkPurpose::VerifyEmail;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "\"verify_email\"");
    }
}




