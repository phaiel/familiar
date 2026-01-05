//! # Legacy Modules
//!
//! **DEPRECATED** - This module contains old hand-written types that have been
//! replaced by schema-generated types in `crate::contracts`.
//!
//! This code is kept for reference only during migration. It is NOT exported
//! from the crate's public API.
//!
//! ## Migration Guide
//!
//! Replace imports:
//! ```rust,ignore
//! // OLD
//! use crate::types::*;
//! use crate::entities::*;
//! use crate::components::*;
//!
//! // NEW
//! use crate::contracts::*;
//! ```

pub mod types;
pub mod entities;
pub mod components;
pub mod generated;


