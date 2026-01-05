//! Impl module for model_info types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ModelInfo

// Trait impl: From
impl From < & ModelConfig > for ModelInfo { fn from (config : & ModelConfig) -> Self { Self { id : config . id . clone () , name : config . name . clone () , provider : config . provider . as_str () . to_string () , api_model_id : config . api_model_id . clone () , } } }

// Trait impl: From
impl From < ModelConfig > for ModelInfo { fn from (config : ModelConfig) -> Self { Self { id : config . id , name : config . name , provider : config . provider . as_str () . to_string () , api_model_id : config . api_model_id , } } }

