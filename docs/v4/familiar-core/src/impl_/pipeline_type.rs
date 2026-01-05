//! Impl module for pipeline_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PipelineType

// Methods: tool_categories, from_purpose_pipeline
impl PipelineType { # [doc = " Get the tool categories used in this pipeline"] pub fn tool_categories (& self) -> Vec < ToolCategory > { match self { Self :: MemoryRecording => vec ! [ToolCategory :: Segmentation , ToolCategory :: Classification , ToolCategory :: Spawn , ToolCategory :: Hints ,] , Self :: Query => vec ! [ToolCategory :: Classification , ToolCategory :: Retrieval ,] , Self :: Analysis => vec ! [ToolCategory :: Segmentation , ToolCategory :: Classification , ToolCategory :: Retrieval ,] , Self :: Command => vec ! [ToolCategory :: Classification , ToolCategory :: Orchestration ,] , Self :: Conversation => vec ! [ToolCategory :: Classification ,] , Self :: MultiModal => vec ! [ToolCategory :: Segmentation , ToolCategory :: Classification , ToolCategory :: Spawn , ToolCategory :: Hints ,] , } } # [doc = " Convert from PurposePipeline"] pub fn from_purpose_pipeline (purpose : PurposePipeline) -> Self { match purpose { PurposePipeline :: Recording => Self :: MemoryRecording , PurposePipeline :: Retrieval => Self :: Query , PurposePipeline :: Analysis => Self :: Analysis , PurposePipeline :: Action => Self :: Command , PurposePipeline :: Conversational => Self :: Conversation , } } }

