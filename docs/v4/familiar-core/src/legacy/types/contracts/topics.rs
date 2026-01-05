//! Kafka Topic Names
//!
//! Topic names for Kafka/Redpanda communication.

/// All commands (EnvelopeV1 with command payloads)
pub const COMMANDS: &str = "familiar.commands";

/// All events (EnvelopeV1 with event payloads)
pub const EVENTS: &str = "familiar.events";

/// All traces (EnvelopeV1 with trace payloads)
pub const TRACE: &str = "familiar.trace";

/// Dead letter queue for failed messages
pub const DLQ: &str = "familiar.dlq";

/// Windmill onboarding topic
pub const WINDMILL_ONBOARDING: &str = "windmill.onboarding";






