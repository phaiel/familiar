//! Impl module for content_classification_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ContentClassificationResponse

// Methods: from_json, example_json
impl ContentClassificationResponse { pub fn from_json (json : & str) -> Result < Self , String > { let json = extract_json (json) ; serde_json :: from_str (json) . map_err (| e | format ! ("Failed to parse classification response: {}" , e)) } pub fn example_json () -> & 'static str { r#"{
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
}"# } }

