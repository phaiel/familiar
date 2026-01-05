//! Impl module for familiar_entity_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for FamiliarEntityType

// Methods: as_str
impl FamiliarEntityType { pub fn as_str (& self) -> & 'static str { match self { Self :: Moment => "MOMENT" , Self :: Pulse => "PULSE" , Self :: Intent => "INTENT" , Self :: Thread => "THREAD" , Self :: Bond => "BOND" , Self :: Motif => "MOTIF" , Self :: Filament => "FILAMENT" , Self :: Focus => "FOCUS" , } } }

