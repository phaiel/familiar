//! Impl module for course_summary types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CourseSummary

// Trait impl: From
impl From < & Course > for CourseSummary { fn from (course : & Course) -> Self { course . to_summary () } }

