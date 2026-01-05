//! Impl module for request_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RequestConfig

// Trait impl: Default
impl Default for RequestConfig { fn default () -> Self { Self { model : ModelConfig :: new ("mock" , "Mock" , crate :: config :: AIProvider :: Mock , "mock") , temperature : Temperature :: CLASSIFICATION , max_tokens : MaxTokens :: CLASSIFICATION , json_mode : true , retry : RetryConfig :: default () , timeout_ms : 30_000 , } } }

// Methods: with_model, with_temperature, with_max_tokens, with_json_mode, with_retry, with_timeout
impl RequestConfig { pub fn with_model (mut self , model : ModelConfig) -> Self { self . model = model ; self } pub fn with_temperature (mut self , temp : Temperature) -> Self { self . temperature = temp ; self } pub fn with_max_tokens (mut self , tokens : MaxTokens) -> Self { self . max_tokens = tokens ; self } pub fn with_json_mode (mut self , enabled : bool) -> Self { self . json_mode = enabled ; self } pub fn with_retry (mut self , config : RetryConfig) -> Self { self . retry = config ; self } pub fn with_timeout (mut self , timeout_ms : u64) -> Self { self . timeout_ms = timeout_ms ; self } }

