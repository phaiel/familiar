//! Impl module for value types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Value

// Trait impl: From
impl From < Settings > for serde_json :: Value { fn from (settings : Settings) -> Self { settings . 0 } }

// Trait impl: From
impl From < Metadata > for serde_json :: Value { fn from (metadata : Metadata) -> Self { metadata . 0 } }

