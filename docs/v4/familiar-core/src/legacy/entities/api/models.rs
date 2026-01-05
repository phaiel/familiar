//! GET /models endpoint entity
//!
//! Lists available AI models.

use serde::{Deserialize, Serialize};
use crate::config::ModelConfig;

/// Response payload for GET /models
pub type ModelsResponse = Vec<ModelInfo>;

/// Model information component
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_model_id: String,
}

impl From<&ModelConfig> for ModelInfo {
    fn from(config: &ModelConfig) -> Self {
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            provider: config.provider.as_str().to_string(), // lowercase for JS
            api_model_id: config.api_model_id.clone(),
        }
    }
}

impl From<ModelConfig> for ModelInfo {
    fn from(config: ModelConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            provider: config.provider.as_str().to_string(), // lowercase for JS
            api_model_id: config.api_model_id,
        }
    }
}

/// Get all available models as response
pub fn get_models_response() -> ModelsResponse {
    crate::config::get_available_models()
        .into_iter()
        .filter(|m| !m.deprecated)
        .map(ModelInfo::from)
        .collect()
}

