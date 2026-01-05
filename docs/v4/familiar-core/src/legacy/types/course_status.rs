//! Course Status Type
//!
//! Course-Thread Architecture:
//! - Course status reflects the SESSION state (idle, active, archived)
//! - Processing status (segmenting, classifying, etc.) is on the SHUTTLE

use serde::{Deserialize, Serialize};

/// Session status for a Course
/// 
/// The Course is a persistent session/history bucket.
/// Processing states (Segmenting, Classifying, etc.) belong on the Shuttle.
/// 
/// Course status reflects whether the session is:
/// - Idle: No active processing
/// - Active: Currently being processed by a Shuttle
/// - Archived: Closed/completed, read-only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CourseStatus {
    /// Session is idle, no active processing
    Idle,
    /// Session is active, a Shuttle is processing
    Active,
    /// Session is archived/closed
    Archived,
}

impl CourseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    /// Check if course can accept new messages
    pub fn is_writable(&self) -> bool {
        matches!(self, Self::Idle | Self::Active)
    }
    
    /// Check if course is being actively processed
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl Default for CourseStatus {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for CourseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
