//! Familiar Primitives
//!
//! Semantic ID types and validated primitives shared across the Familiar codebase.
//! This crate provides type-safe wrappers with feature-gated derives.
//!
//! ## Schema-First Architecture
//!
//! This crate does NOT generate schemas from Rust types. Schemas are defined
//! in familiar-schemas and Rust types must match them.
//!
//! ## Features
//!
//! - `serde` (default) - Serialize/Deserialize support
//! - `sqlx` - Database type support (sqlx::Type)
//! - `ts-rs` - TypeScript generation
//! - `schemars` - JSON Schema generation
//! - `sea-orm` - SeaORM entity support
//! - `password-hashing` - Argon2id password hashing
//! - `full` - Enables all optional features
//!
//! ## Usage
//!
//! ```rust
//! use familiar_primitives::{TenantId, UserId, Email};
//!
//! let tenant = TenantId::new();
//! let user = UserId::new();
//! let email = Email::new("user@example.com").unwrap();
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// =============================================================================
// Macro for defining UUID-based ID types with feature-gated derives
// =============================================================================

/// Define a UUID-based ID primitive type with feature-gated derives.
macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[serde(transparent)]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        #[cfg_attr(feature = "ts-rs", ts(export))]
        #[cfg_attr(feature = "ts-rs", ts(as = "String"))]
        pub struct $name(Uuid);

        impl $name {
            #[inline]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[inline]
            pub fn parse(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }

            #[inline]
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }

            #[inline]
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        #[cfg(feature = "sea-orm")]
        impl From<$name> for sea_orm::Value {
            fn from(id: $name) -> Self {
                sea_orm::Value::Uuid(Some(Box::new(id.0)))
            }
        }

        #[cfg(feature = "sea-orm")]
        impl sea_orm::TryGetable for $name {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &sea_orm::QueryResult,
                idx: I,
            ) -> Result<Self, sea_orm::TryGetError> {
                let uuid: Uuid = res.try_get_by(idx)?;
                Ok(Self::from(uuid))
            }
        }

        #[cfg(feature = "sea-orm")]
        impl sea_orm::sea_query::ValueType for $name {
            fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    sea_orm::Value::Uuid(Some(x)) => Ok(Self::from(*x)),
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($name).to_string()
            }

            fn array_type() -> sea_orm::sea_query::ArrayType {
                sea_orm::sea_query::ArrayType::Uuid
            }

            fn column_type() -> sea_orm::sea_query::ColumnType {
                sea_orm::sea_query::ColumnType::Uuid
            }
        }

        #[cfg(feature = "sea-orm")]
        impl sea_orm::sea_query::Nullable for $name {
            fn null() -> sea_orm::Value {
                sea_orm::Value::Uuid(None)
            }
        }

        #[cfg(feature = "sea-orm")]
        impl sea_orm::IntoActiveValue<$name> for $name {
            fn into_active_value(self) -> sea_orm::ActiveValue<$name> {
                sea_orm::ActiveValue::Set(self)
            }
        }

        #[cfg(feature = "sea-orm")]
        impl sea_orm::TryFromU64 for $name {
            fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
                Err(sea_orm::DbErr::ConvertFromU64(concat!(
                    stringify!($name),
                    " cannot be converted from u64"
                )))
            }
        }
    };
}

// =============================================================================
// Domain ID Types
// =============================================================================

define_id!(TenantId, "A tenant (family) unique identifier");
define_id!(UserId, "A user's unique identifier");
define_id!(ChannelId, "A channel's unique identifier");
define_id!(MessageId, "A message's unique identifier");
define_id!(CourseId, "A course (persistent session/history bucket) unique identifier");
define_id!(ShuttleId, "A shuttle (transient unit of work) unique identifier");
define_id!(ThreadId, "A thread (domain entity - Person/Concept) unique identifier");
define_id!(SessionId, "A session's unique identifier");
define_id!(InvitationId, "A unique identifier for family invitations");
define_id!(JoinRequestId, "A unique identifier for join requests");
define_id!(MagicLinkId, "A unique identifier for magic links");
define_id!(AuditLogId, "A unique identifier for audit log entries");
define_id!(ConsentRecordId, "A unique identifier for consent records");
define_id!(TaskId, "A unique identifier for async tasks");
define_id!(EntityId, "A unique identifier for entities");
define_id!(ExportRequestId, "A unique identifier for data export requests");
define_id!(DeletionRequestId, "A unique identifier for data deletion requests");

// =============================================================================
// Email Primitive
// =============================================================================

