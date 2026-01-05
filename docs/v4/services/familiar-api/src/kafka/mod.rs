//! Kafka/Redpanda Integration
//!
//! This module provides Kafka configuration.
//! 
//! Producer and consumer implementations are now in `familiar_core::kafka::clients`:
//!
//! ```rust,ignore
//! use familiar_core::kafka::clients::{
//!     CourseCommandProducer,
//!     CourseTraceConsumer,
//! };
//! ```

pub mod config;

pub use self::config::KafkaConfig;
