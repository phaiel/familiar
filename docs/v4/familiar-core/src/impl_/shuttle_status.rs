//! Impl module for shuttle_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ShuttleStatus

// Methods: as_str, is_terminal, is_active
impl ShuttleStatus { pub fn as_str (& self) -> & 'static str { match self { Self :: Pending => "pending" , Self :: Classifying => "classifying" , Self :: Spawning => "spawning" , Self :: Complete => "complete" , Self :: Failed => "failed" , } } pub fn is_terminal (& self) -> bool { matches ! (self , Self :: Complete | Self :: Failed) } pub fn is_active (& self) -> bool { ! self . is_terminal () } }

// Trait impl: Default
impl Default for ShuttleStatus { fn default () -> Self { Self :: Pending } }

// Trait impl: Display
impl std :: fmt :: Display for ShuttleStatus { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

