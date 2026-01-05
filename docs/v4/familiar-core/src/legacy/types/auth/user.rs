//! User Types
//!
//! Types for user identity and profile management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::{UserId, TenantId};
use crate::types::base::SystemEntityMeta;

/// A user's identity that can belong to multiple families
/// 
/// Uses `SystemEntityMeta` because users are not tenant-scoped - 
/// a single user can belong to multiple families.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct User {
    /// Entity metadata (id, timestamps)
    #[serde(flatten)]
    pub meta: SystemEntityMeta<UserId>,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// The user's primary family (can be changed)
    #[serde(default)]
    pub primary_tenant_id: Option<TenantId>,
    #[serde(default)]
    pub settings: serde_json::Value,
    #[serde(default)]
    pub gdpr_consents: serde_json::Value,
    #[serde(default)]
    pub deletion_requested_at: Option<DateTime<Utc>>,
}

/// Public user info (safe to expose to other users)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PublicUser {
    pub id: UserId,
    pub name: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// Input for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    /// Password hash (already hashed by API layer)
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub primary_tenant_id: Option<TenantId>,
}

/// Input for updating user profile
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateUserInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub primary_tenant_id: Option<TenantId>,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

