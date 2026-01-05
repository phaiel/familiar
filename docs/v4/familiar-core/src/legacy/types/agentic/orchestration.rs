//! Orchestration Types
//!
//! Input/output types for the orchestration agent.

use serde::{Deserialize, Serialize};

use super::state::AgentState;

/// Input payload for the orchestration agent
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OrchestrationInput {
    /// Current agent state
    pub state: AgentState,
    /// User's message content
    #[serde(default)]
    pub user_message: Option<String>,
    /// Request ID for tracing
    pub request_id: String,
}

/// Output from the orchestration agent
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OrchestrationOutput {
    /// Which agent should speak next
    pub next_speaker: String,
    /// Updated state
    pub state: AgentState,
    /// Whether the orchestration loop should continue
    pub should_continue: bool,
    /// Optional message to pass to next agent
    #[serde(default)]
    pub forwarded_message: Option<String>,
}




