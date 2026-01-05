//! Impl module for db_pool_size types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbPoolSize

// Methods: new, value
impl DbPoolSize { pub const MIN : u32 = 1 ; pub const MAX : u32 = 100 ; pub const DEFAULT : u32 = 5 ; pub fn new (size : u32) -> Result < Self , String > { if size < Self :: MIN || size > Self :: MAX { return Err (format ! ("Pool size {} must be between {} and {}" , size , Self :: MIN , Self :: MAX)) ; } Ok (Self (size)) } pub fn value (& self) -> u32 { self . 0 } }

// Trait impl: Default
impl Default for DbPoolSize { fn default () -> Self { Self (Self :: DEFAULT) } }

