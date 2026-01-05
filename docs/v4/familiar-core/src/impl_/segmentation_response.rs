//! Impl module for segmentation_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SegmentationResponse

// Methods: from_json, example_json
impl SegmentationResponse { pub fn from_json (json : & str) -> Result < Self , String > { let json = extract_json (json) ; serde_json :: from_str (json) . map_err (| e | format ! ("Failed to parse segmentation response: {}" , e)) } pub fn example_json () -> & 'static str { r#"{
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
}"# } }

