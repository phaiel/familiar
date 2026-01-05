//! Impl module for list_result types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ListResult

// Methods: new, empty, with_cursor, with_total, has_more, is_empty
impl < T > ListResult < T > { # [doc = " Create a new list result"] pub fn new (items : Vec < T >) -> Self { let count = items . len () ; Self { items , count , total : None , cursor : None , } } # [doc = " Create an empty list result"] pub fn empty () -> Self { Self { items : Vec :: new () , count : 0 , total : Some (0) , cursor : None , } } # [doc = " Add a cursor for pagination"] pub fn with_cursor (mut self , cursor : impl Into < String >) -> Self { self . cursor = Some (cursor . into ()) ; self } # [doc = " Add total count for pagination"] pub fn with_total (mut self , total : usize) -> Self { self . total = Some (total) ; self } # [doc = " Check if there are more items"] pub fn has_more (& self) -> bool { self . cursor . is_some () } # [doc = " Check if the list is empty"] pub fn is_empty (& self) -> bool { self . items . is_empty () } }

// Trait impl: Default
impl < T > Default for ListResult < T > { fn default () -> Self { Self :: empty () } }

