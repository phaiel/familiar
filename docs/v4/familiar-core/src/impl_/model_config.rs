//! Impl module for model_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ModelConfig

// Methods: new, with_context_window, with_json_mode, deprecated, with_notes
impl ModelConfig { pub fn new (id : impl Into < String > , name : impl Into < String > , provider : AIProvider , api_model_id : impl Into < String > ,) -> Self { Self { id : id . into () , name : name . into () , provider , api_model_id : api_model_id . into () , context_window : 128_000 , supports_json_mode : true , deprecated : false , notes : None , } } pub fn with_context_window (mut self , size : u32) -> Self { self . context_window = size ; self } pub fn with_json_mode (mut self , supported : bool) -> Self { self . supports_json_mode = supported ; self } pub fn deprecated (mut self) -> Self { self . deprecated = true ; self } pub fn with_notes (mut self , notes : impl Into < String >) -> Self { self . notes = Some (notes . into ()) ; self } }

