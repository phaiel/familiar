//! Impl module for member_role types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MemberRole

// Trait impl: Default
impl Default for MemberRole { fn default () -> Self { Self :: Member } }

// Methods: as_str
impl MemberRole { pub fn as_str (& self) -> & 'static str { match self { Self :: Admin => "admin" , Self :: Member => "member" , Self :: Guest => "guest" , } } }

