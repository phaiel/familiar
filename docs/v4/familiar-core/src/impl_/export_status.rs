//! Impl module for export_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ExportStatus

// Methods: as_str
impl ExportStatus { pub fn as_str (& self) -> & 'static str { match self { Self :: Pending => "pending" , Self :: Processing => "processing" , Self :: Ready => "ready" , Self :: Expired => "expired" , Self :: Failed => "failed" , } } }

