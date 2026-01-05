//! Magic Link Types
//!
//! Types for passwordless authentication via magic links.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Purpose of a magic link
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MagicLinkPurpose {
    Login,
    Signup,
    VerifyEmail,
    PasswordReset,
}

impl MagicLinkPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Signup => "signup",
            Self::VerifyEmail => "verify_email",
            Self::PasswordReset => "password_reset",
        }
    }
}

/// A magic link for passwordless auth
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MagicLink {
    pub id: Uuid,
    pub email: String,
    pub purpose: MagicLinkPurpose,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    #[serde(default)]
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a magic link
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateMagicLinkInput {
    pub email: String,
    pub purpose: MagicLinkPurpose,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Link duration in minutes (default 15)
    #[serde(default = "default_magic_link_minutes")]
    pub expires_in_minutes: i64,
}

fn default_magic_link_minutes() -> i64 {
    15
}

/// Result of creating a magic link (includes raw token)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MagicLinkCreated {
    pub link_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}




