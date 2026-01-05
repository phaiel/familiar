//! Impl module for task_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for TaskStatus

// Trait impl: Default
impl Default for TaskStatus { fn default () -> Self { Self :: Pending } }

