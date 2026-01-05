//! Impl module for segmentation_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SegmentationConfig

// Trait impl: Default
impl Default for SegmentationConfig { fn default () -> Self { Self { max_segments : default_max_segments () , min_segment_length : None , extract_entities : true , extract_temporal : true , language : None , } } }

