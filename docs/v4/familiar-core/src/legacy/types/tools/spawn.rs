//! Entity Spawn Suggestion Tool Schema
//!
//! Decides which entities to create based on classification confidence
//! and applies hints for physics, threads, bonds, and bindings.

use serde::{Deserialize, Serialize};

use super::entity::{EntityClassifierOutput, EntityType, ThreadReference};
use super::segmentation::Segment;

// ============================================================================
// Spawn Tool Input/Output
// ============================================================================

/// Input for spawn suggestion tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnSuggesterInput {
    /// Original segment
    pub segment: Segment,
    /// Entity classification result
    pub classification: EntityClassifierOutput,
    /// Spawn configuration
    #[serde(default)]
    pub config: SpawnConfig,
}

/// Configuration for spawn decisions
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnConfig {
    /// High confidence threshold (auto-spawn above this)
    #[serde(default = "default_high_threshold")]
    pub high_confidence_threshold: f64,
    /// Medium confidence threshold (suggest above this)
    #[serde(default = "default_medium_threshold")]
    pub medium_confidence_threshold: f64,
    /// Low confidence threshold (skip below this)
    #[serde(default = "default_low_threshold")]
    pub low_confidence_threshold: f64,
    /// Maximum entities to spawn from one segment
    #[serde(default = "default_max_spawn")]
    pub max_entities_per_segment: usize,
    /// Whether to auto-spawn high confidence entities
    #[serde(default = "default_true")]
    pub auto_spawn_enabled: bool,
    /// Entity types that should never auto-spawn
    #[serde(default)]
    pub manual_only_types: Vec<EntityType>,
}

fn default_high_threshold() -> f64 {
    0.8
}
fn default_medium_threshold() -> f64 {
    0.5
}
fn default_low_threshold() -> f64 {
    0.3
}
fn default_max_spawn() -> usize {
    3
}
fn default_true() -> bool {
    true
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            high_confidence_threshold: default_high_threshold(),
            medium_confidence_threshold: default_medium_threshold(),
            low_confidence_threshold: default_low_threshold(),
            max_entities_per_segment: default_max_spawn(),
            auto_spawn_enabled: true,
            manual_only_types: vec![],
        }
    }
}

// ============================================================================
// Spawn Output
// ============================================================================

/// Output from spawn suggestion tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnSuggesterOutput {
    /// Entities to spawn
    pub suggestions: Vec<SpawnSuggestion>,
    /// Overall spawn decision summary
    pub summary: SpawnSummary,
}

/// Summary of spawn decisions
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnSummary {
    /// Total suggestions
    pub total_suggestions: usize,
    /// Auto-spawn count
    pub auto_spawn_count: usize,
    /// Needs review count
    pub review_count: usize,
    /// Skipped count
    pub skipped_count: usize,
    /// Primary entity type suggested
    #[serde(default)]
    pub primary_type: Option<EntityType>,
}

// ============================================================================
// Spawn Suggestion
// ============================================================================

/// A suggestion to spawn an entity
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnSuggestion {
    /// Entity type to spawn
    pub entity_type: EntityType,
    /// Confidence in this spawn
    pub confidence: f64,
    /// Spawn action
    pub action: SpawnAction,
    /// Reason for suggestion
    pub reason: String,
    /// Content to use for entity
    pub content: EntityContent,
    /// All hints for the entity
    pub hints: SpawnHints,
}

/// Action to take for spawn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpawnAction {
    /// Auto-spawn without user confirmation
    AutoSpawn,
    /// Suggest to user for review
    Suggest,
    /// Skip but log for analysis
    Skip,
    /// Request clarification from user
    Clarify,
}

/// Content for the spawned entity
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityContent {
    /// Primary content text
    pub text: String,
    /// Enriched/contextualized content
    #[serde(default)]
    pub enriched_text: Option<String>,
    /// Title/summary
    #[serde(default)]
    pub title: Option<String>,
    /// Tags extracted
    #[serde(default)]
    pub tags: Vec<String>,
}

// ============================================================================
// Spawn Hints (Combined)
// ============================================================================

/// All hints for a spawned entity
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SpawnHints {
    /// Physics hints for VAE space positioning
    pub physics: PhysicsHints,
    /// Thread hints for narrative connections
    pub thread: ThreadHints,
    /// Bond hints for relationships
    #[serde(default)]
    pub bond: Option<BondHints>,
    /// Binding hints for cognitive connections
    #[serde(default)]
    pub binding: Option<BindingHints>,
}

impl Default for SpawnHints {
    fn default() -> Self {
        Self {
            physics: PhysicsHints::default(),
            thread: ThreadHints::default(),
            bond: None,
            binding: None,
        }
    }
}

// ============================================================================
// Physics Hints
// ============================================================================

/// Hints for physics/VAE space positioning
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHints {
    /// Valence: -1.0 (negative) to 1.0 (positive)
    pub valence: f64,
    /// Arousal: 0.0 (calm) to 1.0 (excited/energized)
    pub arousal: f64,
    /// Significance/mass: 0.0 (trivial) to 1.0 (very important)
    pub significance: f64,
    /// Epistemic certainty: 0.0 (uncertain) to 1.0 (certain)
    #[serde(default = "default_half")]
    pub certainty: f64,
    /// Reasoning for physics values
    #[serde(default)]
    pub reasoning: Option<String>,
}

fn default_half() -> f64 {
    0.5
}

impl Default for PhysicsHints {
    fn default() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            significance: 0.5,
            certainty: 0.5,
            reasoning: None,
        }
    }
}

