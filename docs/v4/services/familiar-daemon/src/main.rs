//! familiar-daemon - Temporal Activity Worker
//!
//! Hot service worker for the Fates/Loom pipeline.
//! Initializes expensive resources once, then polls for activity tasks.
//!
//! ## Usage
//!
//! ```bash
//! # Start with environment variables
//! DATABASE_URL=postgres://... cargo run -p familiar-daemon
//!
//! # Or with .env file
//! cargo run -p familiar-daemon
//! ```
//!
//! ## Environment Variables
//!
//! - `DATABASE_URL` - PostgreSQL connection string (required)
//! - `TEMPORAL_URL` - Temporal server URL (default: http://localhost:7233)
//! - `TEMPORAL_NAMESPACE` - Temporal namespace (default: "default")
//! - `TEMPORAL_TASK_QUEUE` - Task queue name (default: "fates-pipeline")
//! - `RUST_LOG` - Log level filter

use std::sync::Arc;

use familiar_daemon::{
    activities::register_fates_activities,
    config::DaemonConfig,
    state::HotState,
};

use temporalio_sdk::Worker;
use temporalio_sdk_core::{init_worker, CoreRuntime, RuntimeOptions, Url};
use temporalio_client::ClientOptions;
use temporalio_common::{
    worker::{WorkerConfig, WorkerTaskTypes, WorkerVersioningStrategy},
    telemetry::TelemetryOptions,
};

use anyhow::{Context, Result};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = DaemonConfig::from_env()
        .context("Failed to load configuration")?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .with_target(false)
        .init();

    info!("familiar-daemon starting...");
    info!("  Temporal URL: {}", config.temporal_url);
    info!("  Namespace: {}", config.temporal_namespace);
    info!("  Task Queue: {}", config.task_queue);

    // ==========================================================================
    // HOT START - Initialize expensive resources ONCE
    // ==========================================================================
    
    info!("Initializing HotState (schema compilation + DB pool)...");
    let hot_state = Arc::new(
        HotState::new(&config.database_url)
            .await
            .context("Failed to initialize HotState")?
    );
    info!("HotState initialized - {} schemas compiled", hot_state.enforcer.schema_count());

    // ==========================================================================
    // Connect to Temporal Server
    // ==========================================================================
    
    info!("Connecting to Temporal server...");
    
    let telemetry_options = TelemetryOptions::builder().build();
    let runtime_options = RuntimeOptions::builder()
        .telemetry_options(telemetry_options)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build runtime options: {}", e))?;
    let runtime = CoreRuntime::new_assume_tokio(runtime_options)
        .context("Failed to create Temporal runtime")?;

    let temporal_url: Url = config.temporal_url.parse()
        .context("Invalid Temporal URL")?;
    
    let server_options = ClientOptions::builder()
        .target_url(temporal_url)
        .client_name("familiar-daemon".to_string())
        .client_version(env!("CARGO_PKG_VERSION").to_string())
        .identity(format!("familiar-daemon@{}", gethostname::gethostname().to_string_lossy()))
        .build();
    let client = server_options
        .connect(&config.temporal_namespace, None)
        .await
        .context("Failed to connect to Temporal server")?;

    info!("Connected to Temporal server");

    // ==========================================================================
    // Create Worker
    // ==========================================================================
    
    let worker_config = WorkerConfig::builder()
        .namespace(&config.temporal_namespace)
        .task_queue(&config.task_queue)
        .task_types(WorkerTaskTypes::activity_only())
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: env!("CARGO_PKG_VERSION").to_string(),
        })
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build worker config: {}", e))?;

    let core_worker = init_worker(&runtime, worker_config, client)
        .context("Failed to initialize worker")?;
    
    let mut worker = Worker::new_from_core(Arc::new(core_worker), &config.task_queue);

    // ==========================================================================
    // Register Activities
    // ==========================================================================
    
    register_fates_activities(&mut worker, hot_state);

    // ==========================================================================
    // Run Worker (polls indefinitely)
    // ==========================================================================
    
    info!("🚀 familiar-daemon online - polling {} queue", config.task_queue);
    info!("   Schema compilation: PRE-COMPILED (0ms per request)");
    info!("   DB connections: POOLED (0ms per request)");
    info!("   JSON parsing: SIMD-accelerated");
    info!("");
    info!("Waiting for activity tasks from TypeScript workflow worker...");

    worker.run().await
        .context("Worker run failed")?;

    Ok(())
}

