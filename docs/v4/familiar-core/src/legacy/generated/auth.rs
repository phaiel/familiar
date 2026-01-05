//! Generated auth types from JSON schemas.
//!
//! This module re-exports auth types from types/auth/ (manually maintained).
//! The contracts/ module is being phased in via the crate architecture cleanup.

// === Auth types from types/auth/ (manual, with behavior) ===
pub use crate::types::auth::AuditLogEntry;
pub use crate::types::auth::CreateUserInput;
pub use crate::types::auth::LoginRequest;
pub use crate::types::auth::MagicLinkRequest;
pub use crate::types::auth::CreateMagicLinkInput;
pub use crate::types::auth::SessionCreated;
pub use crate::types::auth::CreateJoinRequestInput;
pub use crate::types::auth::CreateEmailInviteInput;
pub use crate::types::auth::CreateCodeInviteInput;

// These will be generated to contracts/ once codegen is complete:
// - AuthResponse
// - UpdateUserInput
// - MagicLinkCreated
// - RecordConsentInput
// - RequestDeletionInput
// - ReviewJoinRequestInput
