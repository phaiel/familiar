//! Message Types
//!
//! Types for conversation message management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::{MessageId, ChannelId, UserId};
use crate::types::MessageRole;

/// A message in a channel
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    /// Sender (None for AI/system messages)
    #[serde(default)]
    pub sender_id: Option<UserId>,
    /// Parent message for threading
    #[serde(default)]
    pub parent_id: Option<MessageId>,
    pub role: MessageRole,
    pub content: String,
    /// Which agent generated this (for assistant messages)
    #[serde(default)]
    pub agent_speaker: Option<String>,
    /// Chain of thought steps
    #[serde(default)]
    pub thinking_steps: serde_json::Value,
    /// Tool calls made
    #[serde(default)]
    pub tool_calls: serde_json::Value,
    /// Full weave_result from Fates pipeline
    #[serde(default)]
    pub weave_result: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new message
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateMessageInput {
    pub channel_id: ChannelId,
    #[serde(default)]
    pub sender_id: Option<UserId>,
    #[serde(default)]
    pub parent_id: Option<MessageId>,
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub agent_speaker: Option<String>,
    #[serde(default)]
    pub thinking_steps: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_calls: Option<serde_json::Value>,
    #[serde(default)]
    pub weave_result: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Simplified message for conversation history (LLM context)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

/// Options for listing messages
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListMessagesOptions {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<DateTime<Utc>>,
    #[serde(default)]
    pub after: Option<DateTime<Utc>>,
}

impl Default for ListMessagesOptions {
    fn default() -> Self {
        Self {
            limit: Some(50),
            before: None,
            after: None,
        }
    }
}

