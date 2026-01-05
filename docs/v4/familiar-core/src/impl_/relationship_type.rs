//! Impl module for relationship_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RelationshipType

// Methods: default_characteristics
impl RelationshipType { # [doc = " Default characteristics for this relationship type"] pub fn default_characteristics (& self) -> BondCharacteristics { match self { Self :: Family => BondCharacteristics { relationship_type : * self , strength : 0.8 , valence : 0.6 , reciprocity : 0.8 , intimacy : 0.7 , trust : 0.7 , .. Default :: default () } , Self :: Friend | Self :: CloseFriend => BondCharacteristics { relationship_type : * self , strength : 0.7 , valence : 0.8 , reciprocity : 0.8 , intimacy : 0.6 , trust : 0.7 , .. Default :: default () } , Self :: BestFriend => BondCharacteristics { relationship_type : * self , strength : 0.9 , valence : 0.9 , reciprocity : 0.9 , intimacy : 0.9 , trust : 0.9 , .. Default :: default () } , Self :: Romantic | Self :: Spouse => BondCharacteristics { relationship_type : * self , strength : 0.9 , valence : 0.9 , reciprocity : 0.9 , intimacy : 0.95 , trust : 0.9 , .. Default :: default () } , Self :: Colleague => BondCharacteristics { relationship_type : * self , strength : 0.5 , valence : 0.5 , reciprocity : 0.6 , intimacy : 0.3 , trust : 0.5 , .. Default :: default () } , Self :: Adversary | Self :: Rival => BondCharacteristics { relationship_type : * self , strength : 0.6 , valence : - 0.5 , reciprocity : 0.5 , intimacy : 0.2 , trust : 0.1 , .. Default :: default () } , Self :: Acquaintance => BondCharacteristics { relationship_type : * self , strength : 0.3 , valence : 0.5 , reciprocity : 0.4 , intimacy : 0.1 , trust : 0.4 , .. Default :: default () } , _ => BondCharacteristics { relationship_type : * self , .. Default :: default () } , } } }

