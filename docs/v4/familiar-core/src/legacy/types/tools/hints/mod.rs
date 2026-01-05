//! Hint Generation Tools
//!
//! Specialized tools for generating hints:
//! - Physics: VAE space positioning (valence, arousal, significance)
//! - Thread: Narrative connections and subjects
//! - Bond: Relationship characterization
//! - Binding: Cognitive connections between entities

pub mod physics;
pub mod thread;
pub mod bond;
pub mod binding;

pub use self::physics::*;
pub use self::thread::*;
pub use self::bond::*;
pub use self::binding::*;
