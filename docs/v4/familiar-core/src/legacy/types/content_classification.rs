//! Phase 3: Content Classification
//!
//! Classifies pre-segmented content with purpose and entity type per segment.

use serde::{Deserialize, Serialize};
use crate::types::{HeddleEntityType, MessageIntent};

/// Classification for a single segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentClassification {
    /// Index of the segment being classified
    pub segment_index: usize,
    /// Purpose of this specific segment (LOG, QUERY, COMMAND, etc.)
    /// Only LOG segments spawn entities
    #[serde(default)]
    pub purpose: MessageIntent,
    /// The entity type (MOMENT, PULSE, INTENT) - relevant for LOG purpose
    pub entity_type: HeddleEntityType,
    /// Confidence weight (0.0 to 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Physics hints for entity spawning (only for LOG purpose)
    #[serde(default)]
    pub physics: Option<ClassificationPhysics>,
}

fn default_weight() -> f64 { 1.0 }

/// Physics hints extracted during classification
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassificationPhysics {
    /// Emotional valence: -1.0 (negative) to 1.0 (positive)
    #[serde(default)]
    pub valence: Option<f64>,
    /// Activation level: 0.0 (calm) to 1.0 (activated)
    #[serde(default)]
    pub arousal: Option<f64>,
    /// How significant/important: 0.0 to 1.0
    #[serde(default)]
    pub significance: Option<f64>,
    /// How clear/specific: 0.0 (vague) to 1.0 (clear)
    #[serde(default)]
    pub clarity: Option<f64>,
}

/// Phase 3 response: Classifications for each segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContentClassificationResponse {
    /// Classifications for each segment
    pub classifications: Vec<SegmentClassification>,
}

impl ContentClassificationResponse {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let json = extract_json(json);
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse classification response: {}", e))
    }
    
    pub fn example_json() -> &'static str {
        r#"{
  "classifications": [
    {
      "segment_index": 0,
      "purpose": "LOG",
      "entity_type": "MOMENT",
      "weight": 1.0,
      "physics": { "valence": 0.0, "arousal": 0.5, "clarity": 1.0 }
    },
    {
      "segment_index": 1,
      "purpose": "LOG",
      "entity_type": "PULSE",
      "weight": 1.0,
      "physics": { "valence": -0.3, "arousal": 0.2, "clarity": 0.8 }
    },
    {
      "segment_index": 2,
      "purpose": "QUERY",
      "entity_type": "MOMENT",
      "weight": 1.0
    }
  ]
}"#
    }
}

fn extract_json(content: &str) -> &str {
    let json_start = content.find('{').unwrap_or(0);
    let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
    &content[json_start..json_end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_classification() {
        let json = r#"{
            "classifications": [
                { "segment_index": 0, "purpose": "LOG", "entity_type": "MOMENT", "weight": 1.0 }
            ]
        }"#;
        
        let response = ContentClassificationResponse::from_json(json).unwrap();
        assert_eq!(response.classifications.len(), 1);
        assert_eq!(response.classifications[0].entity_type, HeddleEntityType::Moment);
        assert_eq!(response.classifications[0].purpose, MessageIntent::Log);
    }

    #[test]
    fn test_parse_mixed_purpose() {
        let json = r#"{
            "classifications": [
                { "segment_index": 0, "purpose": "LOG", "entity_type": "MOMENT", "weight": 1.0 },
                { "segment_index": 1, "purpose": "QUERY", "entity_type": "MOMENT", "weight": 1.0 }
            ]
        }"#;
        
        let response = ContentClassificationResponse::from_json(json).unwrap();
        assert_eq!(response.classifications.len(), 2);
        assert_eq!(response.classifications[0].purpose, MessageIntent::Log);
        assert_eq!(response.classifications[1].purpose, MessageIntent::Query);
    }
}

