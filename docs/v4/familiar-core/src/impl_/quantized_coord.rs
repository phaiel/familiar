//! Impl module for quantized_coord types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for QuantizedCoord

// Methods: new, value, from_normalized, from_f64, to_normalized, zero
impl QuantizedCoord { # [doc = " Scale factor for converting normalized floats to quantized coords"] pub const SCALE : i64 = 1_000_000 ; pub fn new (value : i64) -> Result < Self , String > { Ok (Self (value)) } pub fn value (& self) -> i64 { self . 0 } # [doc = " Create from a normalized float (-1.0 to 1.0)"] pub fn from_normalized (value : f64) -> Self { let clamped = value . clamp (- 1.0 , 1.0) ; Self ((clamped * Self :: SCALE as f64) as i64) } # [doc = " Create from a regular f64"] pub fn from_f64 (value : f64) -> Self { Self ((value * Self :: SCALE as f64) as i64) } # [doc = " Convert back to normalized float"] pub fn to_normalized (& self) -> f64 { self . 0 as f64 / Self :: SCALE as f64 } # [doc = " Zero coordinate"] pub fn zero () -> Self { Self (0) } }

// Trait impl: Default
impl Default for QuantizedCoord { fn default () -> Self { Self :: zero () } }

// Trait impl: Add
impl std :: ops :: Add for QuantizedCoord { type Output = Self ; fn add (self , other : Self) -> Self { Self (self . 0 + other . 0) } }

// Trait impl: Sub
impl std :: ops :: Sub for QuantizedCoord { type Output = Self ; fn sub (self , other : Self) -> Self { Self (self . 0 - other . 0) } }

