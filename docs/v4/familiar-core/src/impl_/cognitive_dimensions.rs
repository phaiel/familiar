//! Impl module for cognitive_dimensions types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CognitiveDimensions

// Methods: new
impl CognitiveDimensions { pub fn new (o : f64 , c : f64 , e : f64 , a : f64 , n : f64) -> Result < Self , String > { Ok (Self { openness : NormalizedFloat :: new (o) ? , conscientiousness : NormalizedFloat :: new (c) ? , extraversion : NormalizedFloat :: new (e) ? , agreeableness : NormalizedFloat :: new (a) ? , neuroticism : NormalizedFloat :: new (n) ? , }) } }

// Trait impl: Default
impl Default for CognitiveDimensions { fn default () -> Self { Self :: new (0.5 , 0.5 , 0.5 , 0.5 , 0.5) . unwrap () } }

