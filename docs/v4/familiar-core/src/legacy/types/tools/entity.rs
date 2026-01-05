//! Entity Classification Tool Schema
//!
//! Probabilistic multi-label classification of segments into entity types.
//! Maps content to the Symmetric Seven ontology.

use serde::{Deserialize, Serialize};

use super::intent::ClassifiedPurpose;
use super::segmentation::Segment;

// ============================================================================
// Entity Classification Input/Output
// ============================================================================

/// Input for entity classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityClassifierInput {
    /// The segment to classify
    pub segment: Segment,
    /// Purpose classification result
    pub purpose: ClassifiedPurpose,
    /// Context for classification
    #[serde(default)]
    pub context: Option<EntityClassificationContext>,
}

/// Context for entity classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityClassificationContext {
    /// Preceding segment content (for context)
    #[serde(default)]
    pub preceding: Option<String>,
    /// Following segment content (for context)
    #[serde(default)]
    pub following: Option<String>,
    /// Known threads for reference
    #[serde(default)]
    pub known_threads: Vec<ThreadReference>,
    /// User's entity type distribution
    #[serde(default)]
    pub user_patterns: Option<EntityPatterns>,
}

/// Reference to an existing thread
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadReference {
    /// Thread ID
    pub id: String,
    /// Thread name/label
    pub name: String,
    /// Thread type (person, place, concept, etc.)
    pub thread_type: String,
    /// Aliases for matching
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// User's historical entity patterns
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityPatterns {
    /// Most common entity type
    pub primary_type: EntityType,
    /// Distribution (entity_type -> frequency)
    #[serde(default)]
    pub distribution: std::collections::HashMap<String, f64>,
}

// ============================================================================
// Entity Types (Symmetric Seven)
// ============================================================================

/// Entity types from the Symmetric Seven ontology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    /// A specific event/action that happened (Narrative/External Particle)
    /// Linguistic marker: Action verbs (went, did, met, called)
    Moment,
    /// Internal state/feeling/emotion (Internal Particle)
    /// Linguistic marker: State verbs + evaluative language (felt, was nice)
    Pulse,
    /// Definition of person, place, or concept (Definitional/Object)
    Thread,
    /// Relationship between entities (Relational/Connection)
    Bond,
    /// A recurring external pattern (External Wave)
    Motif,
    /// A recurring internal pattern (Internal Wave)
    Filament,
    /// An active thematic goal (Intentional Wave)
    Focus,
    /// A task or goal for the future (Operational/Intentional Particle)
    Intent,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Moment => "MOMENT",
            Self::Pulse => "PULSE",
            Self::Thread => "THREAD",
            Self::Bond => "BOND",
            Self::Motif => "MOTIF",
            Self::Filament => "FILAMENT",
            Self::Focus => "FOCUS",
            Self::Intent => "INTENT",
        }
    }

    /// Description for prompts
    pub fn description(&self) -> &'static str {
        match self {
            Self::Moment => "A discrete event/action - WHAT HAPPENED. Uses action verbs: went, did, met, called, visited.",
            Self::Pulse => "Internal state/feeling - HOW IT WAS/FELT. Uses state verbs + evaluative: felt, was nice, seemed.",
            Self::Thread => "An ongoing narrative, topic, person, or concept being discussed.",
            Self::Bond => "A statement about the relationship between two entities.",
            Self::Motif => "A recurring external pattern noticed over time.",
            Self::Filament => "A recurring internal pattern (habits, tendencies, reactions).",
            Self::Focus => "An active goal or thematic intention being pursued.",
            Self::Intent => "A future-oriented task or goal to accomplish.",
        }
    }

    /// Whether this entity typically spawns new records
    pub fn spawns_entity(&self) -> bool {
        matches!(
            self,
            Self::Moment | Self::Pulse | Self::Intent | Self::Bond
        )
    }

    /// Whether this entity type references existing entities
    pub fn references_entity(&self) -> bool {
        matches!(
            self,
            Self::Thread | Self::Motif | Self::Filament | Self::Focus | Self::Bond
        )
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Classification Output
// ============================================================================

/// Output from entity classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityClassifierOutput {
    /// All classifications with probabilities (sorted by probability)
    pub classifications: Vec<EntityClassification>,
    /// Primary thread reference (main subject/actor)
    #[serde(default)]
    pub primary_thread: Option<ThreadReference>,
    /// Secondary thread references
    #[serde(default)]
    pub secondary_threads: Vec<ThreadReference>,
    /// Temporal marker extracted
    #[serde(default)]
    pub temporal_marker: Option<String>,
    /// Enriched content (e.g., Pulse with context added)
    #[serde(default)]
    pub enriched_content: Option<String>,
    /// Classification reasoning
    pub reasoning: ClassificationReasoning,
}

