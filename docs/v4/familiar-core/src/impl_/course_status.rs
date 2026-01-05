//! Impl module for course_status types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CourseStatus

// Methods: as_str, is_writable, is_active
impl CourseStatus { pub fn as_str (& self) -> & 'static str { match self { Self :: Idle => "idle" , Self :: Active => "active" , Self :: Archived => "archived" , } } # [doc = " Check if course can accept new messages"] pub fn is_writable (& self) -> bool { matches ! (self , Self :: Idle | Self :: Active) } # [doc = " Check if course is being actively processed"] pub fn is_active (& self) -> bool { matches ! (self , Self :: Active) } }

// Trait impl: Default
impl Default for CourseStatus { fn default () -> Self { Self :: Idle } }

// Trait impl: Display
impl std :: fmt :: Display for CourseStatus { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

