//! Impl module for query_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for QueryType

// Methods: as_str
impl QueryType { pub fn as_str (& self) -> & 'static str { match self { Self :: Temporal => "TEMPORAL" , Self :: Entity => "ENTITY" , Self :: Pattern => "PATTERN" , Self :: Comparison => "COMPARISON" , Self :: Summary => "SUMMARY" , Self :: Quantitative => "QUANTITATIVE" , Self :: Boolean => "BOOLEAN" , Self :: Causal => "CAUSAL" , Self :: Spatial => "SPATIAL" , Self :: Exploratory => "EXPLORATORY" , } } }

// Trait impl: Display
impl std :: fmt :: Display for QueryType { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