/// A single entity classification with probability
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityClassification {
    /// Entity type
    pub entity_type: EntityType,
    /// Probability (0.0 to 1.0)
    pub probability: f64,
    /// Evidence supporting this classification
    #[serde(default)]
    pub evidence: Vec<ClassificationEvidence>,
}

impl EntityClassification {
    pub fn new(entity_type: EntityType, probability: f64) -> Self {
        Self {
            entity_type,
            probability,
            evidence: vec![],
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<ClassificationEvidence>) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Evidence for a classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassificationEvidence {
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Description of evidence
    pub description: String,
    /// How much this evidence contributes
    pub weight: f64,
}

/// Types of evidence for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Verb type (action vs state)
    VerbType,
    /// Keywords detected
    Keywords,
    /// Temporal indicators
    Temporal,
    /// Emotional indicators
    Emotional,
    /// Entity mentions
    EntityMention,
    /// Relational language
    Relational,
    /// Pattern indicators
    Pattern,
    /// Context from surrounding content
    Context,
}

/// Reasoning behind classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassificationReasoning {
    /// Main verb identified
    #[serde(default)]
    pub main_verb: Option<String>,
    /// Verb category
    #[serde(default)]
    pub verb_category: Option<VerbCategory>,
    /// Key phrases that influenced classification
    #[serde(default)]
    pub key_phrases: Vec<String>,
    /// Why certain types were included/excluded
    #[serde(default)]
    pub explanations: Vec<String>,
}

/// Categories of verbs for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerbCategory {
    /// Action verbs (went, did, called) → MOMENT
    Action,
    /// State verbs (was, felt, is) → often PULSE
    State,
    /// Stative + evaluative (was nice, felt good) → PULSE
    Evaluative,
    /// Relational verbs (met, talked with) → may indicate BOND
    Relational,
    /// Habitual (always, usually does) → may indicate MOTIF/FILAMENT
    Habitual,
    /// Intentional (want to, plan to) → INTENT
    Intentional,
}

// ============================================================================
// Linguistic Analysis Helpers
// ============================================================================

/// Result of linguistic analysis for classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LinguisticAnalysis {
    /// Identified verbs
    pub verbs: Vec<IdentifiedVerb>,
    /// Subjects identified
    #[serde(default)]
    pub subjects: Vec<String>,
    /// Objects identified
    #[serde(default)]
    pub objects: Vec<String>,
    /// Adverbs/modifiers that affect classification
    #[serde(default)]
    pub modifiers: Vec<String>,
}

/// An identified verb with analysis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IdentifiedVerb {
    /// The verb text
    pub text: String,
    /// Verb category
    pub category: VerbCategory,
    /// Tense
    #[serde(default)]
    pub tense: Option<VerbTense>,
    /// Is this the main verb?
    pub is_main: bool,
}

/// Verb tenses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerbTense {
    Past,
    Present,
    Future,
    Habitual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_descriptions() {
        assert!(EntityType::Moment.description().contains("action"));
        assert!(EntityType::Pulse.description().contains("feeling"));
    }

    #[test]
    fn test_entity_classification() {
        let mut classifications = vec![
            EntityClassification::new(EntityType::Moment, 0.85),
            EntityClassification::new(EntityType::Pulse, 0.70),
        ];

        // Sort by probability descending
        classifications.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        assert_eq!(classifications[0].entity_type, EntityType::Moment);
        assert_eq!(classifications[1].entity_type, EntityType::Pulse);
    }
}
