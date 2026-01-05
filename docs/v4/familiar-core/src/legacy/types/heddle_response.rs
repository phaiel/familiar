//! Heddle Response Schema
//!
//! Schema-driven types for LLM output. These are the "raw" types that can be
//! deserialized directly from LLM JSON output, then validated and converted
//! to proper WeaveUnits.
//!
//! The Heddle classifies:
//! 1. Message Intent - WHAT is the user trying to do? (LOG, QUERY, INFER, etc.)
//! 2. Content - If logging, WHAT are they logging? (MOMENT, PULSE, INTENT entities)

use serde::{Deserialize, Serialize};

use crate::types::{HeddleEntityType, MessageIntent, QueryType, QueryTarget};
use crate::components::WeaveUnit;

// ============================================================================
// Raw types (for LLM parsing - uses simple f64 instead of validated types)
// ============================================================================

/// Raw classification from LLM output
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RawClassification {
    pub entity_type: HeddleEntityType,
    pub weight: f64,
}

/// Raw weave unit from LLM output (before validation)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RawWeaveUnit {
    /// The extracted/cleaned text content for this weave unit
    pub content: String,
    
    /// Purpose of this specific unit (LOG, QUERY, COMMAND, etc.)
    /// Only LOG units spawn entities
    #[serde(default)]
    pub purpose: MessageIntent,
    
    /// Primary thread: the main subject/actor of this unit
    #[serde(default)]
    pub primary_thread: Option<String>,
    
    /// Secondary threads: other people/places/things mentioned
    #[serde(default)]
    pub secondary_threads: Option<Vec<String>>,
    
    /// Temporal marker: when this happened (absolute, relative, or frequency)
    /// Examples: "6pm", "today", "yesterday", "once per hour", "every morning"
    #[serde(default)]
    pub temporal_marker: Option<String>,
    
    /// Classifications in superposition
    pub classifications: Vec<RawClassification>,
    
    /// Physics hints (passed to spawned entities, not stored on WeaveUnit)
    #[serde(default)]
    pub physics_hint: Option<RawPhysicsHint>,
}

/// Raw physics hints from LLM (will be applied to spawned entities)
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RawPhysicsHint {
    #[serde(default)]
    pub valence: Option<f64>,
    #[serde(default)]
    pub arousal: Option<f64>,
    #[serde(default)]
    pub significance: Option<f64>,
    #[serde(default)]
    pub clarity: Option<f64>,
    #[serde(default)]
    pub intrusiveness: Option<f64>,
    #[serde(default)]
    pub volatility: Option<f64>,
}

/// Raw message intent classification from LLM
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RawMessageIntent {
    /// Primary intent (LOG, QUERY, INFER, REFERENCE, REFLECT, COMMAND, SOCIAL)
    pub intent: MessageIntent,
    /// Confidence in classification (0.0 to 1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// If QUERY: what type of query
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_type: Option<QueryType>,
    /// If QUERY: what entities/threads are being queried
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_target: Option<QueryTarget>,
}

fn default_confidence() -> f64 { 1.0 }

impl Default for RawMessageIntent {
    fn default() -> Self {
        Self {
            intent: MessageIntent::Log,
            confidence: 1.0,
            query_type: None,
            query_target: None,
        }
    }
}

/// The full response from the Heddle LLM
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HeddleResponse {
    /// Message intent classification (WHAT is user trying to do?)
    #[serde(default)]
    pub message_intent: RawMessageIntent,
    /// Array of weave units extracted from the input (for LOG intent)
    /// May be empty for QUERY/REFERENCE intents
    #[serde(default)]
    pub weave_units: Vec<RawWeaveUnit>,
}

// ============================================================================
// Validation and conversion
// ============================================================================

impl RawWeaveUnit {
    /// Validate and convert to a proper WeaveUnit
    pub fn validate(self, index: usize) -> Result<(WeaveUnit, Option<RawPhysicsHint>), String> {
        let mut unit = WeaveUnit::new(index, self.content)
            .with_purpose(self.purpose);
        
        if let Some(thread) = self.primary_thread {
            if !thread.is_empty() && thread.to_lowercase() != "unknown" {
                unit = unit.with_primary_thread(thread);
            }
        }
        
        if let Some(threads) = self.secondary_threads {
            let valid_threads: Vec<String> = threads.into_iter()
                .filter(|t| !t.is_empty() && t.to_lowercase() != "unknown")
                .collect();
            if !valid_threads.is_empty() {
                unit = unit.with_secondary_threads(valid_threads);
            }
        }
        
        if let Some(marker) = self.temporal_marker {
            if !marker.is_empty() {
                unit = unit.with_temporal_marker(marker);
            }
        }
        
        for raw_class in self.classifications {
            unit.add_classification(raw_class.entity_type, raw_class.weight)?;
        }
        
        Ok((unit, self.physics_hint))
    }
}

