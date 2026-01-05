//! AI Provider message format adapters
//!
//! Generic adapters for converting between the shared ChatMessage/Conversation
//! format and provider-specific API formats.
//!
//! These have Rust-specific From implementations and cannot be generated from schemas.

use serde::{Deserialize, Serialize};
use familiar_contracts::{MessageRole, ChatMessage, Conversation, AnthropicMessage, GoogleContent, GooglePart};

/// Helper to convert MessageRole to string
fn role_to_string(role: &MessageRole) -> String {
    match role {
        MessageRole::User => "user".to_string(),
        MessageRole::Assistant => "assistant".to_string(),
        MessageRole::System => "system".to_string(),
    }
}

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
            role: role_to_string(&msg.role),
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

/// Anthropic conversation (system is separate from messages)
/// This has a From impl and cannot be generated from schema
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

/// Google conversation (system instruction is separate)
/// This has a From impl and cannot be generated from schema
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
                    contents.push(google_content_from_chat_message(msg));
                }
            }
        }

        Self { system_instruction, contents }
    }
}

/// Helper to convert ChatMessage to GoogleContent
fn google_content_from_chat_message(msg: &ChatMessage) -> GoogleContent {
    let role = match msg.role {
        MessageRole::User => Some("user".to_string()),
        MessageRole::Assistant => Some("model".to_string()),
        MessageRole::System => None,
    };
    GoogleContent {
        role,
        parts: vec![GooglePart { text: msg.content.clone() }],
    }
}

