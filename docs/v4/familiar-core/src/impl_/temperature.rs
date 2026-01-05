//! Impl module for temperature types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Temperature

// Methods: new, value
impl Temperature { pub const MIN : f32 = 0.0 ; pub const MAX : f32 = 2.0 ; # [doc = " Default for classification tasks (low randomness)"] pub const CLASSIFICATION : Self = Self (0.3) ; # [doc = " Default for creative tasks"] pub const CREATIVE : Self = Self (0.9) ; # [doc = " Completely deterministic"] pub const DETERMINISTIC : Self = Self (0.0) ; pub fn new (value : f32) -> Result < Self , String > { if value < Self :: MIN || value > Self :: MAX { return Err (format ! ("Temperature {} must be between {} and {}" , value , Self :: MIN , Self :: MAX)) ; } Ok (Self (value)) } pub fn value (& self) -> f32 { self . 0 } }

// Trait impl: Default
impl Default for Temperature { fn default () -> Self { Self :: CLASSIFICATION } }

