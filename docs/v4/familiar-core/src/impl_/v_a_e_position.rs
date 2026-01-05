//! Impl module for v_a_e_position types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for VAEPosition

// Methods: from_floats, to_floats
impl VAEPosition { # [doc = " Scale factor for quantization (1M = 1.0)"] pub const SCALE : i64 = 1_000_000 ; # [doc = " Create from float values"] pub fn from_floats (valence : f64 , arousal : f64 , epistemic : f64) -> Self { Self { x : (valence . clamp (- 1.0 , 1.0) * Self :: SCALE as f64) as i64 , y : (arousal . clamp (0.0 , 1.0) * Self :: SCALE as f64) as i64 , z : (epistemic . clamp (0.0 , 1.0) * Self :: SCALE as f64) as i64 , } } # [doc = " Convert to float array"] pub fn to_floats (& self) -> [f64 ; 3] { [self . x as f64 / Self :: SCALE as f64 , self . y as f64 / Self :: SCALE as f64 , self . z as f64 / Self :: SCALE as f64 ,] } }

// Trait impl: From
impl From < & PhysicsHintValues > for VAEPosition { fn from (hints : & PhysicsHintValues) -> Self { Self :: from_floats (hints . valence , hints . arousal , hints . epistemic) } }

