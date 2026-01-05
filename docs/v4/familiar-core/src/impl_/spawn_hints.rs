//! Impl module for spawn_hints types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SpawnHints

// Trait impl: Default
impl Default for SpawnHints { fn default () -> Self { Self { physics : PhysicsHints :: default () , thread : ThreadHints :: default () , bond : None , binding : None , } } }

