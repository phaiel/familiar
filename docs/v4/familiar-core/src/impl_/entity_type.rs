//! Impl module for entity_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EntityType

// Methods: as_str, description, spawns_entity, references_entity
impl EntityType { pub fn as_str (& self) -> & 'static str { match self { Self :: Moment => "MOMENT" , Self :: Pulse => "PULSE" , Self :: Thread => "THREAD" , Self :: Bond => "BOND" , Self :: Motif => "MOTIF" , Self :: Filament => "FILAMENT" , Self :: Focus => "FOCUS" , Self :: Intent => "INTENT" , } } # [doc = " Description for prompts"] pub fn description (& self) -> & 'static str { match self { Self :: Moment => "A discrete event/action - WHAT HAPPENED. Uses action verbs: went, did, met, called, visited." , Self :: Pulse => "Internal state/feeling - HOW IT WAS/FELT. Uses state verbs + evaluative: felt, was nice, seemed." , Self :: Thread => "An ongoing narrative, topic, person, or concept being discussed." , Self :: Bond => "A statement about the relationship between two entities." , Self :: Motif => "A recurring external pattern noticed over time." , Self :: Filament => "A recurring internal pattern (habits, tendencies, reactions)." , Self :: Focus => "An active goal or thematic intention being pursued." , Self :: Intent => "A future-oriented task or goal to accomplish." , } } # [doc = " Whether this entity typically spawns new records"] pub fn spawns_entity (& self) -> bool { matches ! (self , Self :: Moment | Self :: Pulse | Self :: Intent | Self :: Bond) } # [doc = " Whether this entity type references existing entities"] pub fn references_entity (& self) -> bool { matches ! (self , Self :: Thread | Self :: Motif | Self :: Filament | Self :: Focus | Self :: Bond) } }

// Trait impl: Display
impl std :: fmt :: Display for EntityType { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

