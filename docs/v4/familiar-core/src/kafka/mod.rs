//! Kafka Integration Module
//!
//! Provides typed producers and consumers using the Opaque Envelope pattern.
//!
//! ## Architecture
//!
//! ```text
//! Domain Structs (JSON-serializable)
//!     │
//!     ▼ serde_json::to_vec()
//! EnvelopeV1.payload_json (opaque bytes)
//!     │
//!     ▼ prost encode
//! Kafka/Redpanda (routes on metadata, never parses payload)
//! ```
//!
//! ## The Opaque Envelope Pattern
//!
//! - Protobuf is used ONLY for the envelope (shipping label)
//! - Domain types live inside `payload_json` as opaque JSON bytes
//! - Kafka can route on metadata without deserializing the payload
//! - Domain schemas are the source of truth in familiar-schemas

pub mod proto;
pub mod clients;

/// Wire format version for Confluent Schema Registry
pub const WIRE_FORMAT_VERSION: u8 = 0;
