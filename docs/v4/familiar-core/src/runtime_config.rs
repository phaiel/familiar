//! Core Runtime Configuration
//!
//! Loads runtime configuration from config.toml with environment variable overrides.
//! This is separate from the schema-driven config module.

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct CoreRuntimeConfig {
    pub windmill: WindmillConfig,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct WindmillConfig {
    pub url: String,
    pub workspace: String,
    pub token: String,
}

impl CoreRuntimeConfig {
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
        if let Ok(url) = std::env::var("WINDMILL_URL") {
            builder = builder.set_override("windmill.url", url)?;
        }
        if let Ok(workspace) = std::env::var("WINDMILL_WORKSPACE") {
            builder = builder.set_override("windmill.workspace", workspace)?;
        }
        if let Ok(token) = std::env::var("WINDMILL_TOKEN") {
            builder = builder.set_override("windmill.token", token)?;
        }

        builder.build()?.try_deserialize()
    }
}

