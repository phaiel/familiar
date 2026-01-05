//! Impl module for open_a_i_message types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for OpenAIMessage

// Trait impl: From
impl From < & ChatMessage > for OpenAIMessage { fn from (msg : & ChatMessage) -> Self { Self { role : msg . role . to_string () , content : msg . content . clone () , } } }

