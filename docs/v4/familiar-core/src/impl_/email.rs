//! Impl module for email types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Email

// Methods: new, from_trusted, validate, as_str, local_part, domain
impl Email { # [doc = " Create a new Email, validating the format"] pub fn new (email : impl Into < String >) -> Result < Self , EmailError > { let email = email . into () ; Self :: validate (& email) ? ; Ok (Self (email . to_lowercase ())) } # [doc = " Create without validation (for trusted sources like DB)"] pub fn from_trusted (email : impl Into < String >) -> Self { Self (email . into ()) } fn validate (email : & str) -> Result < () , EmailError > { if email . is_empty () { return Err (EmailError :: Empty) ; } if ! email . contains ('@') { return Err (EmailError :: MissingAt) ; } let parts : Vec < & str > = email . split ('@') . collect () ; if parts . len () != 2 || parts [0] . is_empty () || parts [1] . is_empty () { return Err (EmailError :: InvalidFormat) ; } if ! parts [1] . contains ('.') { return Err (EmailError :: InvalidFormat) ; } Ok (()) } # [doc = " Get the email as a string slice"] pub fn as_str (& self) -> & str { & self . 0 } # [doc = " Get the local part (before @)"] pub fn local_part (& self) -> & str { self . 0 . split ('@') . next () . unwrap_or ("") } # [doc = " Get the domain part (after @)"] pub fn domain (& self) -> & str { self . 0 . split ('@') . nth (1) . unwrap_or ("") } }

// Trait impl: Display
impl fmt :: Display for Email { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "{}" , self . 0) } }

// Trait impl: AsRef
impl AsRef < str > for Email { fn as_ref (& self) -> & str { & self . 0 } }

