//! Evaluation and internal processing types
//!
//! These are internal types used during processing that don't need schemas.

use serde::{Deserialize, Serialize};

/// Result of an evaluation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub step: String,
    pub success: bool,
    pub score: Option<f64>,
    pub message: Option<String>,
    pub details: serde_json::Value,
}

/// A single evaluation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationStep {
    pub name: String,
    pub description: String,
    pub weight: f64,
    pub threshold: Option<f64>,
}

/// Windmill secrets configuration (never in schemas)
#[derive(Debug, Clone)]
pub struct WindmillSecrets {
    pub base_url: String,
    pub token: String,
    pub workspace: String,
}

impl WindmillSecrets {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            base_url: std::env::var("WINDMILL_BASE_URL").ok()?,
            token: std::env::var("WINDMILL_TOKEN").ok()?,
            workspace: std::env::var("WINDMILL_WORKSPACE").ok()?,
        })
    }
}


