//! Database error types

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

impl From<sqlx::Error> for DbStoreError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound {
                entity_type: "Unknown".to_string(),
                id: "Unknown".to_string(),
            },
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() || db_err.is_foreign_key_violation() {
                    Self::Constraint { message: db_err.to_string() }
                } else {
                    Self::Query { message: db_err.to_string(), query: None }
                }
            }
            _ => Self::Query { message: err.to_string(), query: None },
        }
    }
}

