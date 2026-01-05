//! Impl module for message_role types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MessageRole

// Methods: as_str
impl MessageRole { # [doc = " Get the string representation of the role"] pub fn as_str (& self) -> & 'static str { match self { Self :: System => "system" , Self :: User => "user" , Self :: Assistant => "assistant" , } } }

// Trait impl: Display
impl std :: fmt :: Display for MessageRole { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

