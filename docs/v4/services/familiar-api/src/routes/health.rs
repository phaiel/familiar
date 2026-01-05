//! Health check endpoint

use std::sync::Arc;
use axum::{extract::State, Json};
use familiar_core::entities::api::HealthResponse;

use crate::state::AppState;

/// GET /api/health - Health check
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let status = if state.has_db() {
        "healthy"
    } else {
        "degraded (no database)"
    };
    
    Json(HealthResponse {
        status: status.to_string(),
        service: "familiar-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
