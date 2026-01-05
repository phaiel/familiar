//! Minerva - The Master Weaver
//!
//! CLI-only executor for the Familiar system.
//! Windmill (the Orchestrator) invokes Minerva CLI commands.
//!
//! # Course-Thread Architecture
//!
//! - **Course**: Persistent session/history bucket (owned by database)
//! - **Shuttle**: Transient unit of work (carried by Minerva)
//! - **Thread**: THREAD entity (Person/Concept) - protected domain term
//!
//! # The Evaluator Pattern
//!
//! Every evaluate command returns an EvaluationResult:
//! ```json
//! {
//!   "next_step": "LOOM",
//!   "reason": "Input requires AI classification",
//!   "data": { ... opaque blob ... }
//! }
//! ```
//!
//! Windmill branches on `next_step` (LOOM, DIRECT, REJECT, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   Orchestrator (Windmill)                       │
//! │  Windmill Flow: signup_flow.yaml                               │
//! │    ├── minerva onboarding evaluate-signup --input {...}        │
//! │    ├── if next_step == "LOOM": continue                        │
//! │    └── minerva onboarding execute-signup --input {...}         │
//! └─────────────────────────────────────────────────────────────────┘
//!                                 │
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     Minerva (familiar-worker)                   │
//! │ ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
//! │ │   Fates     │  │ Onboarding  │  │  Manifold   │  Domains     │
//! │ └─────────────┘  └─────────────┘  └─────────────┘              │
//! │ ┌─────────────────────────────────────────────────┐            │
//! │ │              StepRuntime (CLI-only)             │            │
//! │ │  JSON in → Evaluate/Execute → JSON out         │            │
//! │ └─────────────────────────────────────────────────┘            │
//! └─────────────────────────────────────────────────────────────────┘
//!                                 │
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   Broker (Kafka/Redpanda)                       │
//! │  EnvelopeV1 { course_id, shuttle_id, payload_json }            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

// Configuration
pub mod config;

// Minerva modules
pub mod cli;
pub mod runtime;
pub mod domains;

// The Evaluator Pattern (replaces Decider/routing)
pub mod evaluator;
