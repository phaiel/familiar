//! Agent Speaker Types
//!
//! Speaker identifiers for the multi-agent orchestration system.

use serde::{Deserialize, Serialize};

/// Speaker/agent identifiers for the orchestration system
/// 
/// Each speaker is a specialized agent that handles specific types of tasks.
/// The orchestrator routes to the appropriate speaker based on user intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentSpeaker {
    /// Main user-facing agent - greets users, explains capabilities
    Concierge,
    /// Classification agent - determines entity types, message intent
    Classifier,
    /// Physics simulation agent - handles VAE space calculations
    Physics,
    /// Retrieval-augmented generation agent (future)
    Rag,
    /// Memory/context management agent
    Memory,
    /// Task execution agent
    TaskExecutor,
}

impl AgentSpeaker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Concierge => "concierge",
            Self::Classifier => "classifier",
            Self::Physics => "physics",
            Self::Rag => "rag",
            Self::Memory => "memory",
            Self::TaskExecutor => "task_executor",
        }
    }
}

impl std::fmt::Display for AgentSpeaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}




