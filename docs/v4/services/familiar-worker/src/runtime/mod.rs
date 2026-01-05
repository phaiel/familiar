//! Minerva Runtime Layer
//!
//! CLI-only execution runtime for Minerva (Step Mode).
//! Windmill (the Orchestrator) invokes Minerva CLI commands.
//!
//! ## The Evaluator Pattern
//!
//! ```text
//! Windmill Flow
//!     │
//!     ▼
//! minerva onboarding evaluate-signup --input '{"email": "..."}'
//!     │
//!     ▼
//! stdout: {"next_step": "LOOM", "reason": "...", "data": {...}}
//! ```

mod step;
pub mod tower_layers;

pub use step::StepRuntime;

use crate::config::WorkerConfig;
use familiar_core::infrastructure::TigerDataStore;
use familiar_core::ContractEnforcer;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur in the runtime layer
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Input validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Contract violation: {0}")]
    Contract(#[from] familiar_core::ContractError),
    
    #[error("Store error: {0}")]
    Store(#[from] familiar_core::infrastructure::DbStoreError),
}

/// Shared resources for step runtime
pub struct SharedResources {
    /// Database connection pool (SeaORM) - raw connection for direct queries
    pub db: DatabaseConnection,

    /// TigerDataStore - high-level store interface with typed methods
    pub store: TigerDataStore,

    /// Worker configuration
    pub config: WorkerConfig,
    
    /// Contract enforcer for JSON Schema validation
    /// Validates inputs against embedded schemas before processing
    pub enforcer: Arc<ContractEnforcer>,
}

impl SharedResources {
    /// Create shared resources from configuration
    pub async fn new(config: WorkerConfig) -> Result<Self, RuntimeError> {
        // Initialize ContractEnforcer (compiles embedded schemas at startup)
        let enforcer = Arc::new(ContractEnforcer::new());
        tracing::info!(
            schema_count = enforcer.schema_count(),
            "ContractEnforcer initialized"
        );
        
        // Connect to database (required for Minerva operations)
        let db_url = config
            .database
            .as_ref()
            .map(|db| db.url.clone())
            .ok_or_else(|| RuntimeError::Config("DATABASE_URL not configured".to_string()))?;

        let db = sea_orm::Database::connect(&db_url)
            .await
            .map_err(RuntimeError::Database)?;
        
        // Create TigerDataStore from the same connection
        let store = TigerDataStore::from_connection(db.clone());

        Ok(Self { db, store, config, enforcer })
    }
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;
