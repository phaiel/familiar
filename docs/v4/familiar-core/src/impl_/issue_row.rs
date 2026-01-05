//! Impl module for issue_row types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for IssueRow

// Methods: from_issue
impl IssueRow { pub fn from_issue (issue : & Issue) -> Self { Self { file : truncate_path (& issue . file . display () . to_string () , 50) , line : issue . line . to_string () , message : truncate_str (& issue . message , 60) , } } }

