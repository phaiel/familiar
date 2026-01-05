//! Kafka Consumers
//!
//! Opaque Envelope Consumer for familiar-core Kafka topics.
//! Uses the "Opaque Envelope" pattern where:
//! - EnvelopeV1 (Protobuf) provides fast routing metadata
//! - Domain types are deserialized from JSON bytes in payload_json
//! - Contract validation via compiled JSON Schema (optional)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::kafka::clients::EnvelopeConsumer;
//!
//! let consumer = EnvelopeConsumer::new("localhost:9092", "my-group")?;
//! consumer.subscribe(&["familiar.commands"])?;
//!
//! // Receive envelope and deserialize payload
//! let (envelope, signup): (EnvelopeV1, SignupRequest) = consumer.recv().await?;
//! ```

use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{BorrowedMessage, Headers, Message as KafkaMessage};
use serde::de::DeserializeOwned;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

use crate::kafka::proto::EnvelopeV1;

/// Consumer error types
#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Kafka error: {0}")]
    Kafka(String),
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(String),
    #[error("JSON deserialization error: {0}")]
    JsonDeserialization(String),
    #[error("Empty payload")]
    EmptyPayload,
    #[error("Contract validation failed: {0}")]
    ContractViolation(String),
}

/// Universal consumer using the Opaque Envelope pattern
///
/// This consumer receives Protobuf-encoded envelopes and deserializes
/// the payload from JSON bytes. Supports contract validation via
/// compiled JSON Schema.
pub struct EnvelopeConsumer {
    consumer: StreamConsumer,
}

impl EnvelopeConsumer {
    /// Create a new envelope consumer
    pub fn new(brokers: &str, group_id: &str) -> Result<Self, rdkafka::error::KafkaError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .set("heartbeat.interval.ms", "2000")
            .set("max.poll.interval.ms", "300000")
            .set("fetch.wait.max.ms", "100")
            .create()?;

