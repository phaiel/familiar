//! Prompt Configuration Schema
//!
//! Defines schema for configuring AI prompts used in the system.
//! Actual prompt templates are defined in runtime scripts (TypeScript/Windmill).

use serde::{Deserialize, Serialize};

/// Schema for TypeScript prompt configuration
/// Runtime prompts are in services/windmill/scripts/*.ts
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PromptConfig {
    pub phase: PromptPhase,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum PromptPhase {
    Segmentation,
    Purpose,
    Classification,
}

