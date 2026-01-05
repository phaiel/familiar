//! Impl module for log_level types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for LogLevel

// Trait impl: Default
impl Default for LogLevel { fn default () -> Self { Self :: Info } }

