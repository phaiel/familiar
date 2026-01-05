//! Impl module for db_entity_table types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbEntityTable

// Methods: as_str
impl DbEntityTable { pub fn as_str (& self) -> & 'static str { match self { Self :: Pulse => "Pulse" , Self :: Thread => "Thread" , Self :: Bond => "Bond" , Self :: Moment => "Moment" , Self :: Intent => "Intent" , Self :: Focus => "Focus" , Self :: Motif => "Motif" , Self :: Filament => "Filament" , } } }

// Trait impl: Display
impl std :: fmt :: Display for DbEntityTable { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

