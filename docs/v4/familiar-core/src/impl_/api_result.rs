//! Impl module for api_result types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ApiResult

// Methods: ok, err, err_with_details, is_ok, is_err, unwrap, into_option
impl < T > ApiResult < T > { # [doc = " Create a successful response with data"] pub fn ok (data : T) -> Self { Self { success : true , data : Some (data) , error : None , } } # [doc = " Create an error response"] pub fn err (message : impl Into < String > , code : impl Into < String >) -> Self { Self { success : false , data : None , error : Some (ApiError :: new (message , code)) , } } # [doc = " Create an error response with details"] pub fn err_with_details (message : impl Into < String > , code : impl Into < String > , details : serde_json :: Value) -> Self { Self { success : false , data : None , error : Some (ApiError :: with_details (message , code , details)) , } } # [doc = " Check if this is a success response"] pub fn is_ok (& self) -> bool { self . success } # [doc = " Check if this is an error response"] pub fn is_err (& self) -> bool { ! self . success } # [doc = " Get the data, panics if error"] pub fn unwrap (self) -> T { self . data . expect ("Called unwrap on an error ApiResult") } # [doc = " Get the data, returns None if error"] pub fn into_option (self) -> Option < T > { self . data } }

// Trait impl: Default
impl < T : Default > Default for ApiResult < T > { fn default () -> Self { Self :: ok (T :: default ()) } }

