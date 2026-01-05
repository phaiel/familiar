//! Impl module for cognitive_optics types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CognitiveOptics

// Methods: new
impl CognitiveOptics { pub fn new (emissivity : f64 , albedo : f64 , roughness : f64 , occlusion : f64) -> Result < Self , String > { Ok (Self { emissivity : NormalizedFloat :: new (emissivity) ? , albedo : NormalizedFloat :: new (albedo) ? , roughness : NormalizedFloat :: new (roughness) ? , occlusion : NormalizedFloat :: new (occlusion) ? , }) } }

// Trait impl: Default
impl Default for CognitiveOptics { fn default () -> Self { Self { emissivity : NormalizedFloat :: new (0.0) . unwrap () , albedo : NormalizedFloat :: new (0.5) . unwrap () , roughness : NormalizedFloat :: new (0.5) . unwrap () , occlusion : NormalizedFloat :: new (0.1) . unwrap () , } } }

