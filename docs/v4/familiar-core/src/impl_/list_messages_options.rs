//! Impl module for list_messages_options types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ListMessagesOptions

// Trait impl: Default
impl Default for ListMessagesOptions { fn default () -> Self { Self { limit : Some (50) , before : None , after : None , } } }

