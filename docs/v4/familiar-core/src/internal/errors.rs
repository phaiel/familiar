//! Database and observer error types
//!
//! These error types have Rust-specific trait implementations (From, Display, Error)
//! and should not be generated from schemas.

use serde::{Deserialize, Serialize};

/// Errors that can occur during database operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "details")]
pub enum DbStoreError {
    /// Connection failed
    Connection { message: String },
    /// Query execution failed
    Query { message: String, query: Option<String> },
    /// Entity not found
    NotFound { entity_type: String, id: String },
    /// Constraint violation (unique, foreign key, etc.)
    Constraint { message: String },
    /// Serialization/deserialization error
    Serialization { message: String },
    /// Transaction failed
    Transaction { message: String },
    /// Migration error
    Migration { message: String },
    /// Operation failed (e.g. S3/MinIO)
    Operation { message: String },
}

impl DbStoreError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection { message: msg.into() }
    }

    pub fn operation(msg: impl Into<String>) -> Self {
        Self::Operation { message: msg.into() }
    }

    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query { message: msg.into(), query: None }
    }

    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }
}

impl std::fmt::Display for DbStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection { message } => write!(f, "Connection error: {}", message),
            Self::Query { message, .. } => write!(f, "Query error: {}", message),
            Self::NotFound { entity_type, id } => write!(f, "{} not found: {}", entity_type, id),
            Self::Constraint { message } => write!(f, "Constraint violation: {}", message),
            Self::Serialization { message } => write!(f, "Serialization error: {}", message),
            Self::Transaction { message } => write!(f, "Transaction error: {}", message),
            Self::Migration { message } => write!(f, "Migration error: {}", message),
            Self::Operation { message } => write!(f, "Operation error: {}", message),
        }
    }
}

impl std::error::Error for DbStoreError {}

impl From<sea_orm::DbErr> for DbStoreError {
    fn from(err: sea_orm::DbErr) -> Self {
        match err {
            sea_orm::DbErr::RecordNotFound(msg) => Self::NotFound {
                entity_type: "Unknown".to_string(),
                id: msg,
            },
            _ => Self::Query { message: err.to_string(), query: None },
        }
    }
}

/// Errors that can occur during observation/analysis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "details")]
pub enum ObserverError {
    /// Input validation failed
    Validation { message: String },
    /// Processing failed
    Processing { message: String },
    /// External service error
    External { service: String, message: String },
    /// Configuration error
    Config { message: String },
}

impl std::fmt::Display for ObserverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation { message } => write!(f, "Validation error: {}", message),
            Self::Processing { message } => write!(f, "Processing error: {}", message),
            Self::External { service, message } => write!(f, "{} error: {}", service, message),
            Self::Config { message } => write!(f, "Config error: {}", message),
        }
    }
}

impl std::error::Error for ObserverError {}

/// Optimistic locking error for database operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OptimisticLockError {
    pub entity_type: String,
    pub id: String,
    pub expected_version: i32,
    pub actual_version: i32,
}

impl std::fmt::Display for OptimisticLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Optimistic lock failed for {} {}: expected version {}, found {}",
            self.entity_type, self.id, self.expected_version, self.actual_version
        )
    }
}

impl std::error::Error for OptimisticLockError {}


