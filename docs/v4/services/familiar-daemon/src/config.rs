//! Daemon Configuration
//!
//! Configuration for the familiar-daemon Temporal worker.
//! Loads from environment variables with sensible defaults.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
}

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// PostgreSQL database URL
    pub database_url: String,

    /// Temporal server URL (gRPC)
    pub temporal_url: String,

    /// Temporal namespace (use "default" for auto-setup image)
    pub temporal_namespace: String,

    /// Task queue name for activities
    pub task_queue: String,

    /// Log level filter
    pub log_level: String,
}

impl DaemonConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".to_string()))?;

        let temporal_url = std::env::var("TEMPORAL_URL")
            .unwrap_or_else(|_| "http://localhost:7233".to_string());

        // Use "default" namespace - Temporal auto-setup only creates this one
        let temporal_namespace = std::env::var("TEMPORAL_NAMESPACE")
            .unwrap_or_else(|_| "default".to_string());

        let task_queue = std::env::var("TEMPORAL_TASK_QUEUE")
            .unwrap_or_else(|_| "fates-pipeline".to_string());

        let log_level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "familiar_daemon=info,familiar_core=info".to_string());

        Ok(Self {
            database_url,
            temporal_url,
            temporal_namespace,
            task_queue,
            log_level,
        })
    }
}

