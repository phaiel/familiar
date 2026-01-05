//! Impl module for spawn_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SpawnConfig

// Trait impl: Default
impl Default for SpawnConfig { fn default () -> Self { Self { high_confidence_threshold : default_high_threshold () , medium_confidence_threshold : default_medium_threshold () , low_confidence_threshold : default_low_threshold () , max_entities_per_segment : default_max_spawn () , auto_spawn_enabled : true , manual_only_types : vec ! [] , } } }

