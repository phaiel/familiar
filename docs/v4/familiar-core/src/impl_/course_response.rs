//! Impl module for course_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CourseResponse

// Methods: error
impl CourseResponse { pub fn error (course_id : & str , shuttle_id : & str , original_weave : & str , provider : & str , message : String) -> Self { Self { success : false , course_id : course_id . to_string () , shuttle_id : shuttle_id . to_string () , original_weave : original_weave . to_string () , provider : provider . to_string () , media_refs : None , segments : vec ! [] , message_intent : MessageIntentResponse { intent : MessageIntent :: Log , confidence : 0.0 , query_type : None , query_target : None , } , unit_count : 0 , weave_units : vec ! [] , entities : None , metadata : None , error : Some (message) , debug_llm_request : None , debug_llm_response : None , } } }

