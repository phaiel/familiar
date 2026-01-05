//! Shuttle Status Type

use serde::{Deserialize, Serialize};

/// Processing status for a Shuttle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShuttleStatus {
    /// Segment received, not yet processed
    Pending,
    /// Currently being classified by LLM
    Classifying,
    /// Classification complete, spawning entities
    Spawning,
    /// All processing complete
    Complete,
    /// Processing failed
    Failed,
}

impl ShuttleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Classifying => "classifying",
            Self::Spawning => "spawning",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed)
    }

    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }
}

impl Default for ShuttleStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ShuttleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

