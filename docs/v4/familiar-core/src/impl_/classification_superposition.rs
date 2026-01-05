//! Impl module for classification_superposition types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ClassificationSuperposition

// Methods: new, add, get_collapsible, dominant
impl ClassificationSuperposition { pub fn new () -> Self { Self { classifications : Vec :: new () } } pub fn add (& mut self , entity_type : HeddleEntityType , weight : f64) -> Result < () , String > { self . classifications . push (WeightedClassification :: new (entity_type , weight) ?) ; Ok (()) } # [doc = " Get all classifications that exceed the collapse threshold"] pub fn get_collapsible (& self , threshold : f64) -> Vec < & WeightedClassification > { self . classifications . iter () . filter (| c | c . should_collapse (threshold)) . collect () } # [doc = " Get the dominant (highest weight) classification"] pub fn dominant (& self) -> Option < & WeightedClassification > { self . classifications . iter () . max_by (| a , b | a . weight . value () . partial_cmp (& b . weight . value ()) . unwrap ()) } }

