//! Message Intent Classification
//!
//! Classifies WHAT the user is trying to DO with their message.
//! This is the first-level classification before content analysis.

use serde::{Deserialize, Serialize};

/// The primary intent of the user's message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageIntent {
    /// Recording something - events, states, observations (current default)
    Log,
    /// Asking a question - wants information back
    Query,
    /// Requesting system to make connections or derive insights
    Infer,
    /// Looking up specific entities, threads, or past entries
    Reference,
    /// Requesting analysis, patterns, or reflection on data
    Reflect,
    /// Giving a command/instruction to the system
    Command,
    /// Conversational/social - greetings, acknowledgments
    Social,
}

impl MessageIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Log => "LOG",
            Self::Query => "QUERY",
            Self::Infer => "INFER",
            Self::Reference => "REFERENCE",
            Self::Reflect => "REFLECT",
            Self::Command => "COMMAND",
            Self::Social => "SOCIAL",
        }
    }
    
    /// Does this intent expect data to be returned?
    pub fn expects_response(&self) -> bool {
        matches!(self, Self::Query | Self::Infer | Self::Reference | Self::Reflect)
    }
    
    /// Does this intent involve storing new data?
    pub fn stores_data(&self) -> bool {
        matches!(self, Self::Log)
    }
}

impl Default for MessageIntent {
    fn default() -> Self {
        Self::Log
    }
}

impl std::fmt::Display for MessageIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The type of query (when MessageIntent is QUERY)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryType {
    /// When did X happen? Time-based lookup
    Temporal,
    /// Who/what questions - entity lookup
    Entity,
    /// How often? Pattern/frequency questions
    Pattern,
    /// Compare X and Y
    Comparison,
    /// Give me a summary/overview
    Summary,
    /// Count/quantity questions (how many?)
    Quantitative,
    /// Yes/no questions (did X happen?)
    Boolean,
    /// Why questions - causation
    Causal,
    /// Location-based questions
    Spatial,
    /// Open-ended/exploratory
    Exploratory,
}

impl QueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Temporal => "TEMPORAL",
            Self::Entity => "ENTITY",
            Self::Pattern => "PATTERN",
            Self::Comparison => "COMPARISON",
            Self::Summary => "SUMMARY",
            Self::Quantitative => "QUANTITATIVE",
            Self::Boolean => "BOOLEAN",
            Self::Causal => "CAUSAL",
            Self::Spatial => "SPATIAL",
            Self::Exploratory => "EXPLORATORY",
        }
    }
}

impl std::fmt::Display for QueryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Query target - what data is the query looking for?
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QueryTarget {
    /// The entity/thread types being queried (e.g., "moments", "threads", "pulses")
    #[serde(default)]
    pub entity_types: Vec<String>,
    /// Specific thread hints (names, concepts) to search for
    #[serde(default)]
    pub thread_hints: Vec<String>,
    /// Temporal scope (e.g., "today", "last week", "all time")
    #[serde(default)]
    pub temporal_scope: Option<String>,
    /// Keywords extracted from the query
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl Default for QueryTarget {
    fn default() -> Self {
        Self {
            entity_types: vec![],
            thread_hints: vec![],
            temporal_scope: None,
            keywords: vec![],
        }
    }
}

/// Complete message classification (intent + optional details)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MessageClassification {
    /// Primary intent of the message
    pub intent: MessageIntent,
    /// Confidence in the classification (0.0 to 1.0)
    pub confidence: f64,
    /// If QUERY, what type of query
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_type: Option<QueryType>,
    /// If QUERY, what data is being requested
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_target: Option<QueryTarget>,
    /// Alternative intents that might also apply
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_intents: Vec<(MessageIntent, f64)>,
}

impl MessageClassification {
    pub fn log() -> Self {
        Self {
            intent: MessageIntent::Log,
            confidence: 1.0,
            query_type: None,
            query_target: None,
            secondary_intents: vec![],
        }
    }
    
    pub fn query(query_type: QueryType) -> Self {
        Self {
            intent: MessageIntent::Query,
            confidence: 1.0,
            query_type: Some(query_type),
            query_target: None,
            secondary_intents: vec![],
        }
    }
    
    pub fn with_target(mut self, target: QueryTarget) -> Self {
        self.query_target = Some(target);
        self
    }
    
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

