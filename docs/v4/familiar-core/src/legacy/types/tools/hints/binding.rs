//! Binding Hint Tool Schema
//!
//! NEW: Cognitive binding between entities.
//! Bindings represent how memories/entities are connected in cognitive space:
//! - Causal: One leads to another
//! - Temporal: Sequence in time
//! - Associative: Connected by context
//! - Compositional: Part-whole relationships

use serde::{Deserialize, Serialize};

use crate::types::tools::entity::EntityType;
use crate::types::tools::segmentation::Segment;
use crate::types::tools::spawn::{BindingType, Directionality, TemporalRelation};

// ============================================================================
// Binding Hint Tool
// ============================================================================

/// Input for binding hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingHintInput {
    /// The current segment
    pub segment: Segment,
    /// Entity being spawned
    pub current_entity: EntityRef,
    /// Recent entities for binding detection
    #[serde(default)]
    pub recent_entities: Vec<EntityRef>,
    /// Context window for binding analysis
    #[serde(default)]
    pub context_window: Option<ContextWindow>,
}

/// Reference to an entity for binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityRef {
    /// Entity ID (if existing)
    #[serde(default)]
    pub id: Option<String>,
    /// Entity type
    pub entity_type: EntityType,
    /// Content/description
    pub content: String,
    /// When created (for temporal binding)
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Context window for binding analysis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContextWindow {
    /// Preceding content
    #[serde(default)]
    pub preceding: Vec<String>,
    /// Following content
    #[serde(default)]
    pub following: Vec<String>,
    /// Time span in hours
    #[serde(default)]
    pub time_span_hours: Option<f64>,
}

/// Output from binding hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingHintOutput {
    /// Detected bindings
    pub bindings: Vec<DetectedBinding>,
    /// Binding graph additions
    #[serde(default)]
    pub graph_updates: Vec<BindingGraphUpdate>,
}

// ============================================================================
// Binding Types
// ============================================================================

/// A detected binding between entities
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedBinding {
    /// Source entity
    pub source: EntityRef,
    /// Target entity
    pub target: EntityRef,
    /// Binding characteristics
    pub binding: BindingCharacteristics,
    /// Evidence for the binding
    pub evidence: BindingEvidence,
    /// Confidence in detection
    pub confidence: f64,
}

/// Binding characteristics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingCharacteristics {
    /// Type of cognitive binding
    pub binding_type: BindingType,
    /// Strength of binding (0.0 to 1.0)
    pub strength: f64,
    /// Directionality
    pub directionality: Directionality,
    /// Salience (how notable/memorable)
    pub salience: f64,
    /// Temporal properties
    #[serde(default)]
    pub temporal: Option<TemporalProperties>,
    /// Semantic properties
    #[serde(default)]
    pub semantic: Option<SemanticProperties>,
}


/// Temporal properties of binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TemporalProperties {
    /// Time relation
    pub relation: TemporalRelation,
    /// Time gap between entities
    #[serde(default)]
    pub gap: Option<TimeGap>,
    /// Is this a recurring pattern?
    #[serde(default)]
    pub recurring: bool,
}


/// Time gap between entities
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TimeGap {
    /// Duration value
    pub value: f64,
    /// Duration unit
    pub unit: TimeUnit,
    /// Is this approximate?
    pub approximate: bool,
}

/// Time units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

/// Semantic properties of binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SemanticProperties {
    /// Shared concepts/topics
    #[serde(default)]
    pub shared_concepts: Vec<String>,
    /// Semantic similarity score
    pub similarity: f64,
    /// Relationship description
    #[serde(default)]
    pub relationship_description: Option<String>,
}

// ============================================================================
// Binding Evidence
// ============================================================================

/// Evidence for a detected binding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingEvidence {
    /// Type of evidence
    pub evidence_type: BindingEvidenceType,
    /// Specific markers found
    #[serde(default)]
    pub markers: Vec<String>,
    /// Text that triggered detection
    pub trigger_text: String,
    /// Explanation
    pub explanation: String,
}

/// Types of binding evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BindingEvidenceType {
    /// Explicit linguistic marker
    LinguisticMarker,
    /// Temporal proximity
    TemporalProximity,
    /// Semantic similarity
    SemanticSimilarity,
    /// Entity co-reference
    CoReference,
    /// Emotional similarity
    EmotionalSimilarity,
    /// Narrative continuity
    NarrativeContinuity,
    /// User explicit statement
    ExplicitStatement,
    /// Inferred from context
    ContextualInference,
}

// ============================================================================
// Binding Graph
// ============================================================================

/// Update to the binding graph
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingGraphUpdate {
    /// Type of update
    pub update_type: GraphUpdateType,
    /// Source entity ID
    pub source_id: String,
    /// Target entity ID
    #[serde(default)]
    pub target_id: Option<String>,
    /// Binding characteristics
    #[serde(default)]
    pub binding: Option<BindingCharacteristics>,
}

/// Types of graph updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphUpdateType {
    /// Add new binding
    AddBinding,
    /// Strengthen existing binding
    StrengthenBinding,
    /// Weaken existing binding
    WeakenBinding,
    /// Remove binding
    RemoveBinding,
    /// Merge duplicate bindings
    MergeBindings,
}

// ============================================================================
// Binding Patterns
// ============================================================================

/// Common binding patterns for detection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BindingPattern {
    /// Pattern name
    pub name: String,
    /// Binding type this pattern indicates
    pub binding_type: BindingType,
    /// Regex or linguistic pattern
    pub pattern: String,
    /// Confidence boost when matched
    pub confidence_boost: f64,
}

impl BindingPattern {
    /// Get common binding patterns
    pub fn common_patterns() -> Vec<Self> {
        vec![
            Self {
                name: "causal_because".to_string(),
                binding_type: BindingType::Causal,
                pattern: r"because|since|as a result|therefore|so that".to_string(),
                confidence_boost: 0.3,
            },
            Self {
                name: "temporal_sequence".to_string(),
                binding_type: BindingType::Temporal,
                pattern: r"then|after|before|later|earlier|next".to_string(),
                confidence_boost: 0.25,
            },
            Self {
                name: "contrast".to_string(),
                binding_type: BindingType::Contrastive,
                pattern: r"but|however|unlike|instead|whereas".to_string(),
                confidence_boost: 0.3,
            },
            Self {
                name: "analogy".to_string(),
                binding_type: BindingType::Analogical,
                pattern: r"like|reminds me of|similar to|same as".to_string(),
                confidence_boost: 0.35,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_patterns() {
        let patterns = BindingPattern::common_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.binding_type == BindingType::Causal));
    }
}
