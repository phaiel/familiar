//! Impl module for timestamp types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Timestamp

// Methods: now, as_utc
impl Timestamp { pub fn now () -> Self { Self (Utc :: now ()) } # [doc = " Get the inner DateTime<Utc> for database operations"] pub fn as_utc (& self) -> DateTime < Utc > { self . 0 } }

// Trait impl: Default
impl Default for Timestamp { fn default () -> Self { Self :: now () } }

// Trait impl: Sub
# [doc = " Subtracting two Timestamps returns a Duration"] impl Sub for Timestamp { type Output = Duration ; fn sub (self , rhs : Timestamp) -> Duration { self . 0 - rhs . 0 } }

