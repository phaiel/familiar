//! Impl module for analysis_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AnalysisResponse

// Trait impl: Default
impl Default for AnalysisResponse { fn default () -> Self { Self { classification : None , should_migrate : false , priority : 0 , reasoning : String :: new () , suggested_fix : None , dependencies : vec ! [] , error : None , } } }

