//! Impl module for anthropic_conversation types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AnthropicConversation

// Trait impl: From
impl From < & Conversation > for AnthropicConversation { fn from (conv : & Conversation) -> Self { let mut system = None ; let mut messages = Vec :: new () ; for msg in & conv . messages { match msg . role { MessageRole :: System => { system = Some (msg . content . clone ()) ; } MessageRole :: User => { messages . push (AnthropicMessage { role : "user" . to_string () , content : msg . content . clone () , }) ; } MessageRole :: Assistant => { messages . push (AnthropicMessage { role : "assistant" . to_string () , content : msg . content . clone () , }) ; } } } Self { system , messages } } }

