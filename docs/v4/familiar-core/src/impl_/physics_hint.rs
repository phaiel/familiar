//! Impl module for physics_hint types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PhysicsHint

// Methods: valence_or_vacuum, arousal_or_vacuum, significance_or_vacuum, clarity_or_vacuum, intrusiveness_or_vacuum, volatility_or_vacuum
impl PhysicsHint { # [doc = " Get valence with vacuum state default (neutral)"] pub fn valence_or_vacuum (& self) -> f64 { self . valence . unwrap_or (0.0) } # [doc = " Get arousal with vacuum state default (calm)"] pub fn arousal_or_vacuum (& self) -> f64 { self . arousal . unwrap_or (0.0) } # [doc = " Get significance with vacuum state default (light/fleeting)"] pub fn significance_or_vacuum (& self) -> f64 { self . significance . unwrap_or (0.1) } # [doc = " Get clarity with vacuum state default (foggy/vague)"] pub fn clarity_or_vacuum (& self) -> f64 { self . clarity . unwrap_or (0.1) } # [doc = " Get intrusiveness with vacuum state default (passive)"] pub fn intrusiveness_or_vacuum (& self) -> f64 { self . intrusiveness . unwrap_or (0.0) } # [doc = " Get volatility with vacuum state default (stable)"] pub fn volatility_or_vacuum (& self) -> f64 { self . volatility . unwrap_or (0.1) } }

