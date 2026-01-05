//! Impl module for weave types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Weave

// Methods: new, with_context, len, is_empty
impl Weave { pub fn new (content : impl Into < String >) -> Self { Self { raw_content : content . into () , context : None , } } pub fn with_context (mut self , context : impl Into < String >) -> Self { self . context = Some (context . into ()) ; self } # [doc = " Get the content length"] pub fn len (& self) -> usize { self . raw_content . len () } # [doc = " Check if empty"] pub fn is_empty (& self) -> bool { self . raw_content . is_empty () } }

// Trait impl: Default
impl Default for Weave { fn default () -> Self { Self { raw_content : String :: new () , context : None , } } }

// Trait impl: From
impl From < & str > for Weave { fn from (s : & str) -> Self { Self :: new (s) } }

// Trait impl: From
impl From < String > for Weave { fn from (s : String) -> Self { Self :: new (s) } }

// Trait impl: From
impl From < & String > for Weave { fn from (s : & String) -> Self { Self :: new (s) } }

