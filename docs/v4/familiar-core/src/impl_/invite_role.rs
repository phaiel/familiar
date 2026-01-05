//! Impl module for invite_role types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for InviteRole

// Trait impl: Default
impl Default for InviteRole { fn default () -> Self { Self :: Member } }

// Methods: as_str
impl InviteRole { pub fn as_str (& self) -> & 'static str { match self { Self :: Admin => "admin" , Self :: Member => "member" , Self :: Guest => "guest" , } } }

