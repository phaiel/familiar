//! Chat message component

use serde::{Deserialize, Serialize};
use crate::types::MessageRole;

/// A single message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ChatMessage {
    /// The role of the message sender
    pub role: MessageRole,
    /// The message content
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// A conversation (sequence of messages)
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
}

impl Conversation {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn with_system(system: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::system(system)],
        }
    }

    pub fn add(&mut self, message: ChatMessage) -> &mut Self {
        self.messages.push(message);
        self
    }

    pub fn add_system(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(ChatMessage::system(content));
        self
    }

    pub fn add_user(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(ChatMessage::user(content));
        self
    }

    pub fn add_assistant(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(ChatMessage::assistant(content));
        self
    }

    /// Extract system prompt(s) as a single string (for debugging)
    pub fn system_prompt(&self) -> String {
        self.messages.iter()
            .filter(|m| m.role == MessageRole::System)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract user prompt(s) as a single string (for debugging)
    pub fn user_prompt(&self) -> String {
        self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

