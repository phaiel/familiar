//! HotState - Long-lived shared resources for the daemon
//!
//! This is the "hot" part of the hot service worker pattern.
//! All expensive initialization happens once at startup:
//! - ContractEnforcer compiles JSON schemas to DFA (~100ms saved per request)
//! - SeaORM opens connection pool with TLS (~50ms saved per request)
//!
//! The state is wrapped in Arc for safe concurrent access across activities.

use familiar_core::infrastructure::TigerDataStore;
use familiar_core::ContractEnforcer;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during state initialization
#[derive(Error, Debug)]
pub enum StateError {
    #[error("Database connection failed: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Long-lived shared resources - initialized once at startup
///
/// This struct holds all the expensive-to-create resources:
/// - Database connection pool
/// - Pre-compiled JSON schema validators
/// - TigerData store interface
///
/// By keeping these hot, we eliminate the ~185ms startup tax per request.
pub struct HotState {
    /// Database connection pool (SeaORM)
    pub db: DatabaseConnection,

    /// TigerDataStore - high-level store interface
    pub store: TigerDataStore,

    /// Contract enforcer for JSON Schema validation
    /// Schemas are compiled to DFA at startup for O(1) validation
    pub enforcer: Arc<ContractEnforcer>,
}

impl HotState {
    /// Create hot state from database URL
    ///
    /// This is expensive (~150ms) but only done once at startup.
    /// All activities share this state via Arc.
    pub async fn new(database_url: &str) -> Result<Self, StateError> {
        // 1. Compile all JSON schemas (expensive, ~100ms)
        let enforcer = Arc::new(ContractEnforcer::new());
        tracing::info!(
            schema_count = enforcer.schema_count(),
            "ContractEnforcer initialized (schemas compiled to DFA)"
        );

        // 2. Open database connection pool (expensive, ~50ms for TLS handshake)
        let db = sea_orm::Database::connect(database_url).await?;
        tracing::info!("Database connection pool opened");

        // 3. Create TigerDataStore from connection
        let store = TigerDataStore::from_connection(db.clone());

        Ok(Self { db, store, enforcer })
    }
}

/// Type alias for Arc-wrapped HotState
///
/// Use this type when passing state to activities.
/// Clone is cheap (just increments Arc refcount).
pub type SharedState = Arc<HotState>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_state_is_send_sync() {
        // Ensure SharedState can be safely shared across threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedState>();
    }
}





