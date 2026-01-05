//! Impl module for retry_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RetryConfig

// Methods: aggressive, conservative, none, delay_for_attempt
impl RetryConfig { pub fn aggressive () -> Self { Self { max_retries : 5 , initial_delay_ms : 100 , max_delay_ms : 10000 , backoff_factor : 2.0 } } pub fn conservative () -> Self { Self { max_retries : 2 , initial_delay_ms : 500 , max_delay_ms : 5000 , backoff_factor : 2.0 } } pub fn none () -> Self { Self { max_retries : 0 , initial_delay_ms : 0 , max_delay_ms : 0 , backoff_factor : 1.0 } } pub fn delay_for_attempt (& self , attempt : u32) -> Duration { if attempt == 0 { return Duration :: from_millis (0) ; } let delay = self . initial_delay_ms as f64 * self . backoff_factor . powi (attempt as i32 - 1) ; Duration :: from_millis (delay . min (self . max_delay_ms as f64) as u64) } }

// Trait impl: Default
impl Default for RetryConfig { fn default () -> Self { Self :: conservative () } }

