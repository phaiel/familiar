//! Impl module for file_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for FileType

// Methods: from_extension
impl FileType { pub fn from_extension (ext : & str) -> Option < Self > { match ext { "rs" => Some (Self :: Rust) , "ts" | "tsx" => Some (Self :: TypeScript) , "py" => Some (Self :: Python) , _ => None , } } }

