//! Impl module for observation_request types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ObservationRequest

// Methods: new, with_context, with_config
impl ObservationRequest { pub fn new (segment : impl Into < String >) -> Self { Self { context : String :: new () , segment : segment . into () , config : RequestConfig :: default () } } pub fn with_context (mut self , context : impl Into < String >) -> Self { self . context = context . into () ; self } pub fn with_config (mut self , config : RequestConfig) -> Self { self . config = config ; self } }

