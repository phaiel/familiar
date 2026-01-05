//! Impl module for sim_l_o_d types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SimLOD

// Trait impl: Default
impl Default for SimLOD { fn default () -> Self { Self { tier : SimulationTier :: Conscious , last_update : 0 , } } }

