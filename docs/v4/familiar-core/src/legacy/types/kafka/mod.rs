//! Kafka Message Types
//!
//! Opaque Envelope pattern types for Redpanda/Kafka communication.
//!
//! ## Architecture: The Opaque Envelope Pattern
//!
//! ```text
//! API ──▶ EnvelopeV1{payload_json} ──▶ familiar.commands ──▶ Worker
//! ```
//!
//! - EnvelopeV1 (Protobuf) provides routing metadata
//! - Domain types live as JSON bytes in payload_json
//! - Kafka routes on metadata WITHOUT parsing payload
//!
//! ## Schema Landscape
//!
//! - **Protobuf**: `envelope.proto` (static envelope, never changes)
//! - **JSON Schema**: `familiar-schemas/` (domain types in payload)

// Wire-to-domain mapping (deprecated - use opaque pattern)
pub mod mapping;

// === Re-exports from contracts module ===
pub use crate::types::contracts::{
    // Envelope types (JSON convenience types)
    EnvelopeV1, Payload, ProducerInfo, SchemaInfo, ENVELOPE_VERSION,
    // Topics
    topics, COMMANDS, EVENTS, TRACE, DLQ, WINDMILL_ONBOARDING,
    // Trace types
    TraceKind, TraceStatus, TracePayload,
    // Onboarding types
    SignupConsents, RequestContext,
    SignupRequest, CreateFamilyRequest, AcceptInvitationRequest,
    SignupCompleted, FamilyCreated, InvitationAccepted, OnboardingFailed,
    // Course types
    CourseStart, CourseContinue, CourseCancel, CourseRetry,
    CourseStarted, CourseSegmented, CourseClassified, CourseCompleted,
    CourseFailed, CourseCancelled, CourseRetrying,
};

// Re-export ID types from primitives
pub use familiar_primitives::{TenantId, UserId, CourseId, ShuttleId};

/// Schema version constant (same as ENVELOPE_VERSION for compatibility)
pub const SCHEMA_VERSION: u32 = 1;
