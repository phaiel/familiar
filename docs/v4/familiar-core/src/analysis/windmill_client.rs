//! Windmill Flow Integration for Schema Analysis
//!
//! Calls Windmill flows for complex LLM-powered analysis tasks.
//! Uses the centralized `u/phaiel/anthropic_analyzer` resource.
//!
//! Enable with: `cargo build --features windmill`

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Request to the schema analyzer flow
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AnalysisRequest {
    /// Type of analysis to perform
    pub analysis_type: AnalysisType,
    /// The struct/code to analyze
    pub code: String,
    /// Name of the struct/type
    pub name: String,
    /// File path for context
    pub file_path: String,
    /// Additional context (e.g., project conventions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    /// Should this struct use EntityMeta/SystemEntityMeta?
    EntityClassification,
    /// Generate migration code for a struct
    GenerateMigration,
    /// Prioritize a list of issues
    PrioritizeFixes,
    /// General schema compliance check
    ComplianceCheck,
}

/// Response from the schema analyzer flow
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct AnalysisResponse {
    /// Classification result
    pub classification: Option<String>,
    /// Should this be migrated?
    pub should_migrate: bool,
    /// Priority 1-5 (5 = highest)
    pub priority: u8,
    /// Explanation of the analysis
    pub reasoning: String,
    /// Suggested code fix
    pub suggested_fix: Option<String>,
    /// Files that would need changes
    pub dependencies: Vec<String>,
    /// Any errors during analysis
    pub error: Option<String>,
}

impl Default for AnalysisResponse {
    fn default() -> Self {
        Self {
            classification: None,
            should_migrate: false,
            priority: 0,
            reasoning: String::new(),
            suggested_fix: None,
            dependencies: vec![],
            error: None,
        }
    }
}

/// Client for calling Windmill flows
pub struct WindmillClient {
    client: Client,
    base_url: String,
    workspace: String,
    token: String,
}

impl WindmillClient {
    /// Create a new Windmill client
    ///
    /// Reads config from config.toml with env var overrides:
    /// - WINDMILL_URL (default: http://localhost:8000)
    /// - WINDMILL_WORKSPACE (default: familiar)
    /// - WINDMILL_TOKEN (required)
    pub fn from_env() -> Option<Self> {
        // Try to load from config.toml first
        let config = crate::runtime_config::CoreRuntimeConfig::load().ok();
        
        let token = config.as_ref()
            .map(|c| c.windmill.token.clone())
            .or_else(|| std::env::var("WINDMILL_TOKEN").ok())?;
        
        let base_url = config.as_ref()
            .map(|c| c.windmill.url.clone())
            .or_else(|| std::env::var("WINDMILL_URL").ok())
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        
        let workspace = config.as_ref()
            .map(|c| c.windmill.workspace.clone())
            .or_else(|| std::env::var("WINDMILL_WORKSPACE").ok())
            .unwrap_or_else(|| "familiar".to_string());
        
        Some(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .ok()?,
            base_url,
            workspace,
            token,
        })
    }

    /// Create with explicit config
    pub fn new(base_url: &str, workspace: &str, token: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            workspace: workspace.to_string(),
            token: token.to_string(),
        }
    }

    /// Analyze whether a struct should use EntityMeta
    pub async fn analyze_entity(&self, name: &str, code: &str, file_path: &PathBuf) -> AnalysisResponse {
        let request = AnalysisRequest {
            analysis_type: AnalysisType::EntityClassification,
            code: code.to_string(),
            name: name.to_string(),
            file_path: file_path.display().to_string(),
            context: Some(FAMILIAR_CONVENTIONS.to_string()),
        };

        self.run_flow("f/familiar/analyzer/schema", &request).await
    }

    /// Generate migration code for a struct
    pub async fn generate_migration(&self, name: &str, code: &str, target_pattern: &str) -> AnalysisResponse {
        let request = AnalysisRequest {
            analysis_type: AnalysisType::GenerateMigration,
            code: code.to_string(),
            name: name.to_string(),
            file_path: String::new(),
            context: Some(format!("Target pattern: {}\n\n{}", target_pattern, FAMILIAR_CONVENTIONS)),
        };

        self.run_flow("f/familiar/analyzer/schema", &request).await
    }

    /// Run a Windmill flow and return the result
    async fn run_flow(&self, flow_path: &str, request: &AnalysisRequest) -> AnalysisResponse {
        let url = format!(
            "{}/api/w/{}/jobs/run_wait_result/f/{}",
            self.base_url, self.workspace, flow_path
        );

        match self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    response.json().await.unwrap_or_else(|e| AnalysisResponse {
                        error: Some(format!("Failed to parse response: {}", e)),
                        ..Default::default()
                    })
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    AnalysisResponse {
                        error: Some(format!("Windmill error {}: {}", status, body)),
                        ..Default::default()
                    }
                }
            }
            Err(e) => AnalysisResponse {
                error: Some(format!("Request failed: {}", e)),
                ..Default::default()
            },
        }
    }
}

/// Familiar project schema conventions for LLM context
const FAMILIAR_CONVENTIONS: &str = r#"
# Familiar Schema Conventions

## Entity Types

1. **Domain Entities** (most types): Use `EntityMeta<{Type}Id>` with `#[serde(flatten)]`
   - Has: id, tenant_id, created_at, updated_at
   - Example: Channel, Message, FamiliarEntity

2. **System Entities** (User, Tenant): Use `SystemEntityMeta<{Type}Id>`
   - Has: id, created_at, updated_at (NO tenant_id)
   - Only for: User, Tenant

3. **DTOs/Inputs** (CreateXInput, UpdateXInput): Do NOT use EntityMeta
   - These are request payloads, not stored entities

4. **UI Types** (UIChannel, UIMessage): Do NOT use EntityMeta
   - These are frontend DTOs with different field types (String dates)

5. **Components** (Timestamps, EntityPhysics): Do NOT use EntityMeta
   - These are composable pieces, not standalone entities

## Semantic Primitives

- Use `UserId`, `TenantId`, `ChannelId` instead of raw `Uuid`
- Create new primitives with `define_uuid_id!` macro
- Comment raw Uuid with intended type: `pub id: Uuid,  // FooId`
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_request_serialization() {
        let request = AnalysisRequest {
            analysis_type: AnalysisType::EntityClassification,
            code: "pub struct Foo {}".to_string(),
            name: "Foo".to_string(),
            file_path: "src/types/foo.rs".to_string(),
            context: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("entity_classification"));
    }
}

