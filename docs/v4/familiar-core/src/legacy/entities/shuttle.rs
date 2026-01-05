//! Shuttle Entity - The Transient Unit of Work
//!
//! The Shuttle is the "carrier" of work in the Loom Pattern:
//! - Course = persistent session/history bucket (1)
//! - Shuttle = transient unit of work (N per Course)
//! - Thread = THREAD entity (Person/Concept) - protected domain term
//!
//! There is a 1:N relationship between Course and Shuttle.
//! Each user message creates a new Shuttle that is tethered to its Course.
//!
//! Shuttles are transient - they exist only during processing.
//! When complete, results are committed to the Course history.

use serde::{Deserialize, Serialize};

use crate::primitives::{UUID, Timestamp};
use crate::types::ShuttleStatus;
use crate::components::{Weave, WeaveUnit};
use crate::types::HeddleEntityType;

// ============================================================================
// Shuttle Details (Processing Metadata)
// ============================================================================

/// Processing metadata for a Shuttle
/// 
/// These are transient concerns - each Shuttle may use different providers,
/// models, or have different latencies. This info belongs on the Shuttle,
/// not the Course.
#[derive(Debug, Clone, Serialize, Deserialize, Default, schemars::JsonSchema)]
pub struct ShuttleDetails {
    /// LLM provider (e.g., "anthropic", "openai")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    
    /// Model used (e.g., "claude-sonnet-4", "gpt-4o")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    /// Latency in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    
    /// Token usage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
    
    /// Number of weave units processed
    #[serde(default)]
    pub unit_count: usize,
    
    /// Number of entities spawned
    #[serde(default)]
    pub spawn_count: usize,
}

impl ShuttleDetails {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
    
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    pub fn set_latency(&mut self, ms: u64) {
        self.latency_ms = Some(ms);
    }
    
    pub fn set_tokens(&mut self, tokens: u32) {
        self.tokens_used = Some(tokens);
    }
    
    pub fn set_unit_count(&mut self, count: usize) {
        self.unit_count = count;
    }
    
    pub fn set_spawn_count(&mut self, count: usize) {
        self.spawn_count = count;
    }
}

// ============================================================================
// Shuttle (The Transient Unit of Work)
// ============================================================================

