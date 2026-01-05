//! Impl module for schema_info types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SchemaInfo

// Trait impl: Default
impl Default for SchemaInfo { fn default () -> Self { Self { format : "json" . to_string () , subject : "familiar.envelope.v1" . to_string () , version : ENVELOPE_VERSION , } } }

