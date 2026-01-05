//! AI Provider message format adapters
//!
//! Generic adapters for converting between the shared ChatMessage/Conversation
//! format and provider-specific API formats.
//!
//! These can be reused by any system that integrates with AI providers.

use serde::{Deserialize, Serialize};
use crate::types::MessageRole;
use crate::components::{ChatMessage, Conversation};

// ============================================================================
// OpenAI Format
// ============================================================================

/// OpenAI message format
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

impl From<&ChatMessage> for OpenAIMessage {
    fn from(msg: &ChatMessage) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }
}

/// Convert a Conversation to OpenAI format
pub fn to_openai_messages(conv: &Conversation) -> Vec<OpenAIMessage> {
    conv.messages.iter().map(OpenAIMessage::from).collect()
}

// ============================================================================
// Anthropic Format
// ============================================================================

/// Anthropic message format (no system role in messages)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

/// Anthropic conversation (system is separate from messages)
#[derive(Debug, Clone)]
pub struct AnthropicConversation {
    pub system: Option<String>,
    pub messages: Vec<AnthropicMessage>,
}

impl From<&Conversation> for AnthropicConversation {
    fn from(conv: &Conversation) -> Self {
        let mut system = None;
        let mut messages = Vec::new();

        for msg in &conv.messages {
            match msg.role {
                MessageRole::System => {
                    system = Some(msg.content.clone());
                }
                MessageRole::User => {
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        Self { system, messages }
    }
}

// ============================================================================
// Google (Gemini) Format
// ============================================================================

/// Google content part
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GooglePart {
    pub text: String,
}

/// Google content (message)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GoogleContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<GooglePart>,
}

impl From<&ChatMessage> for GoogleContent {
    fn from(msg: &ChatMessage) -> Self {
        let role = match msg.role {
            MessageRole::User => Some("user".to_string()),
            MessageRole::Assistant => Some("model".to_string()),
            MessageRole::System => None,
        };
        Self {
            role,
            parts: vec![GooglePart { text: msg.content.clone() }],
        }
    }
}

/// Google conversation (system instruction is separate)
#[derive(Debug, Clone)]
pub struct GoogleConversation {
    pub system_instruction: Option<GoogleContent>,
    pub contents: Vec<GoogleContent>,
}

impl From<&Conversation> for GoogleConversation {
    fn from(conv: &Conversation) -> Self {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in &conv.messages {
            match msg.role {
                MessageRole::System => {
                    system_instruction = Some(GoogleContent {
                        role: None,
                        parts: vec![GooglePart { text: msg.content.clone() }],
                    });
                }
                _ => {
                    contents.push(GoogleContent::from(msg));
                }
            }
        }

        Self { system_instruction, contents }
    }
}

