//! Impl module for google_conversation types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for GoogleConversation

// Trait impl: From
impl From < & Conversation > for GoogleConversation { fn from (conv : & Conversation) -> Self { let mut system_instruction = None ; let mut contents = Vec :: new () ; for msg in & conv . messages { match msg . role { MessageRole :: System => { system_instruction = Some (GoogleContent { role : None , parts : vec ! [GooglePart { text : msg . content . clone () }] , }) ; } _ => { contents . push (GoogleContent :: from (msg)) ; } } } Self { system_instruction , contents } } }

