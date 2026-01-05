//! AI Observation request components

use serde::{Deserialize, Serialize};
use crate::primitives::{Temperature, MaxTokens, RetryConfig};
use crate::config::ModelConfig;

/// Configuration for an AI completion request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RequestConfig {
    pub model: ModelConfig,
    pub temperature: Temperature,
    pub max_tokens: MaxTokens,
    pub json_mode: bool,
    pub retry: RetryConfig,
    pub timeout_ms: u64,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::new("mock", "Mock", crate::config::AIProvider::Mock, "mock"),
            temperature: Temperature::CLASSIFICATION,
            max_tokens: MaxTokens::CLASSIFICATION,
            json_mode: true,
            retry: RetryConfig::default(),
            timeout_ms: 30_000,
        }
    }
}

impl RequestConfig {
    pub fn with_model(mut self, model: ModelConfig) -> Self { self.model = model; self }
    pub fn with_temperature(mut self, temp: Temperature) -> Self { self.temperature = temp; self }
    pub fn with_max_tokens(mut self, tokens: MaxTokens) -> Self { self.max_tokens = tokens; self }
    pub fn with_json_mode(mut self, enabled: bool) -> Self { self.json_mode = enabled; self }
    pub fn with_retry(mut self, config: RetryConfig) -> Self { self.retry = config; self }
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self { self.timeout_ms = timeout_ms; self }
}

/// A complete Heddle observation request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ObservationRequest {
    pub context: String,
    pub segment: String,
    pub config: RequestConfig,
}

impl ObservationRequest {
    pub fn new(segment: impl Into<String>) -> Self {
        Self { context: String::new(), segment: segment.into(), config: RequestConfig::default() }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self { self.context = context.into(); self }
    pub fn with_config(mut self, config: RequestConfig) -> Self { self.config = config; self }
}
