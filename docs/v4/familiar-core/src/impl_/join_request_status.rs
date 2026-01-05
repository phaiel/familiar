//! Impl module for join_request_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for JoinRequestStatus

// Trait impl: Default
impl Default for JoinRequestStatus { fn default () -> Self { Self :: Pending } }

// Methods: as_str
impl JoinRequestStatus { pub fn as_str (& self) -> & 'static str { match self { Self :: Pending => "pending" , Self :: Approved => "approved" , Self :: Rejected => "rejected" , } } }

