//! Impl module for magic_link_purpose types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MagicLinkPurpose

// Methods: as_str
impl MagicLinkPurpose { pub fn as_str (& self) -> & 'static str { match self { Self :: Login => "login" , Self :: Signup => "signup" , Self :: VerifyEmail => "verify_email" , Self :: PasswordReset => "password_reset" , } } }

