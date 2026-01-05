//! Impl module for t types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for T

// Trait impl: SystemAsTool
impl < T : System > SystemAsTool for T { }

