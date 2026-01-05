//! AI Provider configuration component

use serde::{Deserialize, Serialize};
use crate::primitives::ApiKey;
use crate::config::AIProvider;

/// Configuration for an AI provider connection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProviderConfig {
    pub provider: AIProvider,
    #[serde(skip_serializing)]
    pub api_key: Option<ApiKey>,
    pub base_url: Option<String>,
    pub organization_id: Option<String>,
    pub api_version: Option<String>,
}

impl ProviderConfig {
    pub fn openai(api_key: ApiKey) -> Self {
        Self { provider: AIProvider::OpenAI, api_key: Some(api_key), base_url: None, organization_id: None, api_version: None }
    }

    pub fn anthropic(api_key: ApiKey) -> Self {
        Self { provider: AIProvider::Anthropic, api_key: Some(api_key), base_url: None, organization_id: None, api_version: Some("2023-06-01".to_string()) }
    }

    pub fn google(api_key: ApiKey) -> Self {
        Self { provider: AIProvider::Google, api_key: Some(api_key), base_url: None, organization_id: None, api_version: None }
    }

    pub fn mock() -> Self {
        Self { provider: AIProvider::Mock, api_key: None, base_url: None, organization_id: None, api_version: None }
    }


    pub fn with_api_key(mut self, key: ApiKey) -> Self { self.api_key = Some(key); self }
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self { self.base_url = Some(url.into()); self }
    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self { self.organization_id = Some(org_id.into()); self }

    pub fn effective_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or_else(|| self.provider.default_base_url())
    }
}

impl Default for ProviderConfig {
    fn default() -> Self { Self::mock() }
}

