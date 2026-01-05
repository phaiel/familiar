//! Impl module for agentic_event types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AgenticEvent

// Methods: message_received, error, course_error
impl AgenticEvent { # [doc = " Create a MessageReceived event"] pub fn message_received (course_id : impl Into < String > , message_id : impl Into < String > , role : impl Into < String > , content : impl Into < String > ,) -> Self { Self :: MessageReceived { course_id : course_id . into () , message_id : message_id . into () , role : role . into () , content : Some (content . into ()) , message_type : None , agent : None , timestamp : chrono :: Utc :: now () . to_rfc3339 () , } } # [doc = " Create an Error event"] pub fn error (code : impl Into < String > , message : impl Into < String >) -> Self { Self :: Error { course_id : None , code : code . into () , error : message . into () , details : None , } } # [doc = " Create an Error event for a specific course"] pub fn course_error (course_id : impl Into < String > , code : impl Into < String > , message : impl Into < String > ,) -> Self { Self :: Error { course_id : Some (course_id . into ()) , code : code . into () , error : message . into () , details : None , } } }

