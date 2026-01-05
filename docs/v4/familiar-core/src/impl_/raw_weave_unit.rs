//! Impl module for raw_weave_unit types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RawWeaveUnit

// Methods: validate
impl RawWeaveUnit { # [doc = " Validate and convert to a proper WeaveUnit"] pub fn validate (self , index : usize) -> Result < (WeaveUnit , Option < RawPhysicsHint >) , String > { let mut unit = WeaveUnit :: new (index , self . content) . with_purpose (self . purpose) ; if let Some (thread) = self . primary_thread { if ! thread . is_empty () && thread . to_lowercase () != "unknown" { unit = unit . with_primary_thread (thread) ; } } if let Some (threads) = self . secondary_threads { let valid_threads : Vec < String > = threads . into_iter () . filter (| t | ! t . is_empty () && t . to_lowercase () != "unknown") . collect () ; if ! valid_threads . is_empty () { unit = unit . with_secondary_threads (valid_threads) ; } } if let Some (marker) = self . temporal_marker { if ! marker . is_empty () { unit = unit . with_temporal_marker (marker) ; } } for raw_class in self . classifications { unit . add_classification (raw_class . entity_type , raw_class . weight) ? ; } Ok ((unit , self . physics_hint)) } }

