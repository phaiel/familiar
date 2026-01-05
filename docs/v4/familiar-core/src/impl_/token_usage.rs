//! Impl module for token_usage types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for TokenUsage

// Methods: new
impl TokenUsage { pub fn new (prompt : u32 , completion : u32) -> Self { Self { prompt_tokens : prompt , completion_tokens : completion , total_tokens : prompt + completion , } } }

