//! Impl module for physics_hints types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PhysicsHints

// Trait impl: Default
impl Default for PhysicsHints { fn default () -> Self { Self { valence : 0.0 , arousal : 0.5 , significance : 0.5 , certainty : 0.5 , reasoning : None , } } }

// Methods: from_emotion, to_vae_position
impl PhysicsHints { # [doc = " Create hints from emotional content"] pub fn from_emotion (emotion : & str , intensity : f64) -> Self { let (valence , arousal) = match emotion . to_lowercase () . as_str () { "happy" | "joy" | "excited" => (0.8 , 0.7) , "sad" | "depressed" => (- 0.7 , 0.3) , "angry" | "frustrated" => (- 0.6 , 0.8) , "calm" | "peaceful" => (0.4 , 0.2) , "anxious" | "worried" => (- 0.4 , 0.7) , "neutral" => (0.0 , 0.5) , _ => (0.0 , 0.5) , } ; Self { valence : valence * intensity , arousal : arousal * intensity , significance : 0.5 , certainty : 0.5 , reasoning : Some (format ! ("Derived from emotion: {}" , emotion)) , } } # [doc = " Convert to VAE position array"] pub fn to_vae_position (& self) -> [f64 ; 3] { [self . valence , self . arousal , self . certainty] } }

