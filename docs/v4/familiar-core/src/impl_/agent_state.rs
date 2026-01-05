//! Impl module for agent_state types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AgentState

// Trait impl: Default
impl Default for AgentState { fn default () -> Self { Self { current_speaker : None , is_authenticated : false , just_finished : false , tenant_id : String :: new () , conversation_context : vec ! [] , thread_id : None , metadata : serde_json :: Value :: Null , } } }

// Methods: new, with_speaker, add_turn, finish_task
impl AgentState { # [doc = " Create a new state for a tenant"] pub fn new (tenant_id : impl Into < String >) -> Self { Self { tenant_id : tenant_id . into () , .. Default :: default () } } # [doc = " Set the current speaker"] pub fn with_speaker (mut self , speaker : impl Into < String >) -> Self { self . current_speaker = Some (speaker . into ()) ; self } # [doc = " Add a conversation turn"] pub fn add_turn (& mut self , role : impl Into < String > , content : impl Into < String >) { self . conversation_context . push (ConversationTurn { role : role . into () , content : content . into () , speaker : self . current_speaker . clone () , timestamp : Some (chrono :: Utc :: now () . to_rfc3339 ()) , }) ; } # [doc = " Mark the current task as finished"] pub fn finish_task (& mut self) { self . just_finished = true ; self . current_speaker = None ; } }

