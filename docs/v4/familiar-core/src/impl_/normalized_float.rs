//! Impl module for normalized_float types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for NormalizedFloat

// Methods: new, value
impl NormalizedFloat { pub fn new (value : f64) -> Result < Self , String > { if value < 0.0 || value > 1.0 { Err (format ! ("Value {} must be between 0.0 and 1.0" , value)) } else { Ok (Self (value)) } } pub fn value (& self) -> f64 { self . 0 } }

// Trait impl: Default
impl Default for NormalizedFloat { fn default () -> Self { Self (0.0) } }

