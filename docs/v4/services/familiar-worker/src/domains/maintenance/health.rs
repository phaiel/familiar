//! Health - System Health Check
//!
//! Verifies connectivity to database, etc.

use crate::runtime::SharedResources;
use sea_orm::ConnectionTrait;
use tracing::{debug, info};

/// Execute health check
pub async fn execute(resources: &SharedResources) -> Result<String, String> {
    debug!("Starting health check");

    // Check database connectivity
    let db_status = match resources.db.execute_unprepared("SELECT 1").await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let result = serde_json::json!({
        "status": if db_status == "healthy" { "healthy" } else { "unhealthy" },
        "checks": {
            "database": db_status
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    info!(status = db_status, "Health check complete");

    serde_json::to_string(&result).map_err(|e| e.to_string())
}
