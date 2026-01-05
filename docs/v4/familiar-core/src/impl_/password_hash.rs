//! Impl module for password_hash types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PasswordHash

// Methods: from_hash, as_str, to_string_for_storage, verify, hash
impl PasswordHash { # [doc = " Create from an already-hashed password (e.g., from database)"] pub fn from_hash (hash : impl Into < String >) -> Self { Self (hash . into ()) } # [doc = " Get the hash for storage (internal use only)"] pub (crate) fn as_str (& self) -> & str { & self . 0 } # [doc = " Get the hash string for storage in database"] # [doc = " "] # [doc = " This is the PHC string format containing algorithm, params, salt, and hash."] pub fn to_string_for_storage (& self) -> String { self . 0 . clone () } # [doc = " Verify this hash against a plaintext password"] # [doc = " "] # [doc = " Returns true if the password matches."] # [doc = " This method exists so the raw hash never needs to be exposed."] # [cfg (feature = "password-hashing")] pub fn verify (& self , password : & str) -> bool { use argon2 :: password_hash :: PasswordHash as ParsedHash ; let parsed_hash = match ParsedHash :: new (& self . 0) { Ok (h) => h , Err (_) => return false , } ; Argon2 :: default () . verify_password (password . as_bytes () , & parsed_hash) . is_ok () } # [doc = " Create a new hash from a plaintext password"] # [doc = " "] # [doc = " Uses Argon2id with default parameters (recommended for most use cases):"] # [doc = " - Memory cost: 19 MiB"] # [doc = " - Time cost: 2 iterations  "] # [doc = " - Parallelism: 1"] # [cfg (feature = "password-hashing")] pub fn hash (password : & str) -> Result < Self , PasswordHashError > { let salt = SaltString :: generate (& mut OsRng) ; let argon2 = Argon2 :: default () ; let hash = argon2 . hash_password (password . as_bytes () , & salt) . map_err (| _ | PasswordHashError :: HashingFailed) ? ; Ok (Self (hash . to_string ())) } }

// Trait impl: Debug
impl fmt :: Debug for PasswordHash { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "PasswordHash([REDACTED])") } }

// Trait impl: Display
impl fmt :: Display for PasswordHash { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "[PASSWORD_HASH]") } }

