//! Impl module for fix_summary types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for FixSummary

// Methods: total_applied
impl FixSummary { pub fn total_applied (& self) -> usize { self . safe_applied + self . unsafe_applied } }

