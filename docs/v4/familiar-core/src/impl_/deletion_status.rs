//! Impl module for deletion_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DeletionStatus

// Methods: as_str
impl DeletionStatus { pub fn as_str (& self) -> & 'static str { match self { Self :: Pending => "pending" , Self :: Cancelled => "cancelled" , Self :: Processing => "processing" , Self :: Completed => "completed" , } } }

