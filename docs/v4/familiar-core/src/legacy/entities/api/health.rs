//! GET /health endpoint entity
//!
//! Standard health check.

use serde::{Deserialize, Serialize};

/// Response payload for GET /health
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

impl HealthResponse {
    pub fn healthy(service: &str, version: &str) -> Self {
        Self {
            status: "healthy".to_string(),
            service: service.to_string(),
            version: version.to_string(),
        }
    }
}

