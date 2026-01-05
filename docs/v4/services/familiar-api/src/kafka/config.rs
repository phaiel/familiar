//! Kafka Configuration
//!
//! Configuration for Kafka/Redpanda connection.
//! Now uses AppConfig from config.toml

use crate::config::AppConfig;

/// Kafka configuration loaded from AppConfig
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    /// Bootstrap servers (comma-separated)
    pub bootstrap_servers: String,
    /// Client ID for this instance
    pub client_id: String,
    /// Consumer group ID
    pub group_id: String,
    /// Command topic name
    pub commands_topic: String,
    /// Events topic name
    pub events_topic: String,
    /// Trace topic name
    pub trace_topic: String,
}

impl KafkaConfig {
    /// Load configuration from AppConfig
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            bootstrap_servers: config.kafka.bootstrap_servers.clone(),
            client_id: config.kafka.client_id.clone(),
            group_id: config.kafka.group_id.clone(),
            commands_topic: config.kafka.commands_topic.clone(),
            events_topic: config.kafka.events_topic.clone(),
            trace_topic: config.kafka.trace_topic.clone(),
        }
    }

    /// Create a new config with defaults for local development
    pub fn local() -> Self {
        Self {
            bootstrap_servers: "localhost:19092".to_string(),
            client_id: "familiar-api".to_string(),
            group_id: "familiar-api-group".to_string(),
            commands_topic: "course.commands".to_string(),
            events_topic: "course.events".to_string(),
            trace_topic: "course.trace".to_string(),
        }
    }
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self::local()
    }
}




