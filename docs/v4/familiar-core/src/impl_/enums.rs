//! Display and helper implementations for enum types
//!
//! Provides extension traits for enums from familiar-primitives.
//! Direct impls are not possible for types defined in other crates.

// Note: InviteRole methods (as_str, is_admin, etc.) are defined in familiar-primitives
// Extension traits can be added here for additional functionality if needed

// =============================================================================
// Extension Traits
// =============================================================================

/// Extension trait for InviteRole with additional helpers
pub trait InviteRoleExt {
    fn can_invite(&self) -> bool;
    fn can_manage(&self) -> bool;
}

impl InviteRoleExt for familiar_primitives::InviteRole {
    fn can_invite(&self) -> bool {
        use familiar_primitives::InviteRole::*;
        matches!(self, Admin | Member)
    }

    fn can_manage(&self) -> bool {
        use familiar_primitives::InviteRole::*;
        matches!(self, Admin)
    }
}

// Additional extension traits will be added as needed
