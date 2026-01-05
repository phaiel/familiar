//! Impl module for api_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ApiError

// Methods: new, with_details
impl ApiError { # [doc = " Create a new API error"] pub fn new (message : impl Into < String > , code : impl Into < String >) -> Self { Self { code : code . into () , message : message . into () , details : None , } } # [doc = " Create an API error with details"] pub fn with_details (message : impl Into < String > , code : impl Into < String > , details : serde_json :: Value) -> Self { Self { code : code . into () , message : message . into () , details : Some (details) , } } # [doc = " Common error codes"] pub const NOT_FOUND : & 'static str = "NOT_FOUND" ; pub const UNAUTHORIZED : & 'static str = "UNAUTHORIZED" ; pub const FORBIDDEN : & 'static str = "FORBIDDEN" ; pub const VALIDATION_FAILED : & 'static str = "VALIDATION_FAILED" ; pub const INTERNAL_ERROR : & 'static str = "INTERNAL_ERROR" ; pub const CONFLICT : & 'static str = "CONFLICT" ; pub const RATE_LIMITED : & 'static str = "RATE_LIMITED" ; }

