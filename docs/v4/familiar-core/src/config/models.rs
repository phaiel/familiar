//! AI Model Configuration
//!
//! Schema-driven model definitions. Update model IDs here when providers change them.
//! This is the single source of truth for model configurations.

use serde::{Deserialize, Serialize};

/// AI Provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Google,
    Mock,
}

impl AIProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::Mock => "mock",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Google => "Google",
            Self::Mock => "Mock",
        }
    }

    pub fn api_key_env_var(&self) -> Option<&'static str> {
        match self {
            Self::OpenAI => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::Google => Some("GOOGLE_API_KEY"),
            Self::Mock => None,
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com/v1",
            Self::Google => "https://generativelanguage.googleapis.com/v1beta",
            Self::Mock => "http://localhost:0",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, Self::Mock)
    }
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Model definition with all configuration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModelConfig {
    /// Unique identifier (e.g., "gpt-4o", "claude-sonnet-4")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// The provider
    pub provider: AIProvider,
    /// The actual API model ID to send to the provider
    pub api_model_id: String,
    /// Context window size (tokens)
    pub context_window: u32,
    /// Whether the model supports JSON mode
    pub supports_json_mode: bool,
    /// Whether this model is deprecated
    #[serde(default)]
    pub deprecated: bool,
    /// Notes about the model
    #[serde(default)]
    pub notes: Option<String>,
}

impl ModelConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: AIProvider,
        api_model_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider,
            api_model_id: api_model_id.into(),
            context_window: 128_000,
            supports_json_mode: true,
            deprecated: false,
            notes: None,
        }
    }

    pub fn with_context_window(mut self, size: u32) -> Self {
        self.context_window = size;
        self
    }

    pub fn with_json_mode(mut self, supported: bool) -> Self {
        self.supports_json_mode = supported;
        self
    }

    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Get all available models
/// 
/// UPDATE THIS FUNCTION when model IDs change!
/// Last updated: December 2025
pub fn get_available_models() -> Vec<ModelConfig> {
    vec![
        // =====================================================================
        // OpenAI Models (December 2025)
        // GPT-5 released, GPT-4 series still available
        // =====================================================================
        ModelConfig::new("gpt-5", "GPT-5", AIProvider::OpenAI, "gpt-5")
            .with_context_window(256_000)
            .with_notes("Latest flagship model - Dec 2025"),
        
        ModelConfig::new("gpt-5-mini", "GPT-5 Mini", AIProvider::OpenAI, "gpt-5-mini")
            .with_context_window(256_000)
            .with_notes("Fast GPT-5 variant"),
        
        ModelConfig::new("gpt-4o", "GPT-4o", AIProvider::OpenAI, "gpt-4o")
            .with_context_window(128_000)
            .with_notes("Previous gen, still excellent"),
        
        ModelConfig::new("gpt-4o-mini", "GPT-4o Mini", AIProvider::OpenAI, "gpt-4o-mini")
            .with_context_window(128_000)
            .with_notes("Fast and cheap"),
        
        ModelConfig::new("o1", "o1 (Reasoning)", AIProvider::OpenAI, "o1")
            .with_context_window(200_000)
            .with_notes("Advanced reasoning model"),
        
        ModelConfig::new("o1-mini", "o1 Mini", AIProvider::OpenAI, "o1-mini")
            .with_context_window(128_000)
            .with_notes("Smaller reasoning model"),

        // =====================================================================
        // Anthropic Models (December 2025)
        // Claude 4 series with Haiku 4.5 (fast), Sonnet 4, Opus 4
        // =====================================================================
        ModelConfig::new("claude-haiku-4.5", "Claude Haiku 4.5", AIProvider::Anthropic, "claude-4-5-haiku-20251201")
            .with_context_window(200_000)
            .with_json_mode(false)
            .with_notes("Fast & cheap - Dec 2025"),
        
        ModelConfig::new("claude-sonnet-4", "Claude Sonnet 4", AIProvider::Anthropic, "claude-sonnet-4-20250514")
            .with_context_window(200_000)
            .with_json_mode(false)
            .with_notes("Balanced performance"),
        
        ModelConfig::new("claude-opus-4", "Claude Opus 4", AIProvider::Anthropic, "claude-opus-4-20250514")
            .with_context_window(200_000)
            .with_json_mode(false)
            .with_notes("Most capable Claude"),
        
        ModelConfig::new("claude-haiku-3.5", "Claude Haiku 3.5", AIProvider::Anthropic, "claude-3-5-haiku-20241022")
            .with_context_window(200_000)
            .with_json_mode(false)
            .deprecated()
            .with_notes("Legacy - use Haiku 4.5"),

        // =====================================================================
        // Google Models (December 2025)
        // Gemini 3 released with massive context
        // =====================================================================
        ModelConfig::new("gemini-3-ultra", "Gemini 3 Ultra", AIProvider::Google, "gemini-3.0-ultra")
            .with_context_window(2_000_000)
            .with_notes("Flagship Gemini 3 - Dec 2025"),
        
        ModelConfig::new("gemini-3-pro", "Gemini 3 Pro", AIProvider::Google, "gemini-3.0-pro")
            .with_context_window(2_000_000)
            .with_notes("Balanced Gemini 3"),
        
        ModelConfig::new("gemini-3-flash", "Gemini 3 Flash", AIProvider::Google, "gemini-3.0-flash")
            .with_context_window(1_000_000)
            .with_notes("Fast Gemini 3"),
        
        ModelConfig::new("gemini-2.0-flash", "Gemini 2.0 Flash", AIProvider::Google, "gemini-2.0-flash")
            .with_context_window(1_000_000)
            .with_notes("Previous gen - still good"),
        
        ModelConfig::new("gemini-1.5-pro", "Gemini 1.5 Pro", AIProvider::Google, "gemini-1.5-pro")
            .with_context_window(2_000_000)
            .deprecated()
            .with_notes("Legacy - use Gemini 3"),

        // =====================================================================
        // Mock (No API Key Required)
        // =====================================================================
        ModelConfig::new("mock", "Mock (No API Key)", AIProvider::Mock, "mock")
            .with_context_window(u32::MAX)
            .with_notes("Heuristic-based, no API calls"),
    ]
}

/// Find a model by ID
pub fn find_model(id: &str) -> Option<ModelConfig> {
    get_available_models().into_iter().find(|m| m.id == id)
}

/// Get models for a specific provider
pub fn get_models_for_provider(provider: AIProvider) -> Vec<ModelConfig> {
    get_available_models()
        .into_iter()
        .filter(|m| m.provider == provider && !m.deprecated)
        .collect()
}

/// Get the default model for a provider
pub fn get_default_model(provider: AIProvider) -> Option<ModelConfig> {
    get_models_for_provider(provider).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_models_have_valid_ids() {
        for model in get_available_models() {
            assert!(!model.id.is_empty());
            assert!(!model.api_model_id.is_empty());
            assert!(!model.name.is_empty());
        }
    }

    #[test]
    fn test_find_model() {
        let model = find_model("gpt-4o").unwrap();
        assert_eq!(model.api_model_id, "gpt-4o");
    }

    #[test]
    fn test_anthropic_models_use_latest() {
        let models = get_models_for_provider(AIProvider::Anthropic);
        for model in models {
            // All Anthropic models should use -latest suffix or specific dated version
            assert!(
                model.api_model_id.contains("latest") || model.api_model_id.contains("20"),
                "Model {} should use -latest or dated version",
                model.id
            );
        }
    }
}

