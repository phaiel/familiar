//! Impl module for max_tokens types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MaxTokens

// Methods: new, value
impl MaxTokens { # [doc = " Standard output limit for classification"] pub const CLASSIFICATION : Self = Self (2048) ; # [doc = " Extended output for complex analysis"] pub const EXTENDED : Self = Self (4096) ; # [doc = " Minimal output"] pub const MINIMAL : Self = Self (512) ; pub fn new (value : u32) -> Self { Self (value) } pub fn value (& self) -> u32 { self . 0 } }

// Trait impl: Default
impl Default for MaxTokens { fn default () -> Self { Self :: CLASSIFICATION } }

