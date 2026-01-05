//! Impl module for session_token types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SessionToken

// Methods: generate, from_string, as_str, hash, matches_hash
impl SessionToken { # [doc = " Generate a new random session token"] pub fn generate () -> Self { use rand :: Rng ; let token : String = rand :: thread_rng () . sample_iter (& rand :: distributions :: Alphanumeric) . take (64) . map (char :: from) . collect () ; Self (token) } # [doc = " Create from an existing token string (for parsing from headers)"] pub fn from_string (token : impl Into < String >) -> Self { Self (token . into ()) } # [doc = " Get the token value (for sending to client)"] pub fn as_str (& self) -> & str { & self . 0 } # [doc = " Hash the token for storage"] # [doc = " "] # [doc = " Tokens are stored hashed in the database."] # [doc = " The raw token is only ever sent to the client once."] pub fn hash (& self) -> String { use sha2 :: { Sha256 , Digest } ; let mut hasher = Sha256 :: new () ; hasher . update (self . 0 . as_bytes ()) ; format ! ("{:x}" , hasher . finalize ()) } # [doc = " Check if this token matches a stored hash"] pub fn matches_hash (& self , hash : & str) -> bool { self . hash () == hash } }

// Trait impl: Debug
impl fmt :: Debug for SessionToken { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "SessionToken([REDACTED])") } }

// Trait impl: Display
impl fmt :: Display for SessionToken { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { if self . 0 . len () > 8 { write ! (f , "{}..." , & self . 0 [.. 8]) } else { write ! (f , "[TOKEN]") } } }

