//! Phase 1: Agnostic Segmentation
//!
//! Pure semantic chunking with NO classification.
//! Just break text into atomic semantic units.

use serde::{Deserialize, Serialize};

/// A raw segment from Phase 1 - NO classification, just text
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RawSegment {
    /// The extracted semantic content
    pub content: String,
    /// Who/what is the primary subject? (for context, not classification)
    #[serde(default)]
    pub subject: Option<String>,
    /// Other entities/things mentioned
    #[serde(default)]
    pub mentions: Vec<String>,
    /// Any temporal reference found
    #[serde(default)]
    pub temporal: Option<String>,
}

/// Phase 1 response: Just segments, no classification
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationResponse {
    /// The semantic segments extracted from the input
    pub segments: Vec<RawSegment>,
}

impl SegmentationResponse {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let json = extract_json(json);
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse segmentation response: {}", e))
    }
    
    pub fn example_json() -> &'static str {
        r#"{
  "segments": [
    {
      "content": "had a meeting",
      "subject": "user",
      "mentions": ["someone"],
      "temporal": "1pm"
    },
    {
      "content": "feeling tired",
      "subject": "user",
      "mentions": [],
      "temporal": "today"
    }
  ]
}"#
    }
}

/// Extract JSON from markdown-wrapped responses
fn extract_json(content: &str) -> &str {
    let json_start = content.find('{').unwrap_or(0);
    let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
    &content[json_start..json_end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_segmentation() {
        let json = r#"{
            "segments": [
                { "content": "test segment", "subject": "user", "mentions": [], "temporal": null }
            ]
        }"#;
        
        let response = SegmentationResponse::from_json(json).unwrap();
        assert_eq!(response.segments.len(), 1);
        assert_eq!(response.segments[0].content, "test segment");
    }
}

