//! Agent State Types
//!
//! Global state shared across agents in the orchestration loop.

use serde::{Deserialize, Serialize};

use super::conversation_turn::ConversationTurn;

/// Global state shared across all agents in the orchestration loop
/// 
/// This state is passed between agents and maintains context about
/// the current conversation, authentication status, and which agent
/// is currently "speaking".
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AgentState {
    /// Currently active speaker (None means orchestrator decides)
    #[serde(default)]
    pub current_speaker: Option<String>,
    /// Whether the user is authenticated
    #[serde(default)]
    pub is_authenticated: bool,
    /// Whether the current agent just finished its task
    #[serde(default)]
    pub just_finished: bool,
    /// Tenant ID for multi-tenancy
    pub tenant_id: String,
    /// Conversation history for context
    #[serde(default)]
    pub conversation_context: Vec<ConversationTurn>,
    /// Current thread ID (if in a thread)
    #[serde(default)]
    pub thread_id: Option<String>,
    /// Custom metadata for extensibility
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            current_speaker: None,
            is_authenticated: false,
            just_finished: false,
            tenant_id: String::new(),
            conversation_context: vec![],
            thread_id: None,
            metadata: serde_json::Value::Null,
        }
    }
}

impl AgentState {
    /// Create a new state for a tenant
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            ..Default::default()
        }
    }
    
    /// Set the current speaker
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.current_speaker = Some(speaker.into());
        self
    }
    
    /// Add a conversation turn
    pub fn add_turn(&mut self, role: impl Into<String>, content: impl Into<String>) {
        self.conversation_context.push(ConversationTurn {
            role: role.into(),
            content: content.into(),
            speaker: self.current_speaker.clone(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        });
    }
    
    /// Mark the current task as finished
    pub fn finish_task(&mut self) {
        self.just_finished = true;
        self.current_speaker = None;
    }
}




