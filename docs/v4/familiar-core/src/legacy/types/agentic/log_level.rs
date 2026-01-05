//! Log Level Types
//!
//! Log levels for agent message classification.

use serde::{Deserialize, Serialize};

/// Log level for agent log messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}




