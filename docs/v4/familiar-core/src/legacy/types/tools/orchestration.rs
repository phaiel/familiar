//! Tool Chain Orchestration Schema
//!
//! Defines how tools are composed into processing pipelines.
//! Supports sequential, parallel, and conditional execution.

use serde::{Deserialize, Serialize};

use super::base::ToolCategory;
use super::intent::PurposePipeline;

// ============================================================================
// Tool Chain
// ============================================================================

/// A chain of tools to execute
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolChain {
    /// Chain identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Steps in the chain
    pub steps: Vec<ToolChainStep>,
    /// How context flows between steps
    pub context_strategy: ContextStrategy,
    /// Error handling strategy
    pub error_strategy: ErrorStrategy,
}

/// A step in a tool chain
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolChainStep {
    /// Step identifier
    pub id: String,
    /// Tool to execute
    pub tool: String,
    /// Execution order
    pub execution_order: ExecutionOrder,
    /// Input mapping from previous steps
    #[serde(default)]
    pub input_mapping: Vec<InputMapping>,
    /// Condition for execution (if conditional)
    #[serde(default)]
    pub condition: Option<StepCondition>,
    /// Steps to run in parallel with this one
    #[serde(default)]
    pub parallel_with: Vec<String>,
    /// Timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Whether step is required or optional
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Execution order types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionOrder {
    /// Execute in sequence
    Sequential,
    /// Execute in parallel with other parallel steps
    Parallel,
    /// Execute only if condition is met
    Conditional,
    /// Execute as fallback if previous failed
    Fallback,
}

/// Input mapping from previous step output
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InputMapping {
    /// Source step ID
    pub from_step: String,
    /// Path in source output
    pub from_path: String,
    /// Path in this step's input
    pub to_path: String,
    /// Transform to apply
    #[serde(default)]
    pub transform: Option<InputTransform>,
}

/// Transforms for input mapping
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputTransform {
    /// Direct pass-through
    Identity,
    /// Extract array item
    ArrayItem { index: usize },
    /// Filter array by condition
    ArrayFilter { path: String, value: String },
    /// Map array items
    ArrayMap { template: String },
    /// Concatenate strings
    Concat { separator: String },
    /// Custom transform function
    Custom { function: String },
}

/// Condition for conditional execution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StepCondition {
    /// Type of condition
    pub condition_type: ConditionType,
    /// Source step to check
    pub source_step: String,
    /// Path to check
    pub path: String,
    /// Operator
    pub operator: ConditionOperator,
    /// Value to compare
    pub value: serde_json::Value,
}

/// Types of conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// Check output value
    OutputValue,
    /// Check if step succeeded
    StepSuccess,
    /// Check if step failed
    StepFailed,
    /// Check output exists
    OutputExists,
}

/// Condition operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    NotContains,
    IsNull,
    IsNotNull,
    In,
}

// ============================================================================
// Context Strategy
// ============================================================================

/// How context flows between tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContextStrategy {
    /// Pass full context to every tool
    FullContext,
    /// Only pass relevant context (filtered)
    FilteredContext,
    /// Accumulate results as context grows
    AccumulatingContext,
    /// Each step gets fresh context
    IsolatedContext,
}

/// Error handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    /// Fail the entire chain on any error
    FailFast,
    /// Continue with remaining steps
    ContinueOnError,
    /// Retry failed steps
    RetryOnError,
    /// Use fallback tool
    UseFallback,
}

// ============================================================================
// Standard Pipelines
// ============================================================================

/// Standard processing pipeline definitions
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StandardPipeline {
    /// Pipeline type
    pub pipeline_type: PipelineType,
    /// The tool chain
    pub chain: ToolChain,
    /// When to use this pipeline
    pub triggers: Vec<PipelineTrigger>,
}

/// Types of standard pipelines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PipelineType {
    /// Memory recording: Segment → Classify → Spawn → Hints → Store
    MemoryRecording,
    /// Query: Parse → Search → Format → Return
    Query,
    /// Analysis: Parse → Search → Analyze → Synthesize
    Analysis,
    /// Command: Parse → Validate → Execute → Confirm
    Command,
    /// Conversation: Understand → Respond
    Conversation,
    /// Multi-modal: Route → Process per modality → Merge
    MultiModal,
}

impl PipelineType {
    /// Get the tool categories used in this pipeline
    pub fn tool_categories(&self) -> Vec<ToolCategory> {
        match self {
            Self::MemoryRecording => vec![
                ToolCategory::Segmentation,
                ToolCategory::Classification,
                ToolCategory::Spawn,
                ToolCategory::Hints,
            ],
            Self::Query => vec![
                ToolCategory::Classification,
                ToolCategory::Retrieval,
            ],
            Self::Analysis => vec![
                ToolCategory::Segmentation,
                ToolCategory::Classification,
                ToolCategory::Retrieval,
            ],
            Self::Command => vec![
                ToolCategory::Classification,
                ToolCategory::Orchestration,
            ],
            Self::Conversation => vec![
                ToolCategory::Classification,
            ],
            Self::MultiModal => vec![
                ToolCategory::Segmentation,
                ToolCategory::Classification,
                ToolCategory::Spawn,
                ToolCategory::Hints,
            ],
        }
    }

