//! Impl module for weighted_classification types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for WeightedClassification

// Methods: new, should_collapse
impl WeightedClassification { pub fn new (entity_type : HeddleEntityType , weight : f64) -> Result < Self , String > { Ok (Self { entity_type , weight : NormalizedFloat :: new (weight) ? , }) } # [doc = " Check if this classification exceeds the collapse threshold"] pub fn should_collapse (& self , threshold : f64) -> bool { self . weight . value () >= threshold } }

// Trait impl: Component
impl Component for WeightedClassification { }

