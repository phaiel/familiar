//! Kafka Producers
//!
//! Opaque Envelope Producer for familiar-core Kafka topics.
//! Uses the "Opaque Envelope" pattern where:
//! - EnvelopeV1 (Protobuf) provides fast routing metadata
//! - Domain types are serialized as JSON bytes in payload_json
//! - Zero mapping code required
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::kafka::clients::EnvelopeProducer;
//!
//! let producer = EnvelopeProducer::new("localhost:9092")?;
//!
//! // Send any serializable payload - no mapping needed
//! let signup = SignupRequest { email: "...", ... };
//! producer.send("familiar.commands", "onboarding.signup", &tenant_id, &signup).await?;
//! ```

use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

use crate::kafka::proto::EnvelopeV1;

/// Producer error types
#[derive(Debug, thiserror::Error)]
pub enum ProducerError {
    #[error("Kafka error: {0}")]
    Kafka(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Universal producer using the Opaque Envelope pattern
///
/// This producer sends any serializable type as JSON bytes inside
/// a Protobuf envelope. Kafka can route on envelope metadata without
/// ever parsing the payload.
///
/// ## The Pattern
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │ EnvelopeV1 (Protobuf)                   │
/// ├─────────────────────────────────────────┤
/// │ message_type: "onboarding.signup"       │  ← Kafka routes on this
/// │ tenant_id: "uuid-..."                   │
/// │ correlation_id: "ulid-..."              │
/// ├─────────────────────────────────────────┤
/// │ payload_json: {"email": "...", ...}     │  ← Opaque bytes
/// └─────────────────────────────────────────┘
/// ```
pub struct EnvelopeProducer {
    producer: Arc<FutureProducer>,
}

impl EnvelopeProducer {
    /// Create a new envelope producer
    pub fn new(brokers: &str) -> Result<Self, rdkafka::error::KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .set("enable.idempotence", "true")
            .create()?;

        Ok(Self {
            producer: Arc::new(producer),
        })
    }

    /// Send any serializable payload - zero mapping code
    ///
    /// The payload is serialized to JSON bytes and wrapped in a Protobuf envelope.
    /// Kafka can route on `message_type` and `tenant_id` without parsing the payload.
    ///
    /// # Arguments
    /// - `topic`: Kafka topic to send to
    /// - `message_type`: Message type for routing (e.g., "onboarding.signup")
    /// - `tenant_id`: Tenant (family) context
    /// - `course_id`: Session context (persistent history bucket)
    /// - `shuttle_id`: Job context (transient unit of work)
    /// - `payload`: Any serializable type
    pub async fn send<T: Serialize>(
        &self,
        topic: &str,
        message_type: &str,
        tenant_id: &str,
        course_id: &str,
        shuttle_id: &str,
        payload: &T,
    ) -> Result<(), ProducerError> {
        // Serialize payload to JSON bytes (THE MAGIC - no mapping needed)
        let payload_json = serde_json::to_vec(payload)
            .map_err(|e| ProducerError::Serialization(e.to_string()))?;

        // Create the envelope with routing metadata
        let envelope = EnvelopeV1 {
            message_id: ulid::Ulid::new().to_string(),
            tenant_id: tenant_id.to_string(),
            course_id: course_id.to_string(),
            shuttle_id: shuttle_id.to_string(),
            message_type: message_type.to_string(),
            payload_json,
        };

        // Encode to Protobuf bytes
        let bytes = envelope.encode_to_vec();

        // Build Kafka key for partition affinity: {tenant_id}:{course_id}
        let key = format!("{}:{}", tenant_id, course_id);

        // Build headers for routing without deserializing
        let headers = OwnedHeaders::new()
            .insert(rdkafka::message::Header {
                key: "message_type",
                value: Some(message_type.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "course_id",
                value: Some(course_id.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "tenant_id",
                value: Some(tenant_id.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "content_type",
                value: Some(b"application/x-protobuf"),
            });

        let record = FutureRecord::to(topic)
            .key(&key)
            .headers(headers)
            .payload(&bytes);

        self.producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(e, _)| ProducerError::Kafka(e.to_string()))?;

        Ok(())
    }

    /// Send to a specific topic (convenience method)
    /// Generates new course_id and shuttle_id automatically
    pub async fn send_to_topic<T: Serialize>(
        &self,
        topic: &str,
        message_type: &str,
        tenant_id: &str,
        payload: &T,
    ) -> Result<(), ProducerError> {
        let course_id = ulid::Ulid::new().to_string();
        let shuttle_id = ulid::Ulid::new().to_string();
        self.send(topic, message_type, tenant_id, &course_id, &shuttle_id, payload)
            .await
    }

    /// Send an already-constructed JSON envelope
    ///
    /// This is a convenience method for API services that have already built
    /// the JSON EnvelopeV1 type. The envelope is serialized and wrapped in
    /// the Protobuf wire format.
    pub async fn send_envelope(
        &self,
        topic: &str,
        envelope: &crate::EnvelopeV1,
    ) -> Result<(), ProducerError> {
        // Serialize the payload from the JSON envelope
        let payload_json = serde_json::to_vec(&envelope.payload)
            .map_err(|e| ProducerError::Serialization(e.to_string()))?;

        // Extract IDs from the JSON envelope
        let tenant_id = envelope.tenant_id.to_string();
        let course_id = envelope.course_id.map(|c| c.to_string()).unwrap_or_else(|| envelope.correlation_id.clone());
        let shuttle_id = envelope.shuttle_id.map(|s| s.to_string()).unwrap_or_else(|| ulid::Ulid::new().to_string());

        // Create the Protobuf envelope with routing metadata
        let proto_envelope = EnvelopeV1 {
            message_id: envelope.message_id.clone(),
            tenant_id: tenant_id.clone(),
            course_id: course_id.clone(),
            shuttle_id: shuttle_id.clone(),
            message_type: envelope.message_type.clone(),
            payload_json,
        };

        // Encode to Protobuf bytes
        let bytes = proto_envelope.encode_to_vec();

        // Build Kafka key for partition affinity: {tenant_id}:{course_id}
        let key = format!("{}:{}", tenant_id, course_id);

        // Build headers for routing without deserializing
        let headers = OwnedHeaders::new()
            .insert(rdkafka::message::Header {
                key: "message_type",
                value: Some(envelope.message_type.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "course_id",
                value: Some(course_id.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "tenant_id",
                value: Some(tenant_id.as_bytes()),
            })
            .insert(rdkafka::message::Header {
                key: "content_type",
                value: Some(b"application/x-protobuf"),
            });

        let record = FutureRecord::to(topic)
            .key(&key)
            .headers(headers)
            .payload(&bytes);

        self.producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(e, _)| ProducerError::Kafka(e.to_string()))?;

        Ok(())
    }

    /// Legacy alias for send_envelope (deprecated)
    #[deprecated(since = "0.2.0", note = "Use send_envelope instead")]
    pub async fn send_command(
        &self,
        envelope: &crate::EnvelopeV1,
    ) -> Result<(), ProducerError> {
        self.send_envelope("familiar.commands", envelope).await
    }
}

impl Clone for EnvelopeProducer {
    fn clone(&self) -> Self {
        Self {
            producer: Arc::clone(&self.producer),
        }
    }
}
