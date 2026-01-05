//! Tool Schema Library
//!
//! Modular, schema-first tool definitions for the multi-agent framework.
//! Replaces Heddle with specialized tools for multi-modal segmentation,
//! classification, entity spawning, and cognitive binding hints.
//!
//! All schemas are defined in Rust and generate TypeScript/Python automatically.

pub mod base;
pub mod segmentation;
pub mod intent;
pub mod entity;
pub mod spawn;
pub mod hints;
pub mod orchestration;

// Re-exports for convenience
pub use self::base::*;
pub use self::segmentation::*;
pub use self::intent::*;
pub use self::entity::*;
pub use self::spawn::*;
pub use self::orchestration::*;
