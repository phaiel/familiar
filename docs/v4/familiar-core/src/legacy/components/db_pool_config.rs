//! Database pool configuration component

use serde::{Deserialize, Serialize};
use crate::primitives::{DbConnectionString, DbPoolSize};

/// Database pool configuration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DbPoolConfig {
    /// Database connection string
    pub connection: DbConnectionString,
    /// Maximum pool size
    pub max_connections: DbPoolSize,
    /// Minimum pool size
    pub min_connections: DbPoolSize,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
}

impl DbPoolConfig {
    pub fn new(connection: DbConnectionString) -> Self {
        Self {
            connection,
            max_connections: DbPoolSize::new(10).unwrap(),
            min_connections: DbPoolSize::new(1).unwrap(),
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }


    pub fn with_max_connections(mut self, size: u32) -> Result<Self, String> {
        self.max_connections = DbPoolSize::new(size)?;
        Ok(self)
    }

    pub fn with_min_connections(mut self, size: u32) -> Result<Self, String> {
        self.min_connections = DbPoolSize::new(size)?;
        Ok(self)
    }
}

impl Default for DbPoolConfig {
    fn default() -> Self {
        Self::new(DbConnectionString::default())
    }
}

