//! Impl module for producer_info types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ProducerInfo

// Methods: current, api, worker
impl ProducerInfo { # [doc = " Create producer info for the current service"] pub fn current (service : impl Into < String >) -> Self { Self { service : service . into () , instance : std :: env :: var ("HOSTNAME") . or_else (| _ | std :: env :: var ("POD_NAME")) . unwrap_or_else (| _ | "unknown" . to_string ()) , build : option_env ! ("GIT_HASH") . map (String :: from) , } } # [doc = " Create producer info for familiar-api"] pub fn api () -> Self { Self :: current ("familiar-api") } # [doc = " Create producer info for familiar-worker"] pub fn worker () -> Self { Self :: current ("familiar-worker") } }