impl HeddleResponse {
    /// Get the message intent
    pub fn intent(&self) -> MessageIntent {
        self.message_intent.intent
    }
    
    /// Check if this is a query/lookup type message
    pub fn is_query(&self) -> bool {
        self.message_intent.intent.expects_response()
    }
    
    /// Check if this is a logging type message
    pub fn is_log(&self) -> bool {
        self.message_intent.intent.stores_data()
    }
    
    /// Get query type if this is a QUERY intent
    pub fn query_type(&self) -> Option<QueryType> {
        self.message_intent.query_type
    }
    
    /// Get query target if this is a QUERY intent
    pub fn query_target(&self) -> Option<&QueryTarget> {
        self.message_intent.query_target.as_ref()
    }
    
    /// Validate all weave units and convert to proper WeaveUnit types
    /// Returns (weave_units, physics_hints) - physics are separate because they go to spawned entities
    pub fn validate(self) -> Result<(Vec<WeaveUnit>, Vec<Option<RawPhysicsHint>>, RawMessageIntent), String> {
        let mut units = Vec::new();
        let mut physics = Vec::new();
        
        for (idx, raw) in self.weave_units.into_iter().enumerate() {
            let (unit, phys) = raw.validate(idx)?;
            units.push(unit);
            physics.push(phys);
        }
        
        // Only require weave_units for LOG intent
        // QUERY/REFERENCE/etc. may have empty weave_units
        if units.is_empty() && self.message_intent.intent == MessageIntent::Log {
            return Err("No weave_units in Heddle response for LOG intent".to_string());
        }
        
        Ok((units, physics, self.message_intent))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        // Try to extract JSON if wrapped in markdown
        let json = extract_json(json);
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse Heddle response: {}", e))
    }
}

/// Extract JSON from a response that may contain surrounding text/markdown
fn extract_json(content: &str) -> &str {
    // Find JSON object boundaries
    let json_start = content.find('{').unwrap_or(0);
    let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
    &content[json_start..json_end]
}

// ============================================================================
// JSON examples for LLM prompts
// ============================================================================

