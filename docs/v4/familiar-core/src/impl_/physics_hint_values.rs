//! Impl module for physics_hint_values types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PhysicsHintValues

// Trait impl: Default
impl Default for PhysicsHintValues { fn default () -> Self { Self { valence : 0.0 , arousal : 0.5 , epistemic : 0.5 , significance : 0.5 , energy : 0.5 , temperature : 0.5 , } } }

// Methods: from_emotion, for_entity_type
impl PhysicsHintValues { # [doc = " Create from simple emotional descriptors"] pub fn from_emotion (emotion : & str , intensity : f64) -> Self { let intensity = intensity . clamp (0.0 , 1.0) ; let (valence , arousal , energy) = match emotion . to_lowercase () . as_str () { "excited" | "elated" | "thrilled" => (0.9 * intensity , 0.9 * intensity , 0.8) , "happy" | "joyful" | "pleased" => (0.8 * intensity , 0.6 * intensity , 0.6) , "proud" | "accomplished" => (0.7 * intensity , 0.5 * intensity , 0.6) , "calm" | "peaceful" | "serene" => (0.5 * intensity , 0.2 * intensity , 0.3) , "content" | "satisfied" => (0.6 * intensity , 0.3 * intensity , 0.4) , "relaxed" => (0.4 * intensity , 0.2 * intensity , 0.3) , "angry" | "furious" | "outraged" => (- 0.8 * intensity , 0.9 * intensity , 0.9) , "frustrated" | "annoyed" => (- 0.5 * intensity , 0.7 * intensity , 0.7) , "anxious" | "worried" | "stressed" => (- 0.4 * intensity , 0.8 * intensity , 0.7) , "scared" | "afraid" | "terrified" => (- 0.7 * intensity , 0.9 * intensity , 0.8) , "sad" | "depressed" | "down" => (- 0.7 * intensity , 0.2 * intensity , 0.3) , "tired" | "exhausted" => (- 0.3 * intensity , 0.1 * intensity , 0.2) , "bored" | "apathetic" => (- 0.2 * intensity , 0.1 * intensity , 0.2) , "lonely" | "isolated" => (- 0.5 * intensity , 0.2 * intensity , 0.3) , "neutral" | "normal" | "okay" => (0.0 , 0.5 , 0.5) , "curious" | "interested" => (0.3 * intensity , 0.6 * intensity , 0.6) , "surprised" | "shocked" => (0.0 , 0.8 * intensity , 0.7) , _ => (0.0 , 0.5 , 0.5) , } ; Self { valence , arousal , epistemic : 0.5 , significance : intensity * 0.7 , energy , temperature : arousal * 0.8 , } } # [doc = " Create from entity type defaults"] pub fn for_entity_type (entity_type : EntityType) -> Self { match entity_type { EntityType :: Moment => Self { valence : 0.0 , arousal : 0.5 , epistemic : 0.8 , significance : 0.5 , energy : 0.5 , temperature : 0.3 , } , EntityType :: Pulse => Self { valence : 0.0 , arousal : 0.5 , epistemic : 0.6 , significance : 0.6 , energy : 0.6 , temperature : 0.6 , } , EntityType :: Intent => Self { valence : 0.3 , arousal : 0.6 , epistemic : 0.4 , significance : 0.7 , energy : 0.7 , temperature : 0.4 , } , EntityType :: Thread => Self { valence : 0.0 , arousal : 0.3 , epistemic : 0.7 , significance : 0.8 , energy : 0.3 , temperature : 0.2 , } , EntityType :: Bond => Self { valence : 0.0 , arousal : 0.4 , epistemic : 0.6 , significance : 0.8 , energy : 0.4 , temperature : 0.3 , } , EntityType :: Motif | EntityType :: Filament => Self { valence : 0.0 , arousal : 0.4 , epistemic : 0.7 , significance : 0.7 , energy : 0.4 , temperature : 0.2 , } , EntityType :: Focus => Self { valence : 0.4 , arousal : 0.6 , epistemic : 0.5 , significance : 0.9 , energy : 0.7 , temperature : 0.5 , } , } } }

