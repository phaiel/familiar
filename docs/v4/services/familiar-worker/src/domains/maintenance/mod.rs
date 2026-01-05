//! Maintenance Domain - System Tasks
//!
//! Handles data cleanup and health checks.

pub mod cleanup;
pub mod health;
pub mod metrics;

use serde::{Deserialize, Serialize};

/// Request for data cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRequest {
    /// Retention period in days
    pub retention_days: u32,
    /// Tables to clean (empty = all)
    #[serde(default)]
    pub tables: Vec<String>,
    /// Dry run mode
    #[serde(default)]
    pub dry_run: bool,
}

/// Response from cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResponse {
    pub rows_deleted: u64,
    pub tables_processed: Vec<String>,
    pub dry_run: bool,
}
