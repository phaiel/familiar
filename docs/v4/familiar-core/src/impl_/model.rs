//! Impl module for model types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Model

// Trait impl: OptimisticLock
impl OptimisticLock for super :: task :: async_task :: Model { fn version (& self) -> i32 { self . version } }

// Trait impl: OptimisticLock
impl OptimisticLock for super :: auth :: session :: Model { fn version (& self) -> i32 { self . version } }

// Trait impl: OptimisticLock
impl OptimisticLock for super :: conversation :: channel :: Model { fn version (& self) -> i32 { self . version } }

