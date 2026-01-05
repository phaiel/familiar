//! Impl module for emotional_state types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EmotionalState

// Methods: new, joy, anger, fear, sadness
impl EmotionalState { pub fn new (v : f64 , a : f64 , d : f64) -> Result < Self , String > { Ok (Self { valence : SignedNormalizedFloat :: new (v) ? , arousal : SignedNormalizedFloat :: new (a) ? , dominance : SignedNormalizedFloat :: new (d) ? , }) } pub fn joy () -> Self { Self :: new (0.8 , 0.6 , 0.4) . unwrap () } pub fn anger () -> Self { Self :: new (- 0.6 , 0.8 , 0.5) . unwrap () } pub fn fear () -> Self { Self :: new (- 0.8 , 0.9 , - 0.6) . unwrap () } pub fn sadness () -> Self { Self :: new (- 0.6 , - 0.4 , - 0.3) . unwrap () } }

// Trait impl: Default
impl Default for EmotionalState { fn default () -> Self { Self :: new (0.0 , 0.0 , 0.0) . unwrap () } }

