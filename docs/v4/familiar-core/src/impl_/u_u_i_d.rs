//! Impl module for u_u_i_d types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for UUID

// Methods: new, parse, as_uuid, from_uuid
impl UUID { pub fn new () -> Self { Self (uuid :: Uuid :: new_v4 ()) } pub fn parse (s : & str) -> Result < Self , uuid :: Error > { Ok (Self (uuid :: Uuid :: parse_str (s) ?)) } # [doc = " Get the inner uuid::Uuid for database operations"] pub fn as_uuid (& self) -> uuid :: Uuid { self . 0 } # [doc = " Create from a uuid::Uuid (for database results)"] pub fn from_uuid (uuid : uuid :: Uuid) -> Self { Self (uuid) } }

// Trait impl: Default
impl Default for UUID { fn default () -> Self { Self :: new () } }

// Trait impl: Display
impl fmt :: Display for UUID { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "{}" , self . 0) } }

