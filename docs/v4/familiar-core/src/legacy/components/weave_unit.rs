//! WeaveUnit Component
//!
//! A WeaveUnit is a single segment extracted from a Weave.
//! Each unit represents an atomic thought/idea that can be classified.
//!
//! WeaveUnits are transient - they exist only during processing and are
//! eventually purged. Only the spawned entities persist in the simulation.

use serde::{Deserialize, Serialize};

use crate::primitives::{UUID, NormalizedFloat};
use crate::types::{HeddleEntityType, MessageIntent};

/// Classification result for a weave unit (determines which entity type to spawn)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeaveUnitClassification {
    /// The entity type this unit may become
    pub entity_type: HeddleEntityType,
    /// Confidence weight (0.0 to 1.0)
    pub weight: NormalizedFloat,
}

/// A WeaveUnit is a single segment extracted from a Weave.
/// It's a transient container used for classification routing.
/// Physics are extracted by the LLM but passed directly to spawned entities,
/// not stored on the WeaveUnit itself.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeaveUnit {
    /// Index of this unit within the shuttle (0-based)
    pub index: usize,
    
    /// The extracted/cleaned text content for this segment
    pub content: String,
    
    /// Purpose of this specific unit (LOG, QUERY, COMMAND, INFER, REFERENCE)
    /// Determines how this unit is processed - only LOG units spawn entities
    #[serde(default)]
    pub purpose: MessageIntent,
    
    /// Primary thread: the main subject/actor of this unit
    /// Used for entity resolution (Stitch) downstream
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_thread: Option<String>,
    
    /// Secondary threads: other people/places/things mentioned
    /// Allows capturing companions, locations, etc.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_threads: Vec<String>,
    
    /// Temporal marker: when this event/state occurred
    /// Can be absolute ("6pm"), relative ("today"), or frequency ("once per hour")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_marker: Option<String>,
    
    /// Classifications in superposition (determines which entity types to spawn)
    #[serde(default)]
    pub classifications: Vec<WeaveUnitClassification>,
    
    /// IDs of entities spawned from this unit (the actual simulation objects)
    #[serde(default)]
    pub spawned_entity_ids: Vec<UUID>,
}

impl WeaveUnit {
    pub fn new(index: usize, content: impl Into<String>) -> Self {
        Self {
            index,
            content: content.into(),
            purpose: MessageIntent::Log,  // Default to LOG
            primary_thread: None,
            secondary_threads: vec![],
            temporal_marker: None,
            classifications: vec![],
            spawned_entity_ids: vec![],
        }
    }

    pub fn with_purpose(mut self, purpose: MessageIntent) -> Self {
        self.purpose = purpose;
        self
    }

    pub fn with_primary_thread(mut self, thread: impl Into<String>) -> Self {
        self.primary_thread = Some(thread.into());
        self
    }

    pub fn with_secondary_threads(mut self, threads: Vec<String>) -> Self {
        self.secondary_threads = threads;
        self
    }

    pub fn with_temporal_marker(mut self, marker: impl Into<String>) -> Self {
        self.temporal_marker = Some(marker.into());
        self
    }

    pub fn add_secondary_thread(&mut self, thread: impl Into<String>) {
        self.secondary_threads.push(thread.into());
    }

    /// Legacy alias for backward compatibility
    pub fn with_thread_hint(self, hint: impl Into<String>) -> Self {
        self.with_primary_thread(hint)
    }

    /// Get all threads (primary + secondary)
    pub fn all_threads(&self) -> Vec<&str> {
        let mut threads = Vec::new();
        if let Some(p) = &self.primary_thread {
            threads.push(p.as_str());
        }
        threads.extend(self.secondary_threads.iter().map(|s| s.as_str()));
        threads
    }

    pub fn add_classification(&mut self, entity_type: HeddleEntityType, weight: f64) -> Result<(), String> {
        let weight = NormalizedFloat::new(weight)?;
        self.classifications.push(WeaveUnitClassification { entity_type, weight });
        Ok(())
    }

    pub fn add_spawned(&mut self, entity_id: UUID) {
        self.spawned_entity_ids.push(entity_id);
    }

    /// Get the dominant classification (highest weight)
    pub fn dominant_classification(&self) -> Option<&WeaveUnitClassification> {
        self.classifications.iter()
            .max_by(|a, b| a.weight.value().partial_cmp(&b.weight.value()).unwrap())
    }

    /// Get classifications above a threshold (for superposition collapse)
    pub fn classifications_above(&self, threshold: f64) -> Vec<&WeaveUnitClassification> {
        self.classifications.iter()
            .filter(|c| c.weight.value() >= threshold)
            .collect()
    }

    /// Check if this unit has any classifications
    pub fn is_classified(&self) -> bool {
        !self.classifications.is_empty()
    }

    /// Check if this unit should spawn entities (only LOG purpose units do)
    pub fn should_spawn(&self) -> bool {
        self.purpose == MessageIntent::Log && self.is_classified()
    }

    /// Check if entities have been spawned from this unit
    pub fn has_spawned(&self) -> bool {
        !self.spawned_entity_ids.is_empty()
    }

    /// Get the number of spawned entities
    pub fn spawn_count(&self) -> usize {
        self.spawned_entity_ids.len()
    }
}
