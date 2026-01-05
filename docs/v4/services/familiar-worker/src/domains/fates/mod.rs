//! Fates Domain - AI Pipeline
//!
//! The Fates process messages through the agentic pipeline:
//! - Gate: Classification and routing
//! - Morta: Content segmentation
//! - Decima: Entity extraction
//! - Nona: Response generation

pub mod gate;
pub mod morta;
pub mod decima;
pub mod nona;
pub mod pipeline;
