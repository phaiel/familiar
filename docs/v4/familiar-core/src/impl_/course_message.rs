//! Impl module for course_message types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CourseMessage

// Methods: user, user_with_shuttle, assistant, assistant_with_shuttle, system
impl CourseMessage { # [doc = " Create a new user message"] pub fn user (course_id : UUID , content : impl Into < String >) -> Self { Self { id : UUID :: new () , course_id , role : MessageRole :: User , content : Some (content . into ()) , agent_speaker : None , shuttle_id : None , timestamp : Timestamp :: now () , metadata : None , } } # [doc = " Create a user message with shuttle tracking"] pub fn user_with_shuttle (course_id : UUID , shuttle_id : UUID , content : impl Into < String >) -> Self { Self { id : UUID :: new () , course_id , role : MessageRole :: User , content : Some (content . into ()) , agent_speaker : None , shuttle_id : Some (shuttle_id) , timestamp : Timestamp :: now () , metadata : None , } } # [doc = " Create a new assistant message"] pub fn assistant (course_id : UUID , content : impl Into < String > , agent : Option < String > ,) -> Self { Self { id : UUID :: new () , course_id , role : MessageRole :: Assistant , content : Some (content . into ()) , agent_speaker : agent , shuttle_id : None , timestamp : Timestamp :: now () , metadata : None , } } # [doc = " Create an assistant message with shuttle tracking"] pub fn assistant_with_shuttle (course_id : UUID , shuttle_id : UUID , content : impl Into < String > , agent : Option < String > ,) -> Self { Self { id : UUID :: new () , course_id , role : MessageRole :: Assistant , content : Some (content . into ()) , agent_speaker : agent , shuttle_id : Some (shuttle_id) , timestamp : Timestamp :: now () , metadata : None , } } # [doc = " Create a system message"] pub fn system (course_id : UUID , content : impl Into < String >) -> Self { Self { id : UUID :: new () , course_id , role : MessageRole :: System , content : Some (content . into ()) , agent_speaker : None , shuttle_id : None , timestamp : Timestamp :: now () , metadata : None , } } }

