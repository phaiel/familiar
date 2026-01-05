//! Generated Protobuf Types
//!
//! This module includes the Rust types generated from envelope.proto.
//! The envelope is STATIC and contains only routing metadata.
//! Domain types live as opaque JSON bytes in `payload_json`.

/// Include the generated protobuf types from build.rs
pub mod envelope {
    include!(concat!(env!("OUT_DIR"), "/familiar.rs"));
}

// Re-export for convenience
pub use envelope::EnvelopeV1;







