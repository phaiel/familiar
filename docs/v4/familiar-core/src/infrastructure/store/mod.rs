//! TigerData Store - Modular Persistence Layer
//!
//! This module organizes database access into domain-specific submodules:
//! - `conversation`: Tenants, Channels, Messages, FamiliarEntities
//! - `auth`: Users, Sessions, MagicLinks, Invitations, JoinRequests
//! - `physics`: Pulse/Moment saving, Quantum states, Field excitations
//! - `task`: Async task tracking for Kafka command processing
//!
//! Each submodule extends `TigerDataStore` with domain-specific methods.
//!
//! NOTE: All database access should use SeaORM entities.
//! Direct sqlx pool access has been removed to enforce this.

use sea_orm::DatabaseConnection;
use crate::internal::{DbStoreError, DbPoolConfig};
use crate::primitives::DbConnectionString;

pub mod conversation;
pub mod auth;
pub mod physics;

/// TigerData Store - The central persistence handle
pub struct TigerDataStore {
    pub(crate) db: DatabaseConnection,
}

impl TigerDataStore {
    /// Create a new store with the given configuration
    pub async fn new(config: DbPoolConfig) -> Result<Self, DbStoreError> {
        let db = sea_orm::Database::connect(config.connection.as_str())
            .await
            .map_err(|e| DbStoreError::connection(e.to_string()))?;
        
        Ok(Self { db })
    }

    /// Create from an existing SeaORM DatabaseConnection
    /// 
    /// Useful when you already have a connection and want to wrap it
    /// in the TigerDataStore interface.
    pub fn from_connection(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create from a connection string
    pub async fn from_url(url: &str) -> Result<Self, DbStoreError> {
        Self::from_connection_string(url).await
    }

    /// Create from a connection string
    pub async fn from_connection_string(url: &str) -> Result<Self, DbStoreError> {
        let connection = DbConnectionString::new(url)
            .map_err(|e| DbStoreError::connection(e))?;
        let config = DbPoolConfig::new(connection);
        Self::new(config).await
    }

    /// Get the underlying database connection (SeaORM)
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

impl From<sea_orm::DbErr> for DbStoreError {
    fn from(err: sea_orm::DbErr) -> Self {
        DbStoreError::query(err.to_string())
    }
}

