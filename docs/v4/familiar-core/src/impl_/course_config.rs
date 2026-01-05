//! Impl module for course_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CourseConfig

// Trait impl: Default
impl Default for CourseConfig { fn default () -> Self { Self { max_context_tokens : default_max_context_tokens () , reserved_tokens : default_reserved_tokens () , min_messages : default_min_messages () , estimation_method : default_estimation_method () , } } }

// Methods: available_tokens, large_context, small_context
impl CourseConfig { # [doc = " Get the available token budget for history (max - reserved)"] pub fn available_tokens (& self) -> usize { self . max_context_tokens . saturating_sub (self . reserved_tokens) } # [doc = " Create a config for large context models (e.g., Claude Opus, GPT-4 Turbo)"] pub fn large_context () -> Self { Self { max_context_tokens : 32000 , reserved_tokens : 4000 , min_messages : 4 , estimation_method : TokenEstimationMethod :: Char4 , } } # [doc = " Create a config for small/fast models (e.g., Claude Haiku, GPT-4o Mini)"] pub fn small_context () -> Self { Self { max_context_tokens : 4000 , reserved_tokens : 1000 , min_messages : 2 , estimation_method : TokenEstimationMethod :: Char4 , } } }

