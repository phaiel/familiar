//! Impl module for health_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for HealthResponse

// Methods: healthy
impl HealthResponse { pub fn healthy (service : & str , version : & str) -> Self { Self { status : "healthy" . to_string () , service : service . to_string () , version : version . to_string () , } } }

