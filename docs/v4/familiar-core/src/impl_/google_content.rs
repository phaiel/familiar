//! Impl module for google_content types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for GoogleContent

// Trait impl: From
impl From < & ChatMessage > for GoogleContent { fn from (msg : & ChatMessage) -> Self { let role = match msg . role { MessageRole :: User => Some ("user" . to_string ()) , MessageRole :: Assistant => Some ("model" . to_string ()) , MessageRole :: System => None , } ; Self { role , parts : vec ! [GooglePart { text : msg . content . clone () }] , } } }

