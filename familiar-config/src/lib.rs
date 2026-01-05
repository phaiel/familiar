//! # Familiar Config
//! 
//! Central configuration management for the Familiar platform.
//! This crate serves as the single source of truth for all "magic numbers"
//! and operational parameters used across the system.
//!
//! ## Architecture
//! 
//! The configuration is structured hierarchically:
//! - `nodes/` - Resource allocations and constraints for each node type
//! - `systems/` - Timeouts, retries, and operational parameters for systems
//! - `observability/` - Thresholds and settings for monitoring and alerting
//! - `infra/` - Infrastructure-level settings and defaults
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_config::GlobalConfig;
//!
//! // Load configuration with layered overrides
//! let config = GlobalConfig::load()?;
//!
//! // Access node resource allocations
//! let daemon_cpu = config.nodes.familiar_daemon.resources.cpu;
//!
//! // Access system timeouts
//! let gate_timeout = config.systems.fates_gate.timeouts.weave;
//! ```

pub mod config;
pub mod manifest;

pub use config::*;
pub use manifest::*;
