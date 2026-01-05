//! Impl module for thread_hints types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ThreadHints

// Trait impl: Default
impl Default for ThreadHints { fn default () -> Self { Self { primary_subject : "user" . to_string () , related_threads : vec ! [] , thread_role : ThreadRole :: Subject , keywords : vec ! [] , create_if_missing : false , } } }

