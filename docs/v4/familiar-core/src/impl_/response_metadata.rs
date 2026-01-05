//! Impl module for response_metadata types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ResponseMetadata

// Methods: new, from_model_info, with_usage, with_request_id
impl ResponseMetadata { pub fn new (provider : AIProvider , model : & ModelConfig , latency_ms : u64) -> Self { Self { provider , model_id : model . id . clone () , model_name : model . name . clone () , usage : None , latency_ms , request_id : None , } } pub fn from_model_info (provider : AIProvider , model_id : & str , model_name : & str , latency_ms : u64) -> Self { Self { provider , model_id : model_id . to_string () , model_name : model_name . to_string () , usage : None , latency_ms , request_id : None , } } pub fn with_usage (mut self , usage : TokenUsage) -> Self { self . usage = Some (usage) ; self } pub fn with_request_id (mut self , id : impl Into < String >) -> Self { self . request_id = Some (id . into ()) ; self } }

