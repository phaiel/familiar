//! Impl module for query_target types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for QueryTarget

// Trait impl: Default
impl Default for QueryTarget { fn default () -> Self { Self { entity_types : vec ! [] , thread_hints : vec ! [] , temporal_scope : None , keywords : vec ! [] , } } }

