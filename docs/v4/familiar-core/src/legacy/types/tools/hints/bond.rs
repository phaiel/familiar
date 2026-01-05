//! Bond Hint Tool Schema
//!
//! Detects and characterizes relationships between entities.
//! Bonds connect threads (people, places, concepts) with typed, weighted relationships.

use serde::{Deserialize, Serialize};

use crate::types::tools::entity::ThreadReference;
use crate::types::tools::segmentation::Segment;

// ============================================================================
// Bond Hint Tool
// ============================================================================

/// Input for bond hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondHintInput {
    /// The segment containing relationship information
    pub segment: Segment,
    /// Detected threads in the segment
    pub threads: Vec<ThreadReference>,
    /// Known bonds for context
    #[serde(default)]
    pub known_bonds: Vec<ExistingBond>,
}

/// An existing bond for context
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExistingBond {
    pub id: String,
    pub head_thread_id: String,
    pub tail_thread_id: String,
    pub relationship_type: RelationshipType,
    pub strength: f64,
}

/// Output from bond hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondHintOutput {
    /// Detected bonds
    pub bonds: Vec<DetectedBond>,
    /// Updates to existing bonds
    #[serde(default)]
    pub bond_updates: Vec<BondUpdate>,
}

/// A detected bond between entities
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedBond {
    /// Source entity (head)
    pub head: ThreadReference,
    /// Target entity (tail)
    pub tail: ThreadReference,
    /// Bond characteristics
    pub characteristics: BondCharacteristics,
    /// Evidence for this bond
    pub evidence: BondEvidence,
    /// Confidence in detection
    pub confidence: f64,
}

/// Bond characteristics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondCharacteristics {
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Strength of relationship (0.0 weak to 1.0 strong)
    pub strength: f64,
    /// Valence (-1.0 negative to 1.0 positive)
    pub valence: f64,
    /// Reciprocity (0.0 one-sided to 1.0 mutual)
    pub reciprocity: f64,
    /// Intimacy level (0.0 distant to 1.0 close)
    pub intimacy: f64,
    /// Trust level (0.0 to 1.0)
    pub trust: f64,
    /// Frequency of interaction
    #[serde(default)]
    pub interaction_frequency: Option<InteractionFrequency>,
    /// Duration of relationship
    #[serde(default)]
    pub duration: Option<RelationshipDuration>,
}

impl Default for BondCharacteristics {
    fn default() -> Self {
        Self {
            relationship_type: RelationshipType::Acquaintance,
            strength: 0.5,
            valence: 0.5,
            reciprocity: 0.5,
            intimacy: 0.3,
            trust: 0.5,
            interaction_frequency: None,
            duration: None,
        }
    }
}

/// Types of relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    // Personal
    Family,
    Friend,
    CloseFriend,
    BestFriend,
    Romantic,
    ExRomantic,
    Spouse,
    Parent,
    Child,
    Sibling,
    
    // Professional
    Colleague,
    Manager,
    DirectReport,
    Mentor,
    Mentee,
    Client,
    Vendor,
    Partner,
    
    // Social
    Acquaintance,
    Neighbor,
    Classmate,
    Teammate,
    
    // Negative
    Adversary,
    Rival,
    
    // Neutral/Other
    ServiceProvider,
    Other,
}

impl RelationshipType {
    /// Default characteristics for this relationship type
    pub fn default_characteristics(&self) -> BondCharacteristics {
        match self {
            Self::Family => BondCharacteristics {
                relationship_type: *self,
                strength: 0.8,
                valence: 0.6,
                reciprocity: 0.8,
                intimacy: 0.7,
                trust: 0.7,
                ..Default::default()
            },
            Self::Friend | Self::CloseFriend => BondCharacteristics {
                relationship_type: *self,
                strength: 0.7,
                valence: 0.8,
                reciprocity: 0.8,
                intimacy: 0.6,
                trust: 0.7,
                ..Default::default()
            },
            Self::BestFriend => BondCharacteristics {
                relationship_type: *self,
                strength: 0.9,
                valence: 0.9,
                reciprocity: 0.9,
                intimacy: 0.9,
                trust: 0.9,
                ..Default::default()
            },
            Self::Romantic | Self::Spouse => BondCharacteristics {
                relationship_type: *self,
                strength: 0.9,
                valence: 0.9,
                reciprocity: 0.9,
                intimacy: 0.95,
                trust: 0.9,
                ..Default::default()
            },
            Self::Colleague => BondCharacteristics {
                relationship_type: *self,
                strength: 0.5,
                valence: 0.5,
                reciprocity: 0.6,
                intimacy: 0.3,
                trust: 0.5,
                ..Default::default()
            },
            Self::Adversary | Self::Rival => BondCharacteristics {
                relationship_type: *self,
                strength: 0.6,
                valence: -0.5,
                reciprocity: 0.5,
                intimacy: 0.2,
                trust: 0.1,
                ..Default::default()
            },
            Self::Acquaintance => BondCharacteristics {
                relationship_type: *self,
                strength: 0.3,
                valence: 0.5,
                reciprocity: 0.4,
                intimacy: 0.1,
                trust: 0.4,
                ..Default::default()
            },
            _ => BondCharacteristics {
                relationship_type: *self,
                ..Default::default()
            },
        }
    }
}

/// Interaction frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InteractionFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Rarely,
}

/// Relationship duration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RelationshipDuration {
    /// How long the relationship has existed
    #[serde(default)]
    pub years: Option<f64>,
    /// Start date if known
    #[serde(default)]
    pub since: Option<String>,
    /// Duration category
    pub category: DurationCategory,
}

/// Duration categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DurationCategory {
    New,        // < 3 months
    Recent,     // 3-12 months
    Established, // 1-5 years
    LongTerm,   // 5-15 years
    Lifelong,   // 15+ years
}

/// Evidence for bond detection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondEvidence {
    /// Text that indicates the relationship
    pub trigger_text: String,
    /// Type of indicator
    pub indicator_type: BondIndicatorType,
    /// Specific indicators found
    #[serde(default)]
    pub indicators: Vec<String>,
}

/// Types of bond indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondIndicatorType {
    /// Explicit label ("my friend", "my boss")
    ExplicitLabel,
    /// Action indicating relationship ("had dinner with")
    RelationalAction,
    /// Emotional language about entity
    EmotionalLanguage,
    /// Possessive language ("my", "our")
    PossessiveLanguage,
    /// Frequency indicators ("always see", "often talk")
    FrequencyIndicator,
    /// Context from previous interactions
    ContextualInference,
}

/// Update to an existing bond
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondUpdate {
    /// ID of bond to update
    pub bond_id: String,
    /// Changes to apply
    pub changes: BondChanges,
    /// Reason for update
    pub reason: String,
}

/// Changes to bond characteristics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondChanges {
    #[serde(default)]
    pub strength_delta: Option<f64>,
    #[serde(default)]
    pub valence_delta: Option<f64>,
    #[serde(default)]
    pub trust_delta: Option<f64>,
    #[serde(default)]
    pub new_relationship_type: Option<RelationshipType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_defaults() {
        let chars = RelationshipType::BestFriend.default_characteristics();
        assert!(chars.strength > 0.8);
        assert!(chars.intimacy > 0.8);
    }

    #[test]
    fn test_negative_relationship() {
        let chars = RelationshipType::Adversary.default_characteristics();
        assert!(chars.valence < 0.0);
        assert!(chars.trust < 0.3);
    }
}
