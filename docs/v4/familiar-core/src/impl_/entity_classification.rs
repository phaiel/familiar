//! Impl module for entity_classification types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EntityClassification

// Methods: new, with_evidence
impl EntityClassification { pub fn new (entity_type : EntityType , probability : f64) -> Self { Self { entity_type , probability , evidence : vec ! [] , } } pub fn with_evidence (mut self , evidence : Vec < ClassificationEvidence >) -> Self { self . evidence = evidence ; self } }

