//! Impl module for raw_message_intent types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RawMessageIntent

// Trait impl: Default
impl Default for RawMessageIntent { fn default () -> Self { Self { intent : MessageIntent :: Log , confidence : 1.0 , query_type : None , query_target : None , } } }