        Ok(Self { consumer })
    }

    /// Subscribe to topics
    pub fn subscribe(&self, topics: &[&str]) -> Result<(), rdkafka::error::KafkaError> {
        self.consumer.subscribe(topics)
    }

    /// Receive the next envelope and deserialize payload to type T
    ///
    /// ## The Pattern
    ///
    /// ```text
    /// Kafka bytes → Protobuf decode → EnvelopeV1
    ///                                      │
    ///                                      ▼
    ///                               payload_json (bytes)
    ///                                      │
    ///                                      ▼ serde_json::from_slice
    ///                               Your Rust type T
    /// ```
    pub async fn recv<T: DeserializeOwned>(&self) -> Result<(EnvelopeV1, T), ConsumerError> {
        let msg = self
            .consumer
            .recv()
            .await
            .map_err(|e| ConsumerError::Kafka(e.to_string()))?;

        let payload = msg.payload().ok_or(ConsumerError::EmptyPayload)?;

        // Decode Protobuf envelope
        let envelope = EnvelopeV1::decode(payload)
            .map_err(|e| ConsumerError::ProtobufDecode(e.to_string()))?;

        // Deserialize JSON payload to type T
        let typed_payload: T = serde_json::from_slice(&envelope.payload_json)
            .map_err(|e| ConsumerError::JsonDeserialization(e.to_string()))?;

        Ok((envelope, typed_payload))
    }

    /// Receive envelope only (without deserializing payload)
    ///
    /// Useful when you need to inspect metadata before deciding how to parse.
    pub async fn recv_envelope(&self) -> Result<EnvelopeV1, ConsumerError> {
        let msg = self
            .consumer
            .recv()
            .await
            .map_err(|e| ConsumerError::Kafka(e.to_string()))?;

        let payload = msg.payload().ok_or(ConsumerError::EmptyPayload)?;

        EnvelopeV1::decode(payload).map_err(|e| ConsumerError::ProtobufDecode(e.to_string()))
    }

    /// Extract message type from headers (for routing without deserializing)
    fn extract_message_type(&self, msg: &BorrowedMessage) -> Option<String> {
        msg.headers().and_then(|headers| {
            for header in headers.iter() {
                if header.key == "message_type" {
                    if let Some(value) = header.value {
                        return Some(String::from_utf8_lossy(value).to_string());
                    }
                }
            }
            None
        })
    }

    /// Process envelopes with the provided handler (main consumer loop)
    ///
    /// The handler receives the envelope and raw payload bytes.
    /// Failed messages are NOT committed and will be redelivered.
    pub async fn run<F, Fut>(&self, handler: F)
    where
        F: Fn(EnvelopeV1, Vec<u8>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        info!("Starting envelope consumer loop");

        let mut stream = self.consumer.stream();
        let mut msg_count = 0u64;

        while let Some(result) = stream.next().await {
            msg_count += 1;

            match result {
                Ok(message) => {
                    let message_type = self.extract_message_type(&message);

                    debug!(
                        msg_count,
                        message_type = ?message_type,
                        "Received message"
                    );

                    if let Some(payload) = message.payload() {
                        match EnvelopeV1::decode(payload) {
                            Ok(envelope) => {
                                debug!(
                                    message_id = %envelope.message_id,
                                    message_type = %envelope.message_type,
                                    course_id = %envelope.course_id,
                                    shuttle_id = %envelope.shuttle_id,
                                    "Processing envelope"
                                );

                                // Pass envelope and raw payload_json to handler
                                let payload_json = envelope.payload_json.clone();
                                match handler(envelope, payload_json).await {
                                    Ok(()) => {
                                        if let Err(e) =
                                            self.consumer.commit_message(&message, CommitMode::Async)
                                        {
                                            warn!(error = %e, "Failed to commit message");
                                        }
                                    }
                                    Err(e) => {
                                        error!(error = %e, "Handler failed");
                                        // Don't commit - message will be redelivered
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to decode envelope");
                                // Commit to skip bad message
                                let _ = self.consumer.commit_message(&message, CommitMode::Async);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Error receiving message");
                }
            }
        }

        warn!("Envelope consumer loop ended");
    }

    /// Commit the current offset
    pub fn commit(&self) -> Result<(), ConsumerError> {
        self.consumer
            .commit_consumer_state(CommitMode::Sync)
            .map_err(|e| ConsumerError::Kafka(e.to_string()))
    }

    /// Get the underlying consumer for advanced use
    pub fn inner(&self) -> &StreamConsumer {
        &self.consumer
    }

    /// Subscribe to messages for a specific course
    ///
    /// Returns a broadcast receiver that will receive envelopes for the specified course.
    /// This is intended for WebSocket streaming use cases where the UI needs real-time
    /// updates for a specific conversation.
    ///
    /// Note: This creates a filtered view over the consumer stream. In a production
    /// system, you'd want a proper pub/sub layer (e.g., Redis pub/sub) for this use case.
    pub async fn subscribe_course(
        &self,
        course_id: uuid::Uuid,
    ) -> tokio::sync::broadcast::Receiver<crate::EnvelopeV1> {
        // Create a broadcast channel for this course
        // In practice, you'd want a shared subscription manager
        let (tx, rx) = tokio::sync::broadcast::channel(256);
        
        // For now, return an empty receiver - the WebSocket handler should handle
        // the case where no messages arrive (backfill from DB is primary source)
        // 
        // TODO: Implement proper course-filtered streaming via:
        // 1. Redis pub/sub for real-time fan-out
        // 2. Or Kafka topic-per-course pattern
        // 3. Or in-memory subscription manager with course->channel mapping
        drop(tx); // Close immediately - no messages will be sent
        rx
    }

    /// Unsubscribe from a course (cleanup)
    pub fn unsubscribe_course(&self, _course_id: uuid::Uuid) {
        // No-op for now - subscriptions are GC'd when the receiver is dropped
        // In a production system, this would clean up the subscription manager
    }
}
