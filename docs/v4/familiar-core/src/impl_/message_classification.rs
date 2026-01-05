//! Impl module for message_classification types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MessageClassification

// Methods: log, query, with_target, with_confidence
impl MessageClassification { pub fn log () -> Self { Self { intent : MessageIntent :: Log , confidence : 1.0 , query_type : None , query_target : None , secondary_intents : vec ! [] , } } pub fn query (query_type : QueryType) -> Self { Self { intent : MessageIntent :: Query , confidence : 1.0 , query_type : Some (query_type) , query_target : None , secondary_intents : vec ! [] , } } pub fn with_target (mut self , target : QueryTarget) -> Self { self . query_target = Some (target) ; self } pub fn with_confidence (mut self , confidence : f64) -> Self { self . confidence = confidence ; self } }

