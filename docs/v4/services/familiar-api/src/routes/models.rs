//! Models endpoint

use axum::Json;
use familiar_core::entities::api::{ModelsResponse, get_models_response};

/// GET /api/models - List available AI models
pub async fn get_models() -> Json<ModelsResponse> {
    Json(get_models_response())
}





