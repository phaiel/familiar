//! Impl module for system_entity_meta types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SystemEntityMeta

// Methods: new
impl < Id : Default > SystemEntityMeta < Id > { # [doc = " Create new system entity metadata with a default ID"] pub fn new () -> Self { Self { id : Id :: default () , timestamps : Timestamps :: now () , } } }

// Methods: with_id, touch
impl < Id > SystemEntityMeta < Id > { # [doc = " Create system entity metadata with a specific ID"] pub fn with_id (id : Id) -> Self { Self { id , timestamps : Timestamps :: now () , } } # [doc = " Update the modification timestamp"] pub fn touch (& mut self) { self . timestamps . touch () ; } }

// Trait impl: Default
impl < Id : Default > Default for SystemEntityMeta < Id > { fn default () -> Self { Self :: new () } }

