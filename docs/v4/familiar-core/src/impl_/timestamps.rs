//! Impl module for timestamps types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Timestamps

// Trait impl: Default
impl Default for Timestamps { fn default () -> Self { Self :: now () } }

// Methods: now, from_db, touch, age, since_update, was_modified
impl Timestamps { # [doc = " Create new timestamps with current time for both fields"] pub fn now () -> Self { let now = Utc :: now () ; Self { created_at : now , updated_at : now , } } # [doc = " Create timestamps from specific values (for database reads)"] pub fn from_db (created_at : DateTime < Utc > , updated_at : DateTime < Utc >) -> Self { Self { created_at , updated_at } } # [doc = " Update the `updated_at` timestamp to current time"] pub fn touch (& mut self) { self . updated_at = Utc :: now () ; } # [doc = " Get the age since creation"] pub fn age (& self) -> chrono :: Duration { Utc :: now () - self . created_at } # [doc = " Get the time since last update"] pub fn since_update (& self) -> chrono :: Duration { Utc :: now () - self . updated_at } # [doc = " Check if entity was modified after creation"] pub fn was_modified (& self) -> bool { self . updated_at > self . created_at } }

