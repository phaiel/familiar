//! Agentic System Types
//!
//! Discrete type modules for multi-agent orchestration following the LlamaIndex
//! concierge pattern:
//! - `log_level` - Log levels for agent messages
//! - `finding` - Findings from analysis agents
//! - `message_type` - Agent message type variants
//! - `speaker` - Agent speaker identifiers
//! - `conversation_turn` - Conversation history tracking
//! - `state` - Global agent state
//! - `orchestration` - Orchestration input/output
//! - `tool_call` - Tool invocation types
//!
//! Reference: https://www.llamaindex.ai/blog/building-a-multi-agent-concierge-system

pub mod log_level;
pub mod finding;
pub mod message_type;
pub mod speaker;
pub mod conversation_turn;
pub mod state;
pub mod orchestration;
pub mod tool_call;

// Re-exports for backwards compatibility
pub use self::log_level::LogLevel;
pub use self::finding::Finding;
pub use self::message_type::AgentMessageType;
pub use self::speaker::AgentSpeaker;
pub use self::conversation_turn::ConversationTurn;
pub use self::state::AgentState;
pub use self::orchestration::{OrchestrationInput, OrchestrationOutput};
pub use self::tool_call::{ToolCallRequest, ToolCallResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_builder() {
        let mut state = AgentState::new("tenant-123")
            .with_speaker("concierge");
        
        state.add_turn("user", "Hello!");
        state.add_turn("assistant", "Hi there!");
        
        assert_eq!(state.tenant_id, "tenant-123");
        assert_eq!(state.current_speaker, Some("concierge".to_string()));
        assert_eq!(state.conversation_context.len(), 2);
    }

    #[test]
    fn test_agent_message_serialization() {
        let msg = AgentMessageType::Question {
            prompt: "What would you like to do?".to_string(),
            options: Some(vec!["Option A".to_string(), "Option B".to_string()]),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("message_type"));
        assert!(json.contains("question"));
    }
}




