//! Impl module for shuttle_details types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ShuttleDetails

// Methods: new, with_provider, with_model, set_latency, set_tokens, set_unit_count, set_spawn_count
impl ShuttleDetails { pub fn new () -> Self { Self :: default () } pub fn with_provider (mut self , provider : impl Into < String >) -> Self { self . provider = Some (provider . into ()) ; self } pub fn with_model (mut self , model : impl Into < String >) -> Self { self . model = Some (model . into ()) ; self } pub fn set_latency (& mut self , ms : u64) { self . latency_ms = Some (ms) ; } pub fn set_tokens (& mut self , tokens : u32) { self . tokens_used = Some (tokens) ; } pub fn set_unit_count (& mut self , count : usize) { self . unit_count = count ; } pub fn set_spawn_count (& mut self , count : usize) { self . spawn_count = count ; } }

