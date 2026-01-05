//! Impl module for tool_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ToolError

// Methods: invalid_input, llm_error, timeout
impl ToolError { pub fn invalid_input (message : impl Into < String >) -> Self { Self { code : ToolErrorCode :: InvalidInput , message : message . into () , details : None , retryable : false , } } pub fn llm_error (message : impl Into < String >) -> Self { Self { code : ToolErrorCode :: LlmError , message : message . into () , details : None , retryable : true , } } pub fn timeout (message : impl Into < String >) -> Self { Self { code : ToolErrorCode :: Timeout , message : message . into () , details : None , retryable : true , } } }

