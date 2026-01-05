//! Observer error types for AI provider interactions

use serde::{Deserialize, Serialize};
use crate::config::AIProvider;

/// Errors that can occur during AI observation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "details")]
pub enum ObserverError {
    /// Configuration error (missing API key, invalid model, etc.)
    Configuration { message: String },
    /// The AI provider returned an invalid response
    InvalidResponse {
        provider: AIProvider,
        message: String,
        raw_response: Option<String>,
    },
    /// Network or API error
    Network {
        provider: AIProvider,
        message: String,
        status_code: Option<u16>,
    },
    /// Rate limit exceeded
    RateLimited {
        provider: AIProvider,
        retry_after_ms: Option<u64>,
    },
    /// The AI refused to process (content policy)
    ContentFiltered {
        provider: AIProvider,
        message: String,
    },
    /// Deserialization failed
    ParseError {
        message: String,
        raw_content: Option<String>,
    },
    /// Timeout
    Timeout {
        provider: AIProvider,
        timeout_ms: u64,
    },
}

impl ObserverError {
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration { message: message.into() }
    }

    pub fn network(provider: AIProvider, message: impl Into<String>, status_code: Option<u16>) -> Self {
        Self::Network { provider, message: message.into(), status_code }
    }

    pub fn parse_error(message: impl Into<String>, raw: Option<String>) -> Self {
        Self::ParseError { message: message.into(), raw_content: raw }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network { .. } | Self::RateLimited { .. } | Self::Timeout { .. })
    }

    pub fn provider(&self) -> Option<AIProvider> {
        match self {
            Self::Configuration { .. } | Self::ParseError { .. } => None,
            Self::InvalidResponse { provider, .. }
            | Self::Network { provider, .. }
            | Self::RateLimited { provider, .. }
            | Self::ContentFiltered { provider, .. }
            | Self::Timeout { provider, .. } => Some(*provider),
        }
    }
}

impl std::fmt::Display for ObserverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration { message } => write!(f, "Configuration error: {}", message),
            Self::InvalidResponse { provider, message, .. } => write!(f, "{} invalid response: {}", provider, message),
            Self::Network { provider, message, status_code } => {
                if let Some(code) = status_code {
                    write!(f, "{} network error (HTTP {}): {}", provider, code, message)
                } else {
                    write!(f, "{} network error: {}", provider, message)
                }
            }
            Self::RateLimited { provider, retry_after_ms } => {
                if let Some(ms) = retry_after_ms {
                    write!(f, "{} rate limited, retry after {}ms", provider, ms)
                } else {
                    write!(f, "{} rate limited", provider)
                }
            }
            Self::ContentFiltered { provider, message } => write!(f, "{} content filtered: {}", provider, message),
            Self::ParseError { message, .. } => write!(f, "Parse error: {}", message),
            Self::Timeout { provider, timeout_ms } => write!(f, "{} timeout after {}ms", provider, timeout_ms),
        }
    }
}

impl std::error::Error for ObserverError {}

