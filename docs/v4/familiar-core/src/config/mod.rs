//! Configuration Module
//!
//! Schema-driven configuration that's easy to update.

pub mod models;
pub mod manifest;
pub mod course;
pub mod schema_lock;

pub use models::{
    AIProvider, ModelConfig,
    get_available_models, find_model, get_models_for_provider, get_default_model,
};

pub use manifest::{SystemManifest, SystemDomain, SystemTrigger};

pub use course::{
    CourseConfig, TokenEstimationMethod,
    estimate_tokens, estimate_tokens_char4, estimate_tokens_word_based,
};

