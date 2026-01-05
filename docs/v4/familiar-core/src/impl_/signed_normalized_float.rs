//! Impl module for signed_normalized_float types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SignedNormalizedFloat

// Methods: new, new_clamped, value
impl SignedNormalizedFloat { pub fn new (value : f64) -> Result < Self , String > { if ! (- 1.0 ..= 1.0) . contains (& value) { return Err (format ! ("Value {} must be between -1.0 and 1.0" , value)) ; } Ok (Self (value)) } # [doc = " Clamps the value to the valid range"] pub fn new_clamped (value : f64) -> Self { Self (value . clamp (- 1.0 , 1.0)) } pub fn value (& self) -> f64 { self . 0 } }

// Trait impl: Default
impl Default for SignedNormalizedFloat { fn default () -> Self { Self (0.0) } }

