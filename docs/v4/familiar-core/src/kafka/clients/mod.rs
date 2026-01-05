//! Kafka Producers and Consumers
//!
//! Typed Kafka clients using the Opaque Envelope pattern.
//!
//! ## The Pattern
//!
//! - EnvelopeV1 (Protobuf) provides routing metadata
//! - Domain types are JSON bytes in payload_json
//! - Zero mapping code required
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::kafka::clients::{EnvelopeProducer, EnvelopeConsumer};
//!
//! // Producer - send any serializable type
//! let producer = EnvelopeProducer::new("localhost:9092")?;
//! producer.send("topic", "message.type", "tenant", "user", &payload).await?;
//!
//! // Consumer - receive and deserialize to any type
//! let consumer = EnvelopeConsumer::new("localhost:9092", "my-group")?;
//! consumer.subscribe(&["familiar.commands"])?;
//! let (envelope, payload): (_, MyType) = consumer.recv().await?;
//! ```

#[cfg(feature = "kafka-codegen")]
mod consumer;
#[cfg(feature = "kafka-codegen")]
mod producer;

#[cfg(feature = "kafka-codegen")]
pub use consumer::*;
#[cfg(feature = "kafka-codegen")]
pub use producer::*;
