//! Impl module for async_task_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AsyncTaskStatus

// Trait impl: Default
impl Default for AsyncTaskStatus { fn default () -> Self { Self :: Pending } }

