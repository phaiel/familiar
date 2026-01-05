//! Worker Configuration
//!
//! Loads configuration from config.toml with environment variable overrides.

use config::{Config, Environment, File};
use familiar_core::config::CourseConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub kafka: KafkaConfig,
    pub windmill: WindmillConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    /// Course context configuration (token-based pruning)
    #[serde(default)]
    pub course: CourseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub commands_topic: String,
    pub events_topic: String,
    pub trace_topic: String,
    pub envelope_schema_id: i32,
    pub event_schema_id: i32,
    pub trace_schema_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindmillConfig {
    pub url: String,
    pub workspace: String,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl WorkerConfig {
    /// Load configuration from config.toml with environment variable overrides
    /// Supports both APP_ prefixed variables and direct .env variable names
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            // Load from config.toml (if exists)
            .add_source(File::with_name("config").required(false))
            // Override with environment variables (APP_ prefix, double underscore for nesting)
            .add_source(Environment::with_prefix("APP").separator("__"))
            // Also support direct env vars without prefix (for backward compatibility)
            .add_source(Environment::with_prefix("").separator("__"));

        // Add support for direct .env variable names (single underscore)
        if let Ok(servers) = std::env::var("KAFKA_BOOTSTRAP_SERVERS") {
            builder = builder.set_override("kafka.bootstrap_servers", servers)?;
        }
        if let Ok(group) = std::env::var("KAFKA_GROUP_ID") {
            builder = builder.set_override("kafka.group_id", group)?;
        }
        if let Ok(topic) = std::env::var("KAFKA_COMMANDS_TOPIC") {
            builder = builder.set_override("kafka.commands_topic", topic)?;
        }
        if let Ok(topic) = std::env::var("KAFKA_EVENTS_TOPIC") {
            builder = builder.set_override("kafka.events_topic", topic)?;
        }
        if let Ok(topic) = std::env::var("KAFKA_TRACE_TOPIC") {
            builder = builder.set_override("kafka.trace_topic", topic)?;
        }
        if let Ok(url) = std::env::var("WINDMILL_URL") {
            builder = builder.set_override("windmill.url", url)?;
        }
        if let Ok(workspace) = std::env::var("WINDMILL_WORKSPACE") {
            builder = builder.set_override("windmill.workspace", workspace)?;
        }
        if let Ok(token) = std::env::var("WINDMILL_TOKEN") {
            builder = builder.set_override("windmill.token", token)?;
        }
        if let Ok(url) = std::env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", url)?;
        }

        builder.build()?.try_deserialize()
    }
}

