//! Impl module for command_result types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CommandResult

// Methods: success, failure
impl CommandResult { # [doc = " Create a successful result"] pub fn success (job_id : impl Into < String > , course_id : impl Into < String >) -> Self { let job = job_id . into () ; Self { accepted : true , job_id : Some (job . clone ()) , course_id : Some (course_id . into ()) , ws_url : Some (format ! ("/api/courses/{}/ws" , job)) , error : None , } } # [doc = " Create an error result"] pub fn failure (error : impl Into < String >) -> Self { Self { accepted : false , job_id : None , course_id : None , ws_url : None , error : Some (error . into ()) , } } }

