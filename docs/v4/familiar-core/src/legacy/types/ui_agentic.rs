//! UI Agentic Types
//!
//! Types for the agentic UI - real-time updates, tool calls with status,
//! thinking steps, and the full flow response structure.
//!
//! Schema-first: These Rust types generate TypeScript via ts-rs.

use serde::{Deserialize, Serialize};

// ============================================================================
// Tool Call Status
// ============================================================================

/// Status of a tool call during execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Complete,
    Error,
}

impl Default for ToolCallStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ============================================================================
// UI Tool Call
// ============================================================================

/// A tool call with real-time status updates for UI display
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool
    pub tool: String,
    /// Arguments passed to the tool
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
    /// Result from the tool
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    /// Current status
    pub status: ToolCallStatus,
}

// ============================================================================
// UI Thinking Step
// ============================================================================

/// A thinking/reasoning step for chain-of-thought visibility
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIThinkingStep {
    /// Unique ID
    pub id: String,
    /// Which agent produced this thought
    pub agent: String,
    /// The thought/reasoning content
    pub thought: String,
    /// Timestamp
    #[serde(default)]
    pub timestamp: Option<String>,
}

// ============================================================================
// Heddle Result (Classification Pipeline)
// ============================================================================

/// Structured result from the Heddle classification pipeline
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIHeddleResult {
    /// Segmented content
    #[serde(default)]
    pub segments: Vec<UIHeddleSegment>,
    /// Classification results
    #[serde(default)]
    pub classifications: Vec<UIClassification>,
    /// Physics hints
    #[serde(default)]
    pub physics: Option<UIPhysicsResult>,
    /// Detected purpose/intent
    #[serde(default)]
    pub purpose: Option<String>,
}

/// A segment from Heddle
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIHeddleSegment {
    pub content: String,
    #[serde(default)]
    pub subject: Option<String>,
}

/// A classification result
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIClassification {
    pub entity_type: String,
    pub confidence: f64,
}

/// Physics simulation result
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIPhysicsResult {
    pub position: [f64; 3],
    pub energy: f64,
    pub temperature: f64,
}

// ============================================================================
// Thread Item
// ============================================================================

/// An item in a conversation thread (user message or AI response)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIThreadItem {
    /// Unique ID
    pub id: String,
    /// Role: "user" or "assistant"
    pub role: String,
    /// Text content
    pub content: String,
    /// Timestamp
    pub timestamp: String,
    /// For assistant messages: which agent
    #[serde(default)]
    pub agent_speaker: Option<String>,
    /// Is this message still being generated?
    #[serde(default)]
    pub is_typing: bool,
    /// Current status
    #[serde(default)]
    pub status: Option<String>,
    /// Activity description for typing state
    #[serde(default)]
    pub current_activity: Option<String>,
    /// Chain of thought steps
    #[serde(default)]
    pub thinking_steps: Vec<UIThinkingStep>,
    /// Tool calls made
    #[serde(default)]
    pub tool_calls: Vec<UIToolCall>,
    /// Heddle classification result
    #[serde(default)]
    pub heddle_result: Option<UIHeddleResult>,
    /// Summary for collapsed display
    #[serde(default)]
    pub summary: Option<String>,
}

// ============================================================================
// Channel Message
// ============================================================================

/// A user message that starts a thread in a channel
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIChannelMessage {
    /// Unique ID
    pub id: String,
    /// Original user message content
    pub content: String,
    /// When posted
    pub timestamp: String,
    /// Thread of responses and follow-ups
    pub thread: Vec<UIThreadItem>,
    /// Is thread expanded in UI?
    #[serde(default)]
    pub is_expanded: bool,
    /// Is this the active/latest message?
    #[serde(default)]
    pub is_active: bool,
}

// ============================================================================
// Channel
// ============================================================================

/// A channel (persistent conversation space)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UIChannel {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Messages in this channel
    pub messages: Vec<UIChannelMessage>,
    /// When created
    pub created_at: String,
    /// When last updated
    pub updated_at: String,
}

// ============================================================================
// Agentic Flow Response
// ============================================================================

/// Full response from the agentic Windmill flow
/// This is what the API returns to the UI
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AgenticFlowResponse {
    /// Request ID for tracing
    pub request_id: String,
    /// Thread ID
    pub thread_id: String,
    /// Which agent responded
    pub agent: String,
    /// The conversational response text
    pub response: String,
    /// Thinking steps (for chain-of-thought visibility)
    #[serde(default)]
    pub thinking_steps: Vec<UIThinkingStep>,
    /// Tool calls made
    #[serde(default)]
    pub tool_calls: Vec<UIToolCall>,
    /// Heddle classification result (if applicable)
    #[serde(default)]
    pub heddle_result: Option<UIHeddleResult>,
    /// Whether there are more tasks
    #[serde(default)]
    pub has_more_tasks: bool,
    /// Next request (if continuation needed)
    #[serde(default)]
    pub next_request: Option<String>,
    /// Updated agent state
    #[serde(default)]
    pub state: Option<serde_json::Value>,
}

// Note: AgenticMessageInput and ConversationHistoryItem are defined in commands.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_status_serialization() {
        let status = ToolCallStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");
    }

    #[test]
    fn test_flow_response_serialization() {
        let response = AgenticFlowResponse {
            request_id: "req-123".to_string(),
            thread_id: "thread-456".to_string(),
            agent: "concierge".to_string(),
            response: "Hello!".to_string(),
            thinking_steps: vec![],
            tool_calls: vec![],
            heddle_result: None,
            has_more_tasks: false,
            next_request: None,
            state: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("concierge"));
    }
}