    /// Convert from PurposePipeline
    pub fn from_purpose_pipeline(purpose: PurposePipeline) -> Self {
        match purpose {
            PurposePipeline::Recording => Self::MemoryRecording,
            PurposePipeline::Retrieval => Self::Query,
            PurposePipeline::Analysis => Self::Analysis,
            PurposePipeline::Action => Self::Command,
            PurposePipeline::Conversational => Self::Conversation,
        }
    }
}

/// Trigger for pipeline selection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PipelineTrigger {
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Value to match
    pub value: String,
    /// Priority (higher = checked first)
    #[serde(default)]
    pub priority: u32,
}

/// Types of triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Match on purpose type
    Purpose,
    /// Match on modality
    Modality,
    /// Match on entity type
    EntityType,
    /// Match on keyword
    Keyword,
    /// Custom condition
    Custom,
}

// ============================================================================
// Pipeline Builder
// ============================================================================

impl ToolChain {
    /// Create a memory recording pipeline
    pub fn memory_recording() -> Self {
        Self {
            id: "memory_recording".to_string(),
            name: "Memory Recording Pipeline".to_string(),
            description: "Segment → Purpose → Classify → Spawn → Hints → Store".to_string(),
            steps: vec![
                ToolChainStep {
                    id: "segment".to_string(),
                    tool: "text_segmentation".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(5000),
                    required: true,
                },
                ToolChainStep {
                    id: "purpose".to_string(),
                    tool: "purpose_classifier".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![InputMapping {
                        from_step: "segment".to_string(),
                        from_path: "segments".to_string(),
                        to_path: "segments".to_string(),
                        transform: None,
                    }],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: true,
                },
                ToolChainStep {
                    id: "classify".to_string(),
                    tool: "entity_classifier".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(5000),
                    required: true,
                },
                ToolChainStep {
                    id: "spawn".to_string(),
                    tool: "spawn_suggester".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: true,
                },
                ToolChainStep {
                    id: "physics_hints".to_string(),
                    tool: "physics_hint_generator".to_string(),
                    execution_order: ExecutionOrder::Parallel,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec!["thread_hints".to_string(), "bond_hints".to_string()],
                    timeout_ms: Some(3000),
                    required: false,
                },
                ToolChainStep {
                    id: "thread_hints".to_string(),
                    tool: "thread_hint_generator".to_string(),
                    execution_order: ExecutionOrder::Parallel,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: false,
                },
                ToolChainStep {
                    id: "bond_hints".to_string(),
                    tool: "bond_hint_generator".to_string(),
                    execution_order: ExecutionOrder::Parallel,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: false,
                },
                ToolChainStep {
                    id: "binding_hints".to_string(),
                    tool: "binding_hint_generator".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: false,
                },
            ],
            context_strategy: ContextStrategy::AccumulatingContext,
            error_strategy: ErrorStrategy::ContinueOnError,
        }
    }

    /// Create a query pipeline
    pub fn query() -> Self {
        Self {
            id: "query".to_string(),
            name: "Query Pipeline".to_string(),
            description: "Purpose → Query Analysis → Search → Format".to_string(),
            steps: vec![
                ToolChainStep {
                    id: "purpose".to_string(),
                    tool: "purpose_classifier".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: true,
                },
                ToolChainStep {
                    id: "query_analysis".to_string(),
                    tool: "query_analyzer".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: true,
                },
                ToolChainStep {
                    id: "search".to_string(),
                    tool: "rag_search".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(10000),
                    required: true,
                },
                ToolChainStep {
                    id: "format".to_string(),
                    tool: "response_formatter".to_string(),
                    execution_order: ExecutionOrder::Sequential,
                    input_mapping: vec![],
                    condition: None,
                    parallel_with: vec![],
                    timeout_ms: Some(3000),
                    required: true,
                },
            ],
            context_strategy: ContextStrategy::FilteredContext,
            error_strategy: ErrorStrategy::FailFast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pipeline() {
        let chain = ToolChain::memory_recording();
        assert!(!chain.steps.is_empty());
        assert!(chain.steps.iter().any(|s| s.id == "segment"));
        assert!(chain.steps.iter().any(|s| s.id == "classify"));
    }

    #[test]
    fn test_pipeline_types() {
        let categories = PipelineType::MemoryRecording.tool_categories();
        assert!(categories.contains(&ToolCategory::Segmentation));
        assert!(categories.contains(&ToolCategory::Classification));
    }
}
