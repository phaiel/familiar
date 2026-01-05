//! Impl module for core_runtime_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CoreRuntimeConfig

// Methods: load
impl CoreRuntimeConfig { # [doc = " Load configuration from config.toml with environment variable overrides"] # [doc = " Supports both APP_ prefixed variables and direct .env variable names"] pub fn load () -> Result < Self , config :: ConfigError > { let mut builder = Config :: builder () . add_source (File :: with_name ("config") . required (false)) . add_source (Environment :: with_prefix ("APP") . separator ("__")) . add_source (Environment :: with_prefix ("") . separator ("__")) ; if let Ok (url) = std :: env :: var ("WINDMILL_URL") { builder = builder . set_override ("windmill.url" , url) ? ; } if let Ok (workspace) = std :: env :: var ("WINDMILL_WORKSPACE") { builder = builder . set_override ("windmill.workspace" , workspace) ? ; } if let Ok (token) = std :: env :: var ("WINDMILL_TOKEN") { builder = builder . set_override ("windmill.token" , token) ? ; } builder . build () ? . try_deserialize () } }

