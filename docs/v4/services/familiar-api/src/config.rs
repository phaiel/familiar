//! Application Configuration
//!
//! Loads configuration from config.toml with environment variable overrides.

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub windmill: WindmillConfig,
    pub media_store: MediaStoreConfig,
    pub kafka: KafkaConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT secret for signing/validating tokens
    pub jwt_secret: String,
    /// Token expiry in hours (default: 24)
    #[serde(default = "default_token_expiry")]
    pub token_expiry_hours: u64,
}

fn default_token_expiry() -> u64 {
    24
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindmillConfig {
    pub url: String,
    pub workspace: String,
    pub token: String,
    pub agentic_flow: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MediaStoreConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub group_id: String,
    pub commands_topic: String,
    pub events_topic: String,
    pub trace_topic: String,
    pub envelope_schema_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub dev_mode: bool,
}

impl AppConfig {
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
        if let Ok(url) = std::env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", url)?;
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
        if let Ok(flow) = std::env::var("WINDMILL_AGENTIC_FLOW") {
            builder = builder.set_override("windmill.agentic_flow", flow)?;
        }
        if let Ok(endpoint) = std::env::var("MINIO_ENDPOINT") {
            builder = builder.set_override("media_store.endpoint", endpoint)?;
        }
        if let Ok(key) = std::env::var("MINIO_ACCESS_KEY") {
            builder = builder.set_override("media_store.access_key", key)?;
        }
        if let Ok(key) = std::env::var("MINIO_SECRET_KEY") {
            builder = builder.set_override("media_store.secret_key", key)?;
        }
        if let Ok(bucket) = std::env::var("MINIO_BUCKET") {
            builder = builder.set_override("media_store.bucket", bucket)?;
        }
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
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                builder = builder.set_override("server.port", port_num)?;
            }
        }
        if let Ok(dev_mode) = std::env::var("DEV_MODE") {
            if let Ok(dev_mode_bool) = dev_mode.parse::<bool>() {
                builder = builder.set_override("server.dev_mode", dev_mode_bool)?;
            }
        }
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            builder = builder.set_override("auth.jwt_secret", secret)?;
        }
        if let Ok(expiry) = std::env::var("JWT_TOKEN_EXPIRY_HOURS") {
            if let Ok(expiry_hours) = expiry.parse::<u64>() {
                builder = builder.set_override("auth.token_expiry_hours", expiry_hours)?;
            }
        }

        builder.build()?.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/familiar".to_string(),
            },
            windmill: WindmillConfig {
                url: "http://localhost:8000".to_string(),
                workspace: "familiar".to_string(),
                token: "".to_string(),
                agentic_flow: "f/agentic/main".to_string(),
            },
            media_store: MediaStoreConfig {
                endpoint: "http://localhost:9000".to_string(),
                access_key: "familiar".to_string(),
                secret_key: "familiarpass".to_string(),
                bucket: "familiar-media".to_string(),
                region: "us-east-1".to_string(),
            },
            kafka: KafkaConfig {
                bootstrap_servers: "localhost:19092".to_string(),
                client_id: "familiar-api".to_string(),
                group_id: "familiar-api-group".to_string(),
                commands_topic: "course.commands".to_string(),
                events_topic: "course.events".to_string(),
                trace_topic: "course.trace".to_string(),
                envelope_schema_id: 1,
            },
            server: ServerConfig {
                port: 3001,
                dev_mode: false,
            },
            auth: AuthConfig {
                // In production, this MUST be overridden by JWT_SECRET env var
                jwt_secret: "CHANGE_ME_IN_PRODUCTION_WITH_JWT_SECRET_ENV_VAR".to_string(),
                token_expiry_hours: 24,
            },
        }
    }
}