/// Error for invalid email addresses
#[derive(Debug, Clone, thiserror::Error)]
pub enum EmailError {
    #[error("Email address is empty")]
    Empty,
    #[error("Email address is missing @ symbol")]
    MissingAt,
    #[error("Email address has invalid format")]
    InvalidFormat,
}

/// A validated email address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    pub fn new(email: impl Into<String>) -> Result<Self, EmailError> {
        let email = email.into();
        Self::validate(&email)?;
        Ok(Self(email.to_lowercase()))
    }

    pub fn from_trusted(email: impl Into<String>) -> Self {
        Self(email.into())
    }

    fn validate(email: &str) -> Result<(), EmailError> {
        if email.is_empty() {
            return Err(EmailError::Empty);
        }
        if !email.contains('@') {
            return Err(EmailError::MissingAt);
        }
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(EmailError::InvalidFormat);
        }
        if !parts[1].contains('.') {
            return Err(EmailError::InvalidFormat);
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Invite Code Primitive
// =============================================================================

const INVITE_CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const INVITE_CODE_LENGTH: usize = 8;

/// An invitation code for joining a family
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct InviteCode(String);

impl InviteCode {
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let code: String = (0..INVITE_CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..INVITE_CHARSET.len());
                INVITE_CHARSET[idx] as char
            })
            .collect();
        Self(code)
    }

    pub fn parse(code: impl AsRef<str>) -> Option<Self> {
        let code = code.as_ref().trim().to_uppercase();
        if code.len() != INVITE_CODE_LENGTH {
            return None;
        }
        if !code.chars().all(|c| INVITE_CHARSET.contains(&(c as u8))) {
            return None;
        }
        Some(Self(code))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other.trim())
    }
}

impl fmt::Debug for InviteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InviteCode({})", self.0)
    }
}

impl fmt::Display for InviteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Invite Role Primitive
// =============================================================================

/// Role to assign when invitation is accepted
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum InviteRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "member")]
    Member,
    #[serde(rename = "guest")]
    Guest,
}

impl Default for InviteRole {
    fn default() -> Self {
        Self::Member
    }
}

impl fmt::Display for InviteRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Member => write!(f, "member"),
            Self::Guest => write!(f, "guest"),
        }
    }
}

// =============================================================================
// Password Hash Primitive
// =============================================================================

/// Error type for password hashing operations
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum PasswordHashError {
    #[error("Password hashing failed")]
    HashingFailed,
}

/// A password hash (Argon2id)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_hash(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    pub fn to_string_for_storage(&self) -> String {
        self.0.clone()
    }

    #[cfg(feature = "password-hashing")]
    pub fn verify(&self, password: &str) -> bool {
        use argon2::{
            password_hash::{PasswordHash as ParsedHash, PasswordVerifier},
            Argon2,
        };
        let parsed_hash = match ParsedHash::new(&self.0) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    #[cfg(feature = "password-hashing")]
    pub fn hash(password: &str) -> Result<Self, PasswordHashError> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| PasswordHashError::HashingFailed)?;
        Ok(Self(hash.to_string()))
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PasswordHash([REDACTED])")
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PASSWORD_HASH]")
    }
}

// =============================================================================
// Session Token Primitive
// =============================================================================

/// A session token (bearer token)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn generate() -> Self {
        use rand::Rng;
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        Self(token)
    }

    pub fn from_string(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn matches_hash(&self, hash: &str) -> bool {
        self.hash() == hash
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionToken([REDACTED])")
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() > 8 {
            write!(f, "{}...", &self.0[..8])
        } else {
            write!(f, "[TOKEN]")
        }
    }
}

// =============================================================================
// Numeric Primitives
// =============================================================================

/// A float value normalized to [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct NormalizedFloat(f64);

