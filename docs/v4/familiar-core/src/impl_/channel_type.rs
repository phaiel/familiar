//! Impl module for channel_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ChannelType

// Trait impl: Default
impl Default for ChannelType { fn default () -> Self { Self :: Personal } }

// Methods: as_str
impl ChannelType { pub fn as_str (& self) -> & 'static str { match self { Self :: Personal => "personal" , Self :: Family => "family" , Self :: Shared => "shared" , } } }