impl HeddleResponse {
    /// Generate a simplified JSON example for LLM prompts
    /// Only MOMENT, PULSE, INTENT are valid entity types for LOG
    pub fn example_json() -> &'static str {
        r#"For mixed LOG and QUERY messages:
{
  "message_intent": { "intent": "LOG", "confidence": 0.7 },
  "weave_units": [
    {
      "content": "meeting with someone",
      "purpose": "LOG",
      "primary_thread": "user",
      "secondary_threads": ["person name"],
      "temporal_marker": "1pm",
      "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }],
      "physics_hint": { "valence": 0.0, "arousal": 0.5, "clarity": 1.0 }
    },
    {
      "content": "feeling some way",
      "purpose": "LOG",
      "primary_thread": "user",
      "secondary_threads": [],
      "temporal_marker": "today",
      "classifications": [{ "entity_type": "PULSE", "weight": 1.0 }],
      "physics_hint": { "valence": -0.3, "arousal": 0.7, "clarity": 1.0 }
    },
    {
      "content": "when is next meeting",
      "purpose": "QUERY",
      "primary_thread": "user",
      "secondary_threads": ["meeting"],
      "temporal_marker": null,
      "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }]
    }
  ]
}

PURPOSE (per weave_unit):
- LOG: Recording information - spawns entities
- QUERY: Asking for information - no entity spawn
- COMMAND: Requesting an action - no entity spawn
- INFER: Seeking insight - no entity spawn
- REFERENCE: Looking up existing data - no entity spawn

ENTITY TYPES (for LOG purpose):
- MOMENT: A discrete event/action
- PULSE: An internal state/feeling
- INTENT: A future/planned action

message_intent is the DOMINANT purpose across all units.
Each weave_unit has its OWN purpose.

⚠️ temporal_marker examples: "today", "tonight", "1pm", "tomorrow", "yesterday", "now", "this morning"
⚠️ EVERY weave_unit should have temporal_marker if ANY time word exists
⚠️ Only LOG purpose units need physics_hint"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heddle_response_log() {
        let json = r#"{
            "message_intent": { "intent": "LOG", "confidence": 1.0 },
            "weave_units": [
                {
                    "content": "Test weave unit",
                    "primary_thread": "someone",
                    "secondary_threads": ["place"],
                    "classifications": [
                        { "entity_type": "MOMENT", "weight": 0.9 }
                    ],
                    "physics_hint": { "valence": 0.5 }
                }
            ]
        }"#;

        let response = HeddleResponse::from_json(json).unwrap();
        assert_eq!(response.weave_units.len(), 1);
        assert_eq!(response.weave_units[0].content, "Test weave unit");
        assert!(response.is_log());
        
        let (units, physics, intent) = response.validate().unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].content, "Test weave unit");
        assert_eq!(units[0].primary_thread, Some("someone".to_string()));
        assert_eq!(units[0].secondary_threads, vec!["place".to_string()]);
        assert!(physics[0].is_some());
        assert_eq!(intent.intent, MessageIntent::Log);
    }

    #[test]
    fn test_parse_heddle_response_query() {
        let json = r#"{
            "message_intent": {
                "intent": "QUERY",
                "confidence": 0.95,
                "query_type": "TEMPORAL",
                "query_target": {
                    "entity_types": ["moments"],
                    "thread_hints": ["john"],
                    "temporal_scope": "last week",
                    "keywords": ["meeting"]
                }
            },
            "weave_units": []
        }"#;

        let response = HeddleResponse::from_json(json).unwrap();
        assert!(response.is_query());
        assert_eq!(response.query_type(), Some(QueryType::Temporal));
        
        let target = response.query_target().unwrap();
        assert_eq!(target.entity_types, vec!["moments".to_string()]);
        assert_eq!(target.thread_hints, vec!["john".to_string()]);
        assert_eq!(target.temporal_scope, Some("last week".to_string()));
        
        let (units, _, intent) = response.validate().unwrap();
        assert!(units.is_empty()); // Queries can have empty weave_units
        assert_eq!(intent.intent, MessageIntent::Query);
    }

    #[test]
    fn test_multiple_threads() {
        let json = r#"{
            "message_intent": { "intent": "LOG" },
            "weave_units": [
                {
                    "content": "Subject did action at location with companion",
                    "primary_thread": "Subject",
                    "secondary_threads": ["companion", "location"],
                    "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }]
                },
                {
                    "content": "Person experienced state",
                    "primary_thread": "Person",
                    "secondary_threads": [],
                    "classifications": [{ "entity_type": "PULSE", "weight": 1.0 }]
                },
                {
                    "content": "Subject plans to do something",
                    "primary_thread": "Subject",
                    "secondary_threads": [],
                    "classifications": [{ "entity_type": "INTENT", "weight": 1.0 }]
                }
            ]
        }"#;
        
        let response = HeddleResponse::from_json(json).unwrap();
        assert_eq!(response.weave_units.len(), 3);
        
        let (units, _, _) = response.validate().unwrap();
        assert_eq!(units[0].primary_thread, Some("Subject".to_string()));
        assert_eq!(units[0].secondary_threads, vec!["companion".to_string(), "location".to_string()]);
        assert_eq!(units[1].primary_thread, Some("Person".to_string()));
        assert!(units[1].secondary_threads.is_empty());
    }
    
    #[test]
    fn test_unknown_threads_filtered() {
        let json = r#"{
            "message_intent": { "intent": "LOG" },
            "weave_units": [
                {
                    "content": "Someone did something",
                    "primary_thread": "unknown",
                    "secondary_threads": ["unknown", ""],
                    "classifications": [{ "entity_type": "MOMENT", "weight": 0.8 }]
                }
            ]
        }"#;
        
        let response = HeddleResponse::from_json(json).unwrap();
        let (units, _, _) = response.validate().unwrap();
        // "unknown" should be filtered out
        assert!(units[0].primary_thread.is_none());
        assert!(units[0].secondary_threads.is_empty());
    }
    
    #[test]
    fn test_backward_compatible_no_intent() {
        // Old format without message_intent should still work (defaults to LOG)
        let json = r#"{
            "weave_units": [
                {
                    "content": "Test",
                    "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }]
                }
            ]
        }"#;
        
        let response = HeddleResponse::from_json(json).unwrap();
        assert!(response.is_log()); // Default
        let (units, _, intent) = response.validate().unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(intent.intent, MessageIntent::Log);
    }
}

