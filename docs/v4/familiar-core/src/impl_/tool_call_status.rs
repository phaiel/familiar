//! Impl module for tool_call_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ToolCallStatus

// Trait impl: Default
impl Default for ToolCallStatus { fn default () -> Self { Self :: Pending } }