impl PhysicsHints {
    /// Create hints from emotional content
    pub fn from_emotion(emotion: &str, intensity: f64) -> Self {
        let (valence, arousal) = match emotion.to_lowercase().as_str() {
            "happy" | "joy" | "excited" => (0.8, 0.7),
            "sad" | "depressed" => (-0.7, 0.3),
            "angry" | "frustrated" => (-0.6, 0.8),
            "calm" | "peaceful" => (0.4, 0.2),
            "anxious" | "worried" => (-0.4, 0.7),
            "neutral" => (0.0, 0.5),
            _ => (0.0, 0.5),
        };

        Self {
            valence: valence * intensity,
            arousal: arousal * intensity,
            significance: 0.5,
            certainty: 0.5,
            reasoning: Some(format!("Derived from emotion: {}", emotion)),
        }
    }

    /// Convert to VAE position array
    pub fn to_vae_position(&self) -> [f64; 3] {
        [self.valence, self.arousal, self.certainty]
    }
}

// ============================================================================
// Thread Hints
// ============================================================================

/// Hints for thread/narrative connections
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ThreadHints {
    /// Primary subject/actor
    pub primary_subject: String,
    /// Related threads to link
    #[serde(default)]
    pub related_threads: Vec<ThreadReference>,
    /// Role of entity in narrative
    pub thread_role: ThreadRole,
    /// Keywords for thread matching
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Should create new thread if no match?
    #[serde(default)]
    pub create_if_missing: bool,
}

impl Default for ThreadHints {
    fn default() -> Self {
        Self {
            primary_subject: "user".to_string(),
            related_threads: vec![],
            thread_role: ThreadRole::Subject,
            keywords: vec![],
            create_if_missing: false,
        }
    }
}

/// Role of entity relative to threads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadRole {
    /// Entity is the actor/doer
    Actor,
    /// Entity is the subject being discussed
    Subject,
    /// Entity is observing/witnessing
    Observer,
    /// Entity is the target of action
    Target,
    /// Entity is setting/location
    Setting,
    /// Entity is an instrument/tool
    Instrument,
}

// ============================================================================
// Bond Hints
// ============================================================================

/// Hints for relationship bonds
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondHints {
    /// Source entity (head of relationship)
    pub source: ThreadReference,
    /// Target entity (tail of relationship)
    pub target: ThreadReference,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Strength of relationship (0.0 to 1.0)
    pub strength: f64,
    /// Valence of relationship (-1.0 to 1.0)
    pub valence: f64,
    /// Reciprocity (0.0 one-way to 1.0 mutual)
    #[serde(default = "default_half")]
    pub reciprocity: f64,
    /// Description of relationship
    #[serde(default)]
    pub description: Option<String>,
}

// Re-export RelationshipType from canonical location
pub use super::hints::bond::RelationshipType;

// ============================================================================
// Binding Hints (NEW - Cognitive Binding)
// ============================================================================

/// Hints for cognitive binding between entities
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingHints {
    /// Source entity
    pub source: EntityReference,
    /// Target entity
    pub target: EntityReference,
    /// Type of cognitive binding
    pub binding_type: BindingType,
    /// Strength of binding (0.0 to 1.0)
    pub strength: f64,
    /// Directionality of binding
    pub directionality: Directionality,
    /// Context explaining the binding
    #[serde(default)]
    pub context: Option<String>,
    /// Temporal aspect of binding
    #[serde(default)]
    pub temporal: Option<TemporalBinding>,
}

/// Reference to an entity for binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityReference {
    /// Entity ID (if existing)
    #[serde(default)]
    pub id: Option<String>,
    /// Entity type
    pub entity_type: EntityType,
    /// Description for matching
    pub description: String,
}

/// Types of cognitive binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BindingType {
    /// Cause leads to effect
    Causal,
    /// Temporal sequence (before/after)
    Temporal,
    /// Associated by proximity/context
    Associative,
    /// Part-whole relationship
    Compositional,
    /// Contrast/opposition
    Contrastive,
    /// Similarity/analogy
    Analogical,
    /// Enables/prerequisite
    Enabling,
    /// Thematic connection
    Thematic,
}

impl BindingType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Causal => "One entity causes or leads to another",
            Self::Temporal => "Entities are connected by time sequence",
            Self::Associative => "Entities are associated by context or proximity",
            Self::Compositional => "One entity is part of another",
            Self::Contrastive => "Entities are contrasted or opposed",
            Self::Analogical => "Entities are similar or analogous",
            Self::Enabling => "One entity enables or is prerequisite for another",
            Self::Thematic => "Entities share a common theme",
        }
    }
}

/// Directionality of binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Directionality {
    /// Only source → target
    Unidirectional,
    /// Both directions
    Bidirectional,
    /// No inherent direction
    Undirected,
}

/// Temporal aspect of binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TemporalBinding {
    /// Time relationship
    pub relation: TemporalRelation,
    /// Time gap (if known)
    #[serde(default)]
    pub gap: Option<String>,
}

/// Temporal relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalRelation {
    Before,
    After,
    During,
    Simultaneous,
    Overlapping,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_hints_from_emotion() {
        let hints = PhysicsHints::from_emotion("happy", 1.0);
        assert!(hints.valence > 0.0);
        assert!(hints.arousal > 0.5);
    }

    #[test]
    fn test_spawn_action_thresholds() {
        let config = SpawnConfig::default();
        assert!(config.high_confidence_threshold > config.medium_confidence_threshold);
        assert!(config.medium_confidence_threshold > config.low_confidence_threshold);
    }

    #[test]
    fn test_binding_type_descriptions() {
        assert!(BindingType::Causal.description().contains("cause"));
        assert!(BindingType::Temporal.description().contains("time"));
    }
}
