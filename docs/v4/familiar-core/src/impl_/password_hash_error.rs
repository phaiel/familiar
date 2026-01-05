//! Impl module for password_hash_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PasswordHashError

// Trait impl: Display
impl std :: fmt :: Display for PasswordHashError { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { match self { Self :: HashingFailed => write ! (f , "Password hashing failed") , } } }

// Trait impl: Error
impl std :: error :: Error for PasswordHashError { }

