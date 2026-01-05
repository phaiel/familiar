//! Purpose Classification Tool Schema
//!
//! Classifies WHAT the user is trying to DO with their message.
//! This determines the processing pipeline to use.

use serde::{Deserialize, Serialize};

use super::segmentation::Segment;
use crate::types::agentic::ConversationTurn;
use crate::types::{QueryType, QueryTarget};

// ============================================================================
// Purpose Classification Types
// ============================================================================

/// Input for purpose classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PurposeClassifierInput {
    /// The segment to classify
    pub segment: Segment,
    /// Conversation context for better classification
    #[serde(default)]
    pub context: Option<PurposeContext>,
}

/// Context for purpose classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PurposeContext {
    /// Previous messages in conversation
    #[serde(default)]
    pub conversation_history: Vec<ConversationTurn>,
    /// Previous classifications detected
    #[serde(default)]
    pub previous_classifications: Vec<ClassifiedPurpose>,
    /// User's typical patterns (if known)
    #[serde(default)]
    pub user_patterns: Option<UserPurposePatterns>,
}


/// User's historical purpose patterns
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UserPurposePatterns {
    /// Most common purpose
    pub primary_purpose: ToolPurpose,
    /// Distribution of purposes (purpose -> frequency)
    #[serde(default)]
    pub purpose_distribution: std::collections::HashMap<String, f64>,
}

// ============================================================================
// Purpose Types
// ============================================================================

/// Primary purpose of user's message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolPurpose {
    /// Recording memories, events, observations - wants to store
    Log,
    /// Asking a question - wants information returned
    Query,
    /// Making connections, deriving insights - wants system to infer
    Infer,
    /// Looking up specific entities - wants existing data
    Reference,
    /// Requesting analysis or reflection - wants patterns identified
    Reflect,
    /// System command/instruction - wants action taken
    Command,
    /// Social/conversational - greeting, acknowledgment
    Social,
    /// Clarifying or continuing previous message
    Continuation,
    /// Editing or correcting previous input
    Correction,
}

impl ToolPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Log => "LOG",
            Self::Query => "QUERY",
            Self::Infer => "INFER",
            Self::Reference => "REFERENCE",
            Self::Reflect => "REFLECT",
            Self::Command => "COMMAND",
            Self::Social => "SOCIAL",
            Self::Continuation => "CONTINUATION",
            Self::Correction => "CORRECTION",
        }
    }

    /// Does this purpose involve storing new data?
    pub fn stores_data(&self) -> bool {
        matches!(self, Self::Log | Self::Correction)
    }

    /// Does this purpose expect data to be returned?
    pub fn expects_response(&self) -> bool {
        matches!(
            self,
            Self::Query | Self::Infer | Self::Reference | Self::Reflect
        )
    }

    /// Does this purpose require searching existing data?
    pub fn requires_search(&self) -> bool {
        matches!(self, Self::Query | Self::Reference | Self::Reflect | Self::Infer)
    }

    /// Processing pipeline for this purpose
    pub fn pipeline(&self) -> PurposePipeline {
        match self {
            Self::Log => PurposePipeline::Recording,
            Self::Query | Self::Reference => PurposePipeline::Retrieval,
            Self::Infer | Self::Reflect => PurposePipeline::Analysis,
            Self::Command => PurposePipeline::Action,
            Self::Social | Self::Continuation | Self::Correction => PurposePipeline::Conversational,
        }
    }
}

impl Default for ToolPurpose {
    fn default() -> Self {
        Self::Log
    }
}

/// Processing pipeline category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PurposePipeline {
    /// Memory recording: Segment → Classify → Spawn → Store
    Recording,
    /// Data retrieval: Parse → Search → Format → Return
    Retrieval,
    /// Analysis: Parse → Search → Analyze → Synthesize
    Analysis,
    /// System action: Parse → Validate → Execute → Confirm
    Action,
    /// Conversation: Respond appropriately
    Conversational,
}

// ============================================================================
// Classification Output
// ============================================================================

/// Output from purpose classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PurposeClassifierOutput {
    /// Primary classified purpose
    pub primary: ClassifiedPurpose,
    /// Secondary purposes (if multi-purpose detected)
    #[serde(default)]
    pub secondary: Vec<ClassifiedPurpose>,
    /// Recommended processing pipeline
    pub pipeline: PurposePipeline,
    /// Query analysis (if purpose is Query-related)
    #[serde(default)]
    pub query_analysis: Option<QueryAnalysis>,
    /// Command analysis (if purpose is Command)
    #[serde(default)]
    pub command_analysis: Option<CommandAnalysis>,
}

/// A classified purpose with confidence
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassifiedPurpose {
    /// The purpose of the message
    pub purpose: ToolPurpose,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Evidence/reasoning for classification
    #[serde(default)]
    pub evidence: Vec<String>,
}

impl ClassifiedPurpose {
    pub fn new(purpose: ToolPurpose, confidence: f64) -> Self {
        Self {
            purpose,
            confidence,
            evidence: vec![],
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }
}

// ============================================================================
// Query Analysis
// ============================================================================

/// Analysis of a query purpose
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QueryAnalysis {
    /// Type of query
    pub query_type: QueryType,
    /// What the query is targeting
    pub target: QueryTarget,
    /// Extracted search terms
    pub search_terms: Vec<String>,
    /// Filters to apply
    #[serde(default)]
    pub filters: Vec<QueryFilter>,
}


/// Temporal scope for queries
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TemporalScope {
    /// Start of range (ISO 8601 or relative)
    #[serde(default)]
    pub start: Option<String>,
    /// End of range
    #[serde(default)]
    pub end: Option<String>,
    /// Relative scope (today, this_week, all_time)
    #[serde(default)]
    pub relative: Option<String>,
}

/// A filter for queries
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QueryFilter {
    /// Field to filter on
    pub field: String,
    /// Filter operator
    pub operator: FilterOperator,
    /// Filter value
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    Between,
    In,
    NotIn,
}

// ============================================================================
// Command Analysis
// ============================================================================

/// Analysis of a command purpose
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CommandAnalysis {
    /// Command type
    pub command_type: CommandType,
    /// Target of the command
    #[serde(default)]
    pub target: Option<String>,
    /// Command parameters
    #[serde(default)]
    pub parameters: serde_json::Map<String, serde_json::Value>,
    /// Is this command destructive?
    pub is_destructive: bool,
    /// Does this command need confirmation?
    pub needs_confirmation: bool,
}

/// Types of commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandType {
    /// Create something
    Create,
    /// Update/modify
    Update,
    /// Delete/remove
    Delete,
    /// Link/connect entities
    Link,
    /// Unlink/disconnect entities
    Unlink,
    /// Export data
    Export,
    /// Import data
    Import,
    /// Configure settings
    Configure,
    /// Unknown/custom command
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purpose_pipeline() {
        assert_eq!(ToolPurpose::Log.pipeline(), PurposePipeline::Recording);
        assert_eq!(ToolPurpose::Query.pipeline(), PurposePipeline::Retrieval);
        assert_eq!(ToolPurpose::Reflect.pipeline(), PurposePipeline::Analysis);
    }

    #[test]
    fn test_classified_purpose() {
        let classification = ClassifiedPurpose::new(ToolPurpose::Query, 0.9)
            .with_evidence(vec!["Contains question mark".to_string()]);

        assert_eq!(classification.purpose, ToolPurpose::Query);
        assert_eq!(classification.confidence, 0.9);
        assert!(!classification.evidence.is_empty());
    }
}
