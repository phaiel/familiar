//! Impl module for identity types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Identity

// Methods: new
impl Identity { pub fn new (tenant_id : UUID) -> Self { Self { id : UUID :: new () , tenant_id , created_at : Timestamp :: now () , } } }

// Trait impl: Default
impl Default for Identity { fn default () -> Self { Self :: new (UUID :: new ()) } }

// Trait impl: Component
impl Component for Identity { }

