//! Runtime module for the schema-first enforcement engine.
//!
//! This module provides the core abstractions for executing Systems (activities/tools)
//! with zero-copy JSON parsing support.

mod system;

pub use system::{System, SystemError};

