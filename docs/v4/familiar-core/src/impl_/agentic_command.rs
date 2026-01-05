//! Impl module for agentic_command types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AgenticCommand

// Methods: send_message, reply_to_course, send_with_context, send_to_flow
impl AgenticCommand { # [doc = " Create a new SendMessage command"] pub fn send_message (tenant_id : impl Into < String > , content : impl Into < String > , request_id : impl Into < String > ,) -> Self { Self :: SendMessage { course_id : None , content : content . into () , conversation_history : None , blocks : None , request_id : request_id . into () , tenant_id : tenant_id . into () , flow_path : None , } } # [doc = " Create a SendMessage command for an existing course"] pub fn reply_to_course (tenant_id : impl Into < String > , course_id : impl Into < String > , content : impl Into < String > , request_id : impl Into < String > ,) -> Self { Self :: SendMessage { course_id : Some (course_id . into ()) , content : content . into () , conversation_history : None , blocks : None , request_id : request_id . into () , tenant_id : tenant_id . into () , flow_path : None , } } # [doc = " Create a SendMessage command with conversation history"] pub fn send_with_context (tenant_id : impl Into < String > , content : impl Into < String > , conversation_history : Vec < ConversationHistoryItem > , request_id : impl Into < String > ,) -> Self { Self :: SendMessage { course_id : None , content : content . into () , conversation_history : Some (conversation_history) , blocks : None , request_id : request_id . into () , tenant_id : tenant_id . into () , flow_path : None , } } # [doc = " Create a SendMessage command with a specific flow path"] pub fn send_to_flow (tenant_id : impl Into < String > , content : impl Into < String > , request_id : impl Into < String > , flow_path : impl Into < String > ,) -> Self { Self :: SendMessage { course_id : None , content : content . into () , conversation_history : None , blocks : None , request_id : request_id . into () , tenant_id : tenant_id . into () , flow_path : Some (flow_path . into ()) , } } }

