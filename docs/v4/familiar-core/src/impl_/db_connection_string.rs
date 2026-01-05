//! Impl module for db_connection_string types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbConnectionString

// Methods: new, as_str
impl DbConnectionString { pub fn new (url : impl Into < String >) -> Result < Self , String > { let url = url . into () ; if url . is_empty () { return Err ("Connection string cannot be empty" . to_string ()) ; } if ! url . starts_with ("postgres://") && ! url . starts_with ("postgresql://") { return Err ("Connection string must start with postgres:// or postgresql://" . to_string ()) ; } Ok (Self (url)) } pub fn as_str (& self) -> & str { & self . 0 } }

// Trait impl: Default
impl Default for DbConnectionString { fn default () -> Self { Self ("postgresql://localhost:5432/familiar" . to_string ()) } }

