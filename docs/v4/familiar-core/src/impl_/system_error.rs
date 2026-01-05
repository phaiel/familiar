//! Impl module for system_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SystemError

// Trait impl: From
impl From < serde_json :: Error > for SystemError { fn from (err : serde_json :: Error) -> Self { SystemError :: DeserializationError (err . to_string ()) } }

// Trait impl: From
impl From < anyhow :: Error > for SystemError { fn from (err : anyhow :: Error) -> Self { SystemError :: ExecutionError (err . to_string ()) } }

