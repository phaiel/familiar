//! Impl module for api_key types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ApiKey

// Methods: new, as_str
impl ApiKey { pub fn new (key : String) -> Result < Self , String > { if key . is_empty () { return Err ("API key cannot be empty" . to_string ()) ; } if key . len () < 10 { return Err ("API key appears too short" . to_string ()) ; } Ok (Self (key)) } pub fn as_str (& self) -> & str { & self . 0 } }

// Trait impl: Serialize
impl Serialize for ApiKey { fn serialize < S > (& self , serializer : S) -> Result < S :: Ok , S :: Error > where S : serde :: Serializer , { serializer . serialize_str ("[REDACTED]") } }

// Trait impl: Debug
impl fmt :: Debug for ApiKey { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "ApiKey([REDACTED])") } }

// Trait impl: Display
impl fmt :: Display for ApiKey { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "[REDACTED]") } }

