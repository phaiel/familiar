//! Impl module for request_context types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for RequestContext

// Methods: new, from_ip, system, is_system, safe_ip
impl RequestContext { # [doc = " Create a new request context"] pub fn new (ip_address : Option < String > , user_agent : Option < String >) -> Self { Self { ip_address , user_agent } } # [doc = " Create from IP only"] pub fn from_ip (ip : impl Into < String >) -> Self { Self { ip_address : Some (ip . into ()) , user_agent : None , } } # [doc = " Create empty context (for system-initiated actions)"] pub fn system () -> Self { Self :: default () } # [doc = " Check if this is a system context (no client info)"] pub fn is_system (& self) -> bool { self . ip_address . is_none () && self . user_agent . is_none () } # [doc = " Get a display-safe version of the IP (for logs)"] pub fn safe_ip (& self) -> & str { self . ip_address . as_deref () . unwrap_or ("unknown") } }

