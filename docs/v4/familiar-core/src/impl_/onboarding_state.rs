//! Impl module for onboarding_state types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for OnboardingState

// Trait impl: Default
impl Default for OnboardingState { fn default () -> Self { Self :: Initial } }

