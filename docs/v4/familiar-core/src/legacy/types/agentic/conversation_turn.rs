//! Conversation Turn Types
//!
//! Types for tracking conversation history.

use serde::{Deserialize, Serialize};

/// A single turn in a conversation (for context tracking)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConversationTurn {
    /// Role of the speaker (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Optional speaker identifier (for multi-agent)
    #[serde(default)]
    pub speaker: Option<String>,
    /// Timestamp of the turn
    #[serde(default)]
    pub timestamp: Option<String>,
}




