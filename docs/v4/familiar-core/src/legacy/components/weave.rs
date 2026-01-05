//! Weave Component
//!
//! A Weave represents the raw user input - the message they send to Familiar.

use serde::{Deserialize, Serialize};

/// A Weave is the raw user input - their message to Familiar
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Weave {
    /// The raw text content from the user
    pub raw_content: String,
    
    /// Optional context provided with the message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl Weave {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            raw_content: content.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Get the content length
    pub fn len(&self) -> usize {
        self.raw_content.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.raw_content.is_empty()
    }
}

impl Default for Weave {
    fn default() -> Self {
        Self {
            raw_content: String::new(),
            context: None,
        }
    }
}

impl From<&str> for Weave {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Weave {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for Weave {
    fn from(s: &String) -> Self {
        Self::new(s)
    }
}

