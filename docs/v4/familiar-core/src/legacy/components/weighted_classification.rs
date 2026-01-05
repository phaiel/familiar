use serde::{Deserialize, Serialize};

use crate::primitives::NormalizedFloat;
use crate::types::HeddleEntityType;

/// A weighted classification representing the probability/confidence
/// that a segment belongs to a specific entity type.
/// Part of the Superposition model - a single thought can be multiple things.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeightedClassification {
    /// The entity type this classification refers to
    pub entity_type: HeddleEntityType,
    
    /// Confidence/Weight (0.0 to 1.0)
    /// If > Threshold during collapse, we instantiate this entity.
    pub weight: NormalizedFloat,
}

impl WeightedClassification {
    pub fn new(entity_type: HeddleEntityType, weight: f64) -> Result<Self, String> {
        Ok(Self {
            entity_type,
            weight: NormalizedFloat::new(weight)?,
        })
    }
    
    /// Check if this classification exceeds the collapse threshold
    pub fn should_collapse(&self, threshold: f64) -> bool {
        self.weight.value() >= threshold
    }
}

/// A collection of weighted classifications representing the superposition
/// of possible entity types for a single segment.
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassificationSuperposition {
    pub classifications: Vec<WeightedClassification>,
}

impl ClassificationSuperposition {
    pub fn new() -> Self {
        Self { classifications: Vec::new() }
    }
    
    pub fn add(&mut self, entity_type: HeddleEntityType, weight: f64) -> Result<(), String> {
        self.classifications.push(WeightedClassification::new(entity_type, weight)?);
        Ok(())
    }
    
    /// Get all classifications that exceed the collapse threshold
    pub fn get_collapsible(&self, threshold: f64) -> Vec<&WeightedClassification> {
        self.classifications
            .iter()
            .filter(|c| c.should_collapse(threshold))
            .collect()
    }
    
    /// Get the dominant (highest weight) classification
    pub fn dominant(&self) -> Option<&WeightedClassification> {
        self.classifications
            .iter()
            .max_by(|a, b| a.weight.value().partial_cmp(&b.weight.value()).unwrap())
    }
}

