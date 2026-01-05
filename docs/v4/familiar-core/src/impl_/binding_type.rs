//! Impl module for binding_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for BindingType

// Methods: description
impl BindingType { pub fn description (& self) -> & 'static str { match self { Self :: Causal => "One entity causes or leads to another" , Self :: Temporal => "Entities are connected by time sequence" , Self :: Associative => "Entities are associated by context or proximity" , Self :: Compositional => "One entity is part of another" , Self :: Contrastive => "Entities are contrasted or opposed" , Self :: Analogical => "Entities are similar or analogous" , Self :: Enabling => "One entity enables or is prerequisite for another" , Self :: Thematic => "Entities share a common theme" , } } }

