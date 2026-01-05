//! Impl module for tool_purpose types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ToolPurpose

// Methods: as_str, stores_data, expects_response, requires_search, pipeline
impl ToolPurpose { pub fn as_str (& self) -> & 'static str { match self { Self :: Log => "LOG" , Self :: Query => "QUERY" , Self :: Infer => "INFER" , Self :: Reference => "REFERENCE" , Self :: Reflect => "REFLECT" , Self :: Command => "COMMAND" , Self :: Social => "SOCIAL" , Self :: Continuation => "CONTINUATION" , Self :: Correction => "CORRECTION" , } } # [doc = " Does this purpose involve storing new data?"] pub fn stores_data (& self) -> bool { matches ! (self , Self :: Log | Self :: Correction) } # [doc = " Does this purpose expect data to be returned?"] pub fn expects_response (& self) -> bool { matches ! (self , Self :: Query | Self :: Infer | Self :: Reference | Self :: Reflect) } # [doc = " Does this purpose require searching existing data?"] pub fn requires_search (& self) -> bool { matches ! (self , Self :: Query | Self :: Reference | Self :: Reflect | Self :: Infer) } # [doc = " Processing pipeline for this purpose"] pub fn pipeline (& self) -> PurposePipeline { match self { Self :: Log => PurposePipeline :: Recording , Self :: Query | Self :: Reference => PurposePipeline :: Retrieval , Self :: Infer | Self :: Reflect => PurposePipeline :: Analysis , Self :: Command => PurposePipeline :: Action , Self :: Social | Self :: Continuation | Self :: Correction => PurposePipeline :: Conversational , } } }

// Trait impl: Default
impl Default for ToolPurpose { fn default () -> Self { Self :: Log } }

