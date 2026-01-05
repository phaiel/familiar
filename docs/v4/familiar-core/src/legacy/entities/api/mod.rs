//! API Endpoint Entities
//!
//! Each module represents an API endpoint with its request/response components.

pub mod weave;
pub mod multimodal;
pub mod models;
pub mod health;

// Re-export for convenience
pub use weave::{
    WeaveRequest, CourseResponse, MessageIntentResponse, SegmentResponse,
    WeaveUnitResponse, ClassificationResponse, PhysicsHintResponse,
    EntityResponse, PhysicsResponse,
};
pub use multimodal::{
    WeaveBlock, TextBlock, ImageBlock, AudioBlock, DocumentBlock, WebpageBlock,
    MediaRef, MediaType,
};
pub use models::{ModelInfo, ModelsResponse, get_models_response};
pub use health::HealthResponse;
