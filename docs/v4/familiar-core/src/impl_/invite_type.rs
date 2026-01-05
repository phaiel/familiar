//! Impl module for invite_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for InviteType

// Methods: as_str
impl InviteType { pub fn as_str (& self) -> & 'static str { match self { Self :: Email => "email" , Self :: Code => "code" , } } }

