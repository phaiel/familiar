//! Impl module for classified_purpose types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ClassifiedPurpose

// Methods: new, with_evidence
impl ClassifiedPurpose { pub fn new (purpose : ToolPurpose , confidence : f64) -> Self { Self { purpose , confidence , evidence : vec ! [] , } } pub fn with_evidence (mut self , evidence : Vec < String >) -> Self { self . evidence = evidence ; self } }

