//! Impl module for chat_message types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ChatMessage

// Methods: system, user, assistant
impl ChatMessage { pub fn system (content : impl Into < String >) -> Self { Self { role : MessageRole :: System , content : content . into () , } } pub fn user (content : impl Into < String >) -> Self { Self { role : MessageRole :: User , content : content . into () , } } pub fn assistant (content : impl Into < String >) -> Self { Self { role : MessageRole :: Assistant , content : content . into () , } } }

