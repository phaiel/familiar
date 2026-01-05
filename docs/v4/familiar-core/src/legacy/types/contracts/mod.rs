//! Contract Types (Domain Payloads)
//!
//! These are the domain types that go INSIDE the opaque `payload_json` field.
//! They match the JSON Schema definitions in familiar-schemas.
//!
//! ## The Opaque Envelope Pattern
//!
//! EnvelopeV1 (Protobuf) contains:
//! - Routing metadata (message_type, tenant_id, etc.)
//! - payload_json: bytes containing JSON of these types
//!
//! Kafka never parses these - only Rust/Windmill do.

pub mod course;
pub mod envelope;
pub mod onboarding;
pub mod trace;
pub mod topics;

// Re-exports for convenience
pub use course::*;
pub use envelope::*;
pub use onboarding::*;
pub use trace::*;
pub use topics::*;

