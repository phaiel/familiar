//! Course Configuration
//!
//! Configuration for Course context management and token limits.
//! Controls how much history is included when building LLM context.

use serde::{Deserialize, Serialize};

/// Configuration for Course context management
/// 
/// These values control how history is pruned when building LLM context.
/// The goal is to maximize useful context while staying within model limits.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseConfig {
    /// Maximum tokens for context window (default: 8000 for Claude Sonnet)
    /// This is the budget for history messages, NOT the full model context.
    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: usize,
    
    /// Reserved tokens for response generation (default: 2000)
    /// Subtracted from max_context_tokens to leave room for the model's response.
    #[serde(default = "default_reserved_tokens")]
    pub reserved_tokens: usize,
    
    /// Minimum messages to always include, regardless of token count (default: 2)
    /// Ensures at least the last exchange is included for coherence.
    #[serde(default = "default_min_messages")]
    pub min_messages: usize,
    
    /// Token estimation method (default: "char4")
    /// - "char4": Simple 4 chars = 1 token heuristic
    /// - "tiktoken": Use tiktoken-rs for accurate counting (slower)
    #[serde(default = "default_estimation_method")]
    pub estimation_method: TokenEstimationMethod,
}

fn default_max_context_tokens() -> usize { 8000 }
fn default_reserved_tokens() -> usize { 2000 }
fn default_min_messages() -> usize { 2 }
fn default_estimation_method() -> TokenEstimationMethod { TokenEstimationMethod::Char4 }

/// Method for estimating token count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TokenEstimationMethod {
    /// Simple heuristic: ~4 characters per token
    /// Fast, good for English text, ~80% accurate
    Char4,
    /// Word-based heuristic: ~1.3 tokens per word
    /// Better for mixed content
    WordBased,
    /// Use tiktoken-rs for accurate counting
    /// Most accurate but slower
    Tiktoken,
}

impl Default for CourseConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: default_max_context_tokens(),
            reserved_tokens: default_reserved_tokens(),
            min_messages: default_min_messages(),
            estimation_method: default_estimation_method(),
        }
    }
}

impl CourseConfig {
    /// Get the available token budget for history (max - reserved)
    pub fn available_tokens(&self) -> usize {
        self.max_context_tokens.saturating_sub(self.reserved_tokens)
    }
    
    /// Create a config for large context models (e.g., Claude Opus, GPT-4 Turbo)
    pub fn large_context() -> Self {
        Self {
            max_context_tokens: 32000,
            reserved_tokens: 4000,
            min_messages: 4,
            estimation_method: TokenEstimationMethod::Char4,
        }
    }
    
    /// Create a config for small/fast models (e.g., Claude Haiku, GPT-4o Mini)
    pub fn small_context() -> Self {
        Self {
            max_context_tokens: 4000,
            reserved_tokens: 1000,
            min_messages: 2,
            estimation_method: TokenEstimationMethod::Char4,
        }
    }
}

// ============================================================================
// Token Estimation Functions
// ============================================================================

/// Estimate token count using the simple char/4 heuristic
/// 
/// This is fast and reasonably accurate for English text.
/// For mixed content (code, special chars), accuracy is ~70-80%.
pub fn estimate_tokens_char4(text: &str) -> usize {
    let char_count = text.chars().count();
    (char_count + 3) / 4  // Round up
}

/// Estimate token count using word-based heuristic
/// 
/// Average is ~1.3 tokens per word for English.
/// Better than char4 for mixed content.
pub fn estimate_tokens_word_based(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    let punctuation_count = text.chars().filter(|c| c.is_ascii_punctuation()).count();
    
    // Words contribute ~1.3 tokens, punctuation ~0.5 tokens
    ((word_count as f64 * 1.3) + (punctuation_count as f64 * 0.5)).ceil() as usize
}

/// Estimate tokens using the configured method
pub fn estimate_tokens(text: &str, method: TokenEstimationMethod) -> usize {
    match method {
        TokenEstimationMethod::Char4 => estimate_tokens_char4(text),
        TokenEstimationMethod::WordBased => estimate_tokens_word_based(text),
        TokenEstimationMethod::Tiktoken => {
            // For now, fall back to char4. In production, integrate tiktoken-rs:
            // let bpe = tiktoken_rs::cl100k_base().unwrap();
            // bpe.encode_with_special_tokens(text).len()
            estimate_tokens_char4(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CourseConfig::default();
        assert_eq!(config.max_context_tokens, 8000);
        assert_eq!(config.reserved_tokens, 2000);
        assert_eq!(config.available_tokens(), 6000);
    }

    #[test]
    fn test_char4_estimation() {
        // "Hello world" = 11 chars -> 3 tokens
        assert_eq!(estimate_tokens_char4("Hello world"), 3);
        
        // Empty string = 0 tokens
        assert_eq!(estimate_tokens_char4(""), 0);
        
        // Single char = 1 token
        assert_eq!(estimate_tokens_char4("a"), 1);
    }

    #[test]
    fn test_word_based_estimation() {
        // "Hello world" = 2 words -> ~3 tokens
        let tokens = estimate_tokens_word_based("Hello world");
        assert!(tokens >= 2 && tokens <= 4);
        
        // "Hello, world!" = 2 words + 2 punct -> ~4 tokens
        let tokens = estimate_tokens_word_based("Hello, world!");
        assert!(tokens >= 3 && tokens <= 5);
    }

    #[test]
    fn test_large_context_config() {
        let config = CourseConfig::large_context();
        assert_eq!(config.max_context_tokens, 32000);
        assert_eq!(config.available_tokens(), 28000);
    }

    #[test]
    fn test_small_context_config() {
        let config = CourseConfig::small_context();
        assert_eq!(config.max_context_tokens, 4000);
        assert_eq!(config.available_tokens(), 3000);
    }
}







