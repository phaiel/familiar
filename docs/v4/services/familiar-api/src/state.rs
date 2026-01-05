//! Application State
//!
//! Holds shared resources like database connections, Kafka producers/consumers,
//! and the ContractEnforcer for JSON Schema validation.

use std::sync::Arc;
use familiar_core::infrastructure::{TigerDataStore, MediaStore};
use familiar_core::kafka::clients::{EnvelopeConsumer, EnvelopeProducer};
use familiar_core::ContractEnforcer;
use crate::kafka::KafkaConfig;
use crate::config::AppConfig;

/// Application state shared across all handlers
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,
    /// Database connection (optional - can run without DB for development)
    pub store: Option<TigerDataStore>,
    /// Media store (MinIO/S3) for multimodal content
    pub media_store: Option<MediaStore>,

    /// Universal envelope producer for all commands/events
    pub envelope_producer: Option<EnvelopeProducer>,
    /// Kafka trace consumer for receiving progress updates
    pub kafka_consumer: Option<EnvelopeConsumer>,
    
    /// Contract enforcer for JSON Schema validation
    /// Validates incoming requests against embedded schemas before processing
    pub enforcer: Arc<ContractEnforcer>,
}

impl AppState {
    /// Create new state with database and Kafka connections
    pub async fn new(app_config: AppConfig) -> Result<Self, String> {
        // Initialize ContractEnforcer (compiles embedded schemas at startup)
        let enforcer = Arc::new(ContractEnforcer::new());
        tracing::info!("✅ ContractEnforcer initialized with {} schemas", enforcer.schema_count());
        
        // Try to connect to database from config
        let store = match TigerDataStore::from_connection_string(&app_config.database.url).await {
            Ok(s) => {
                tracing::info!("✅ Connected to TigerData database");
                Some(s)
            }
            Err(e) => {
                tracing::warn!("⚠️ Database unavailable: {}", e);
                None
            }
        };
        
        // Try to connect to MediaStore from config
        let media_store_config = familiar_core::infrastructure::MediaStoreConfig {
            endpoint: app_config.media_store.endpoint.clone(),
            bucket: app_config.media_store.bucket.clone(),
            access_key: app_config.media_store.access_key.clone(),
            secret_key: app_config.media_store.secret_key.clone(),
            region: app_config.media_store.region.clone(),
        };
        let media_store_instance = MediaStore::new(media_store_config).await;
        // Ensure bucket exists
        let media_store = if let Err(e) = media_store_instance.ensure_bucket().await {
            tracing::warn!("⚠️ Media bucket unavailable: {}", e);
            None
        } else {
            tracing::info!("✅ Connected to MinIO media store");
            Some(media_store_instance)
        };
        
        // Get Kafka config from AppConfig
        let kafka_config = KafkaConfig::from_app_config(&app_config);
        
        // EnvelopeProducer (universal pattern - Protobuf envelope with JSON payload)
        let envelope_producer = match EnvelopeProducer::new(&kafka_config.bootstrap_servers) {
            Ok(producer) => {
                tracing::info!("✅ EnvelopeProducer connected to {}", kafka_config.bootstrap_servers);
                Some(producer)
            }
            Err(e) => {
                tracing::warn!("⚠️ EnvelopeProducer unavailable: {}", e);
                None
            }
        };
        
        let kafka_consumer = match EnvelopeConsumer::new(&kafka_config.bootstrap_servers, &kafka_config.group_id) {
            Ok(consumer) => {
                tracing::info!("✅ Kafka consumer connected");
                Some(consumer)
            }
            Err(e) => {
                tracing::warn!("⚠️ Kafka consumer unavailable: {}", e);
                None
            }
        };
        
        Ok(Self { 
            config: app_config, 
            store, 
            media_store, 
            envelope_producer, 
            kafka_consumer,
            enforcer,
        })
    }

    /// Create state without external services (for development/testing)
    pub fn without_db() -> Self {
        tracing::info!("🔧 Running without database or Kafka");
        let enforcer = Arc::new(ContractEnforcer::new());
        tracing::info!("✅ ContractEnforcer initialized with {} schemas", enforcer.schema_count());
        
        Self {
            config: AppConfig::default(), // This won't work, need to fix
            store: None,
            media_store: None,
            envelope_producer: None,
            kafka_consumer: None,
            enforcer,
        }
    }

    /// Check if database is available
    pub fn has_db(&self) -> bool {
        self.store.is_some()
    }
    
    /// Check if EnvelopeProducer is available
    pub fn has_envelope_producer(&self) -> bool {
        self.envelope_producer.is_some()
    }
}