/// A Shuttle is the transient unit of work in the Loom Pattern
/// 
/// Each user message creates a new Shuttle:
/// 1. Shuttle receives the Weave (user input)
/// 2. Shuttle segments the Weave into WeaveUnits
/// 3. Shuttle processes through the Fates pipeline
/// 4. Shuttle commits results to Course history
/// 5. Shuttle is marked complete (persisted only for debugging)
/// 
/// The Course (history) is immutable during processing.
/// Only the Shuttle state changes as work progresses.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Shuttle {
    /// Unique identifier for this shuttle
    pub id: UUID,
    
    /// Reference to the parent Course (the tether to history)
    pub course_id: UUID,
    
    /// Processing status
    pub status: ShuttleStatus,
    
    /// The specific message being processed (the "cargo")
    pub weave: Weave,
    
    /// Processing metadata (provider, model, latency)
    pub details: ShuttleDetails,
    
    /// The weave units (segments) being carried
    #[serde(default)]
    pub units: Vec<WeaveUnit>,
    
    /// The final response (before committing to Course)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    
    /// When this shuttle was created
    pub created_at: Timestamp,
    
    /// When this shuttle started processing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<Timestamp>,
    
    /// When processing completed (or failed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,
    
    /// Error message if shuttle failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Shuttle {
    /// Create a new shuttle for a course with a weave
    pub fn new(course_id: UUID, weave: impl Into<Weave>) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            status: ShuttleStatus::Pending,
            weave: weave.into(),
            details: ShuttleDetails::new(),
            units: vec![],
            response: None,
            created_at: Timestamp::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
    
    /// Set context on the weave
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.weave = self.weave.with_context(context);
        self
    }
    
    /// Set provider and model in details
    pub fn with_provider(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.details = self.details.with_provider(provider).with_model(model);
        self
    }

    /// Add a weave unit to the shuttle
    pub fn add_unit(&mut self, content: impl Into<String>) -> &mut WeaveUnit {
        let index = self.units.len();
        self.units.push(WeaveUnit::new(index, content));
        self.units.last_mut().unwrap()
    }

    /// Add a weave unit with threads, temporal marker, and classifications
    pub fn add_unit_with_details(
        &mut self,
        content: impl Into<String>,
        primary_thread: Option<String>,
        secondary_threads: Vec<String>,
        temporal_marker: Option<String>,
        classifications: Vec<(HeddleEntityType, f64)>,
    ) -> Result<&mut WeaveUnit, String> {
        let index = self.units.len();
        let mut unit = WeaveUnit::new(index, content);
        
        if let Some(thread) = primary_thread {
            unit = unit.with_primary_thread(thread);
        }
        
        if !secondary_threads.is_empty() {
            unit = unit.with_secondary_threads(secondary_threads);
        }
        
        if let Some(marker) = temporal_marker {
            unit = unit.with_temporal_marker(marker);
        }
        
        for (entity_type, weight) in classifications {
            unit.add_classification(entity_type, weight)?;
        }
        
        self.units.push(unit);
        Ok(self.units.last_mut().unwrap())
    }
    
    /// Start processing (receiving -> segmenting)
    pub fn start_processing(&mut self) {
        self.status = ShuttleStatus::Classifying;
        self.started_at = Some(Timestamp::now());
    }

    /// Start classifying
    pub fn start_classifying(&mut self) {
        self.status = ShuttleStatus::Classifying;
    }

    /// Start spawning entities
    pub fn start_spawning(&mut self) {
        self.status = ShuttleStatus::Spawning;
        self.details.set_unit_count(self.units.len());
    }

    /// Mark as complete with response
    pub fn complete(&mut self, response: impl Into<String>) {
        self.response = Some(response.into());
        self.status = ShuttleStatus::Complete;
        self.completed_at = Some(Timestamp::now());
        self.details.set_spawn_count(self.total_spawned());
        
        // Calculate latency if we have started_at
        if let Some(started) = &self.started_at {
            if let Some(completed) = &self.completed_at {
                let latency = (*completed - *started).num_milliseconds() as u64;
                self.details.set_latency(latency);
            }
        }
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = ShuttleStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(Timestamp::now());
    }

    /// Check if processing is complete (success or failure)
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
    
    /// Check if shuttle completed successfully
    pub fn is_complete(&self) -> bool {
        self.status == ShuttleStatus::Complete
    }

    /// Get the number of weave units
    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    /// Get total spawned entities across all units
    pub fn total_spawned(&self) -> usize {
        self.units.iter()
            .map(|u| u.spawned_entity_ids.len())
            .sum()
    }

    /// Check if all units have been classified
    pub fn all_classified(&self) -> bool {
        !self.units.is_empty() && 
        self.units.iter().all(|u| u.is_classified())
    }

    /// Get a mutable reference to a unit by index
    pub fn get_unit_mut(&mut self, index: usize) -> Option<&mut WeaveUnit> {
        self.units.get_mut(index)
    }

    /// Iterate over units
    pub fn units(&self) -> impl Iterator<Item = &WeaveUnit> {
        self.units.iter()
    }

    /// Iterate over units mutably
    pub fn units_mut(&mut self) -> impl Iterator<Item = &mut WeaveUnit> {
        self.units.iter_mut()
    }
    
    /// Get the raw input from the weave
    pub fn raw_input(&self) -> &str {
        &self.weave.raw_content
    }
    
    /// Get the context from the weave
    pub fn context(&self) -> Option<&str> {
        self.weave.context.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuttle_creation() {
        let course_id = UUID::new();
        let shuttle = Shuttle::new(course_id, "Hello world")
            .with_context("Testing")
            .with_provider("anthropic", "claude-sonnet-4");
        
        assert_eq!(shuttle.course_id, course_id);
        assert_eq!(shuttle.status, ShuttleStatus::Pending);
        assert_eq!(shuttle.raw_input(), "Hello world");
        assert_eq!(shuttle.context(), Some("Testing"));
        assert_eq!(shuttle.details.provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_shuttle_processing() {
        let course_id = UUID::new();
        let mut shuttle = Shuttle::new(course_id, "Test input");
        
        shuttle.start_processing();
        assert_eq!(shuttle.status, ShuttleStatus::Classifying);
        assert!(shuttle.started_at.is_some());
        
        shuttle.add_unit("First segment");
        shuttle.start_spawning();
        assert_eq!(shuttle.unit_count(), 1);
        
        shuttle.complete("Done processing");
        assert!(shuttle.is_complete());
        assert_eq!(shuttle.response, Some("Done processing".to_string()));
        assert!(shuttle.completed_at.is_some());
    }

    #[test]
    fn test_shuttle_failure() {
        let course_id = UUID::new();
        let mut shuttle = Shuttle::new(course_id, "Test input");
        
        shuttle.start_processing();
        shuttle.fail("Something went wrong");
        
        assert!(shuttle.is_terminal());
        assert!(!shuttle.is_complete());
        assert_eq!(shuttle.error, Some("Something went wrong".to_string()));
    }
}
