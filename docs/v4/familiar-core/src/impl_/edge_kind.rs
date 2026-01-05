//! Impl module for edge_kind types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EdgeKind

// Methods: label, color, is_composition, is_infrastructure
impl EdgeKind { # [doc = " Get a short label for the edge type (for DOT visualization)"] pub fn label (& self) -> & 'static str { match self { EdgeKind :: TypeRef => "ref" , EdgeKind :: LocalRef => "local" , EdgeKind :: Extends => "extends" , EdgeKind :: VariantOf => "variant" , EdgeKind :: UnionOf => "union" , EdgeKind :: ItemType => "item" , EdgeKind :: ValueType => "value" , EdgeKind :: FieldType => "field" , EdgeKind :: RunsOn => "runs_on" , EdgeKind :: UsesQueue => "uses_queue" , EdgeKind :: Requires => "requires" , EdgeKind :: Reads => "reads" , EdgeKind :: Writes => "writes" , EdgeKind :: ConnectsTo => "connects_to" , EdgeKind :: Input => "input" , EdgeKind :: Output => "output" , } } # [doc = " Get a color for the edge type (for DOT visualization)"] pub fn color (& self) -> & 'static str { match self { EdgeKind :: TypeRef => "#666666" , EdgeKind :: LocalRef => "#AAAAAA" , EdgeKind :: Extends => "#4CAF50" , EdgeKind :: VariantOf => "#FF9800" , EdgeKind :: UnionOf => "#FFC107" , EdgeKind :: ItemType => "#9C27B0" , EdgeKind :: ValueType => "#E91E63" , EdgeKind :: FieldType => "#9E9E9E" , EdgeKind :: RunsOn => "#2196F3" , EdgeKind :: UsesQueue => "#673AB7" , EdgeKind :: Requires => "#FF5722" , EdgeKind :: Reads => "#00BCD4" , EdgeKind :: Writes => "#F44336" , EdgeKind :: ConnectsTo => "#03A9F4" , EdgeKind :: Input => "#8BC34A" , EdgeKind :: Output => "#FF5722" , } } # [doc = " Check if this edge type represents a schema composition construct"] pub fn is_composition (& self) -> bool { matches ! (self , EdgeKind :: Extends | EdgeKind :: VariantOf | EdgeKind :: UnionOf | EdgeKind :: ItemType | EdgeKind :: ValueType | EdgeKind :: FieldType) } # [doc = " Check if this edge type represents an infrastructure relationship"] pub fn is_infrastructure (& self) -> bool { matches ! (self , EdgeKind :: RunsOn | EdgeKind :: UsesQueue | EdgeKind :: Requires | EdgeKind :: Reads | EdgeKind :: Writes | EdgeKind :: ConnectsTo | EdgeKind :: Input | EdgeKind :: Output) } }

