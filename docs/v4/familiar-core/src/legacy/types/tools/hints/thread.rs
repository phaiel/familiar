//! Thread Hint Tool Schema
//!
//! Identifies narrative threads, subjects, and connections.
//! Threads are the persistent topics/people/concepts that tie memories together.

use serde::{Deserialize, Serialize};

use crate::types::tools::entity::{EntityType, ThreadReference};
use crate::types::tools::segmentation::Segment;
use crate::types::tools::spawn::ThreadRole;

// ============================================================================
// Thread Hint Tool
// ============================================================================

/// Input for thread hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadHintInput {
    /// The segment to analyze
    pub segment: Segment,
    /// Entity type being spawned
    pub entity_type: EntityType,
    /// Known threads for matching
    #[serde(default)]
    pub known_threads: Vec<ThreadReference>,
    /// Recent threads for context
    #[serde(default)]
    pub recent_threads: Vec<RecentThread>,
}

/// A recently referenced thread
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecentThread {
    /// Thread reference
    pub thread: ThreadReference,
    /// How recently mentioned (in messages)
    pub recency: u32,
    /// Mention count in recent context
    pub mention_count: u32,
}

/// Output from thread hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadHintOutput {
    /// Thread hints
    pub hints: ThreadHintValues,
    /// Detected thread references
    pub detected_threads: Vec<DetectedThread>,
    /// Suggested new threads to create
    #[serde(default)]
    pub suggested_threads: Vec<SuggestedThread>,
}

/// Thread hint values
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadHintValues {
    /// Primary subject/actor (who/what is the main focus)
    pub primary_subject: SubjectReference,
    /// Related threads
    #[serde(default)]
    pub related_threads: Vec<ThreadReference>,
    /// Role of entity relative to thread
    pub thread_role: ThreadRole,
    /// Keywords for thread matching
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Thread categories
    #[serde(default)]
    pub categories: Vec<ThreadCategory>,
    /// Whether to create thread if no match found
    #[serde(default)]
    pub create_if_missing: bool,
}

/// Reference to a subject
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SubjectReference {
    /// Name/label of subject
    pub name: String,
    /// Type of subject
    pub subject_type: SubjectType,
    /// Matched thread ID (if resolved)
    #[serde(default)]
    pub thread_id: Option<String>,
    /// Confidence in identification
    pub confidence: f64,
}

/// Types of subjects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubjectType {
    /// The user themselves
    Self_,
    /// Another person
    Person,
    /// A place
    Place,
    /// An organization
    Organization,
    /// A concept/topic
    Concept,
    /// An event
    Event,
    /// An object/thing
    Object,
    /// Unknown/other
    Other,
}


/// Categories for threads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadCategory {
    /// People (friends, family, colleagues)
    People,
    /// Places (home, work, cities)
    Places,
    /// Activities (hobbies, work tasks)
    Activities,
    /// Projects (ongoing endeavors)
    Projects,
    /// Topics (interests, subjects)
    Topics,
    /// Events (recurring or one-time)
    Events,
    /// Groups (teams, organizations)
    Groups,
    /// Health (physical, mental)
    Health,
    /// Finance (money, investments)
    Finance,
    /// Relationships
    Relationships,
}

// ============================================================================
// Thread Detection
// ============================================================================

/// A detected thread reference
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedThread {
    /// The matched thread
    pub thread: ThreadReference,
    /// Confidence in match
    pub confidence: f64,
    /// How it was detected
    pub detection_method: DetectionMethod,
    /// Text that triggered detection
    pub trigger_text: String,
    /// Role in this entity
    pub role: ThreadRole,
}

/// How a thread was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    /// Exact name match
    ExactMatch,
    /// Alias match
    AliasMatch,
    /// Fuzzy/similar match
    FuzzyMatch,
    /// Pronoun resolution from context
    PronounResolution,
    /// Keyword match
    KeywordMatch,
    /// Semantic similarity
    SemanticMatch,
}

/// A suggested new thread to create
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SuggestedThread {
    /// Suggested name
    pub name: String,
    /// Type of thread
    pub thread_type: SubjectType,
    /// Category
    pub category: ThreadCategory,
    /// Why this thread should be created
    pub reason: String,
    /// Confidence in suggestion
    pub confidence: f64,
    /// Aliases to register
    #[serde(default)]
    pub aliases: Vec<String>,
}

// ============================================================================
// Thread Graph
// ============================================================================

/// Thread relationship in the graph
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadRelation {
    /// Source thread
    pub source_id: String,
    /// Target thread
    pub target_id: String,
    /// Type of relation
    pub relation_type: ThreadRelationType,
    /// Strength (0.0 to 1.0)
    pub strength: f64,
}

/// Types of thread relations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadRelationType {
    /// Part of (belongs to)
    PartOf,
    /// Contains (has parts)
    Contains,
    /// Related to (general association)
    RelatedTo,
    /// Causes / leads to
    CausesOf,
    /// Opposite of
    OppositeOf,
    /// Same as (alias)
    SameAs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_types() {
        let subject = SubjectReference {
            name: "user".to_string(),
            subject_type: SubjectType::Self_,
            thread_id: None,
            confidence: 1.0,
        };

        assert_eq!(subject.subject_type, SubjectType::Self_);
    }
}
