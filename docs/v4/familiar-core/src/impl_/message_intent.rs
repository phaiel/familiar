//! Impl module for message_intent types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MessageIntent

// Methods: as_str, expects_response, stores_data
impl MessageIntent { pub fn as_str (& self) -> & 'static str { match self { Self :: Log => "LOG" , Self :: Query => "QUERY" , Self :: Infer => "INFER" , Self :: Reference => "REFERENCE" , Self :: Reflect => "REFLECT" , Self :: Command => "COMMAND" , Self :: Social => "SOCIAL" , } } # [doc = " Does this intent expect data to be returned?"] pub fn expects_response (& self) -> bool { matches ! (self , Self :: Query | Self :: Infer | Self :: Reference | Self :: Reflect) } # [doc = " Does this intent involve storing new data?"] pub fn stores_data (& self) -> bool { matches ! (self , Self :: Log) } }

// Trait impl: Default
impl Default for MessageIntent { fn default () -> Self { Self :: Log } }

// Trait impl: Display
impl std :: fmt :: Display for MessageIntent { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

