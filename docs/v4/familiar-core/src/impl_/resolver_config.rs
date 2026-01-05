//! Impl module for resolver_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ResolverConfig

// Trait impl: Default
impl Default for ResolverConfig { fn default () -> Self { Self { enable_fuzzy : true , fuzzy_max_distance : 2 , enable_semantic : false , semantic_min_score : 0.85 , } } }

