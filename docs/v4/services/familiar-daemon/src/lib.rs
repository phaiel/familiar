//! familiar-daemon - Hot Activity Worker for Temporal
//!
//! This is the "muscle" in the polyglot Temporal architecture:
//! - TypeScript workflows (the "brain") orchestrate logic flow
//! - Rust activities (the "muscle") handle high-performance data processing
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    familiar-daemon                               │
//! │                                                                  │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │  HotState (Arc<...>)                                      │   │
//! │  │    ├── ContractEnforcer (SIMD-JSON + compiled schemas)    │   │
//! │  │    └── SeaORM Connection Pool (warm TCP/TLS)              │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! │                              │                                   │
//! │                              ▼                                   │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │  Activities                                               │   │
//! │  │    ├── FatesGate (classification)                         │   │
//! │  │    ├── FatesMorta (segmentation)                          │   │
//! │  │    ├── FatesDecima (entity extraction)                    │   │
//! │  │    ├── FatesNona (response generation)                    │   │
//! │  │    └── FatesPipeline (all stages)                         │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼ gRPC (polls task queue)
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Temporal Server                             │
//! │                       localhost:7233                             │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Performance Gains
//!
//! By keeping resources "hot" (initialized once at startup), we eliminate:
//! - ~100ms schema compilation per request
//! - ~50ms database TLS handshake per request
//! - ~30ms process spawn overhead per request
//!
//! Total savings: ~180ms per request (from ~185ms to ~5ms overhead)

pub mod activities;
pub mod config;
pub mod state;

pub use config::DaemonConfig;
pub use state::{HotState, SharedState};





