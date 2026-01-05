//! Tenant Member Types
//!
//! Types for tenant membership management.

use serde::{Deserialize, Serialize};

use crate::primitives::{UserId, TenantId};
use crate::types::base::EntityMeta;

/// Role of a tenant member
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Admin,
    Member,
    Guest,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }
}

/// A member of a tenant (user-tenant association with role)
///
/// Note: Uses UserId as the entity ID since this is a join table
/// where the user_id + tenant_id forms the unique key.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TenantMember {
    /// Entity metadata (id=user_id, tenant_id, timestamps)
    #[serde(flatten)]
    pub meta: EntityMeta<UserId>,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub role: MemberRole,
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// Input for creating a new tenant member
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateMemberInput {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub role: Option<MemberRole>,
}

