//! Impl module for invite_code types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for InviteCode

// Methods: generate, parse, as_str, matches
impl InviteCode { # [doc = " Generate a new random invite code"] pub fn generate () -> Self { use rand :: Rng ; let mut rng = rand :: thread_rng () ; let code : String = (0 .. INVITE_CODE_LENGTH) . map (| _ | { let idx = rng . gen_range (0 .. INVITE_CHARSET . len ()) ; INVITE_CHARSET [idx] as char }) . collect () ; Self (code) } # [doc = " Parse an invite code from user input"] # [doc = " "] # [doc = " Normalizes to uppercase and validates format."] pub fn parse (code : impl AsRef < str >) -> Option < Self > { let code = code . as_ref () . trim () . to_uppercase () ; if code . len () != INVITE_CODE_LENGTH { return None ; } if ! code . chars () . all (| c | INVITE_CHARSET . contains (& (c as u8))) { return None ; } Some (Self (code)) } # [doc = " Get the code as a string"] pub fn as_str (& self) -> & str { & self . 0 } # [doc = " Check if this code matches another (case-insensitive)"] pub fn matches (& self , other : & str) -> bool { self . 0 . eq_ignore_ascii_case (other . trim ()) } }

// Trait impl: Debug
impl fmt :: Debug for InviteCode { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "InviteCode({})" , self . 0) } }

// Trait impl: Display
impl fmt :: Display for InviteCode { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "{}" , self . 0) } }

