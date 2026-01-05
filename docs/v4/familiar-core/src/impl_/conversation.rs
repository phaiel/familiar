//! Impl module for conversation types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Conversation

// Methods: new, with_system, add, add_system, add_user, add_assistant, system_prompt, user_prompt
impl Conversation { pub fn new () -> Self { Self { messages : vec ! [] } } pub fn with_system (system : impl Into < String >) -> Self { Self { messages : vec ! [ChatMessage :: system (system)] , } } pub fn add (& mut self , message : ChatMessage) -> & mut Self { self . messages . push (message) ; self } pub fn add_system (& mut self , content : impl Into < String >) -> & mut Self { self . messages . push (ChatMessage :: system (content)) ; self } pub fn add_user (& mut self , content : impl Into < String >) -> & mut Self { self . messages . push (ChatMessage :: user (content)) ; self } pub fn add_assistant (& mut self , content : impl Into < String >) -> & mut Self { self . messages . push (ChatMessage :: assistant (content)) ; self } # [doc = " Extract system prompt(s) as a single string (for debugging)"] pub fn system_prompt (& self) -> String { self . messages . iter () . filter (| m | m . role == MessageRole :: System) . map (| m | m . content . as_str ()) . collect :: < Vec < _ > > () . join ("\n") } # [doc = " Extract user prompt(s) as a single string (for debugging)"] pub fn user_prompt (& self) -> String { self . messages . iter () . filter (| m | m . role == MessageRole :: User) . map (| m | m . content . as_str ()) . collect :: < Vec < _ > > () . join ("\n") } }

