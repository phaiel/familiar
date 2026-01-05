//! Schema Analysis - ast-grep + Cross-file Rust
//!
//! ## Architecture
//!
//! Hybrid approach:
//! - ast-grep (YAML rules) for single-file pattern matching
//! - Rust for cross-file analysis requiring multi-file context
//!
//! NOTE: Schema validation (unused schemas, generation sync, drift) is handled
//! by `schema-drift` in familiar-schemas. This analyzer checks that SERVICE CODE
//! uses schemas correctly.
//!
//! ```text
//! analysis/
//! ├── mod.rs                    # This file - re-exports
//! ├── ast_grep_runner.rs        # ast-grep integration
//! ├── communication_analyzer.rs # Redpanda/Kafka communication validation
//! ├── cross_file.rs             # Cross-file Rust analyzer
//! ├── database_analyzer.rs      # SeaORM compliance in services
//! ├── fixer.rs                  # Auto-fix capabilities
//! ├── issue_kinds.rs            # Issue types, severities, stats
//! ├── kafka_analyzer.rs         # Kafka/Protobuf codegen compliance
//! ├── schema_cache.rs           # Schema name cache from familiar-schemas
//! ├── walker.rs                 # Fast file traversal
//! └── windmill_client.rs        # Windmill flow integration for LLM analysis
//!
//! ../rules/               # ast-grep YAML rules
//! ├── schema/             # Schema detection rules
//! ├── imports/            # Import validation rules
//! ├── composition/        # Composition pattern rules
//! └── kafka/              # Communication pattern rules
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::analysis::{AstGrepRunner, CrossFileAnalyzer};
//!
//! // Fast single-file patterns
//! let runner = AstGrepRunner::new(PathBuf::from("."));
//! let report = runner.run()?;
//!
//! // Cross-file checks (pattern detection, duplicate types, etc.)
//! let analyzer = CrossFileAnalyzer::new(PathBuf::from("."));
//! let report = analyzer.analyze();
//! ```

mod ast_grep_runner;
mod communication_analyzer;
mod cross_file;
mod database_analyzer;
mod fixer;
mod issue_kinds;
mod kafka_analyzer;
mod schema_cache;
mod walker;
#[cfg(feature = "windmill")]
mod windmill_client;

// Re-export public types
pub use ast_grep_runner::{AstGrepError, AstGrepRunner};
pub use communication_analyzer::{CommunicationAnalyzer, CommunicationSummary, MigrationGuide};
pub use cross_file::{CrossFileAnalyzer, OrphanStats};
pub use fixer::{AutoFixer, FixResult, FixSummary};
pub use issue_kinds::{
    AnalysisReport, DeriveCoverage, DeriveInfo, Fix, Issue, IssueKind, OrphanRecommendation, Severity, Stats,
};
pub use database_analyzer::DatabaseAnalyzer;
pub use kafka_analyzer::{KafkaAnalyzer, check_worker_kafka_compliance};
pub use schema_cache::SchemaCache;
#[cfg(feature = "windmill")]
pub use windmill_client::{WindmillClient, AnalysisRequest, AnalysisResponse};
pub use walker::{FastWalker, FileType, SourceFile};
