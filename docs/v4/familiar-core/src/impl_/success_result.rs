//! Impl module for success_result types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SuccessResult

// Methods: ok, with_message
impl SuccessResult { # [doc = " Create a success result"] pub fn ok () -> Self { Self { success : true , message : None , } } # [doc = " Create a success result with a message"] pub fn with_message (message : impl Into < String >) -> Self { Self { success : true , message : Some (message . into ()) , } } }

// Trait impl: Default
impl Default for SuccessResult { fn default () -> Self { Self :: ok () } }