impl NormalizedFloat {
    pub fn new(value: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&value) {
            Err(format!("Value {} must be between 0.0 and 1.0", value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for NormalizedFloat {
    fn default() -> Self {
        Self(0.0)
    }
}

/// A float value normalized to [-1.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SignedNormalizedFloat(f64);

impl SignedNormalizedFloat {
    pub fn new(value: f64) -> Result<Self, String> {
        if !(-1.0..=1.0).contains(&value) {
            Err(format!("Value {} must be between -1.0 and 1.0", value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn new_clamped(value: f64) -> Self {
        Self(value.clamp(-1.0, 1.0))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for SignedNormalizedFloat {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Temperature controls randomness in LLM outputs (0.0 to 2.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Temperature(f32);

impl Temperature {
    pub const MIN: f32 = 0.0;
    pub const MAX: f32 = 2.0;
    pub const CLASSIFICATION: Self = Self(0.3);
    pub const CREATIVE: Self = Self(0.9);
    pub const DETERMINISTIC: Self = Self(0.0);

    pub fn new(value: f32) -> Result<Self, String> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            Err(format!(
                "Temperature {} must be between {} and {}",
                value,
                Self::MIN,
                Self::MAX
            ))
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self::CLASSIFICATION
    }
}

/// Maximum number of tokens for input/output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MaxTokens(u32);

impl MaxTokens {
    pub const CLASSIFICATION: Self = Self(2048);
    pub const EXTENDED: Self = Self(4096);
    pub const MINIMAL: Self = Self(512);

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for MaxTokens {
    fn default() -> Self {
        Self::CLASSIFICATION
    }
}

/// Token usage statistics from a completion
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
        }
    }
}

// =============================================================================
// API Key Primitive
// =============================================================================

/// A secure wrapper for API keys that prevents accidental logging
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: String) -> Result<Self, String> {
        if key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        if key.len() < 10 {
            return Err("API key appears too short".to_string());
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for ApiKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// =============================================================================
// Quantized Coordinate
// =============================================================================

/// A quantized coordinate in the 3D VAE Manifold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct QuantizedCoord(i64);

impl QuantizedCoord {
    pub const SCALE: i64 = 1_000_000;

    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }

    pub fn from_normalized(value: f64) -> Self {
        let clamped = value.clamp(-1.0, 1.0);
        Self((clamped * Self::SCALE as f64) as i64)
    }

    pub fn from_f64(value: f64) -> Self {
        Self((value * Self::SCALE as f64) as i64)
    }

    pub fn to_normalized(&self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }

    pub fn zero() -> Self {
        Self(0)
    }
}

impl Default for QuantizedCoord {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::ops::Add for QuantizedCoord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for QuantizedCoord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

// =============================================================================
// Database Configuration Primitives
// =============================================================================

/// A validated PostgreSQL connection string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct DbConnectionString(String);

impl DbConnectionString {
    pub fn new(url: impl Into<String>) -> Result<Self, String> {
        let url = url.into();
        if url.is_empty() {
            return Err("Connection string cannot be empty".to_string());
        }
        if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
            return Err(
                "Connection string must start with postgres:// or postgresql://".to_string(),
            );
        }
        Ok(Self(url))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DbConnectionString {
    fn default() -> Self {
        Self("postgresql://localhost:5432/familiar".to_string())
    }
}

/// Connection pool size configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct DbPoolSize(u32);

impl DbPoolSize {
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 100;
    pub const DEFAULT: u32 = 5;

    pub fn new(size: u32) -> Result<Self, String> {
        if !(Self::MIN..=Self::MAX).contains(&size) {
            Err(format!(
                "Pool size {} must be between {} and {}",
                size,
                Self::MIN,
                Self::MAX
            ))
        } else {
            Ok(Self(size))
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for DbPoolSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

// =============================================================================
// Timestamp and UUID Primitives
// =============================================================================

use chrono::{DateTime, Duration, Utc};

/// An ISO 8601 timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn as_utc(&self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::ops::Sub for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Duration {
        self.0 - rhs.0
    }
}

/// A UUID wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UUID(Uuid);

impl UUID {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for UUID {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_roundtrip() {
        let id = TenantId::new();
        let s = id.to_string();
        let parsed = TenantId::parse(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_email_validation() {
        assert!(Email::new("user@example.com").is_ok());
        assert!(Email::new("").is_err());
        assert!(Email::new("noat").is_err());
        assert!(Email::new("user@nodot").is_err());
    }

    #[test]
    fn test_invite_code() {
        let code = InviteCode::generate();
        assert_eq!(code.as_str().len(), INVITE_CODE_LENGTH);
        assert!(InviteCode::parse(code.as_str()).is_some());
    }

    #[test]
    fn test_session_token_hash() {
        let token = SessionToken::generate();
        let hash = token.hash();
        assert!(token.matches_hash(&hash));
    }

    #[test]
    fn test_normalized_float() {
        assert!(NormalizedFloat::new(0.5).is_ok());
        assert!(NormalizedFloat::new(-0.1).is_err());
        assert!(NormalizedFloat::new(1.1).is_err());
    }

    #[test]
    fn test_temperature() {
        assert!(Temperature::new(0.5).is_ok());
        assert!(Temperature::new(-0.1).is_err());
        assert!(Temperature::new(2.5).is_err());
    }
}
