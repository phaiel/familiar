//! Finding Types
//!
//! Types for analysis agent findings.

use serde::{Deserialize, Serialize};

/// A finding from an analysis agent
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Finding {
    /// Title or label of the finding
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Confidence score (0.0 to 1.0)
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Related entity IDs
    #[serde(default)]
    pub related_entities: Vec<String>,
}




