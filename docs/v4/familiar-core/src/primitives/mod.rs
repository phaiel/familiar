//! Primitives Module
//!
//! This module re-exports all primitives from `familiar-primitives`
//! and adds a few familiar-core specific types.
//!
//! ## From familiar-primitives
//!
//! All ID types, validated values, and numeric primitives are re-exported:
//! - `UserId`, `TenantId`, `CourseId`, etc. (ID types)
//! - `Email`, `PasswordHash`, `SessionToken`, `InviteCode` (validated values)
//! - `NormalizedFloat`, `Temperature`, `MaxTokens`, etc. (numeric)
//!
//! ## familiar-core Specific
//!
//! - `Settings` - User settings structure
//! - `RetryConfig` - Retry configuration for transient failures

// Re-export everything from familiar-primitives
pub use familiar_primitives::*;

// Core-specific primitives
pub mod settings;
pub mod retry_config;

// Macros for defining new primitives
#[macro_use]
pub mod macros;

// Re-export core-specific types
pub use settings::Settings;
pub use retry_config::RetryConfig;
