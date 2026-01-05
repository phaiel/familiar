//! Media endpoints - Upload and manage binary content
//!
//! Provides endpoints for:
//! - Uploading media (images, audio, docs)
//! - Getting presigned URLs for viewing
//! - Deleting media

use std::sync::Arc;
use axum::{
    extract::{State, Path, Multipart},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::state::AppState;
use familiar_core::entities::api::multimodal::MediaType;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    pub key: String,
    pub url: String,
    pub media_type: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/media/upload - Upload a file
pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Check if media store is configured
    let media_store = match &state.media_store {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Media store not configured").into_response(),
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        let name_opt = field.name().map(|s| s.to_string());
        let name = name_opt.as_deref().unwrap_or("file");
        
        if name == "file" {
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let file_name = field.file_name().unwrap_or("upload").to_string();
            
            // Determine media type from content type
            let media_type = if content_type.starts_with("image/") {
                MediaType::Image
            } else if content_type.starts_with("audio/") {
                MediaType::Audio
            } else {
                MediaType::Document
            };
            
            // Generate key: {uuid}-{filename}
            let key = format!("{}-{}", uuid::Uuid::new_v4(), file_name);
            
            // Read data
            let data_bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => return (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)).into_response(),
            };
            let data = data_bytes.to_vec();
            
            // Upload to MinIO
            if let Err(e) = media_store.upload(&key, data, &content_type).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Upload failed: {}", e)).into_response();
            }
            
            // Get presigned URL for immediate display
            let url = match media_store.get_presigned_url(&key, std::time::Duration::from_secs(3600)).await {
                Ok(u) => u,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate URL: {}", e)).into_response(),
            };
            
            return Json(UploadResponse {
                key,
                url,
                media_type: format!("{:?}", media_type),
            }).into_response();
        }
    }

    (StatusCode::BAD_REQUEST, "No file found in request").into_response()
}

/// GET /api/media/:key - Get a presigned URL for a file
pub async fn get_url_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let media_store = match &state.media_store {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Media store not configured").into_response(),
    };

    match media_store.get_presigned_url(&key, std::time::Duration::from_secs(3600)).await {
        Ok(url) => Json(serde_json::json!({ "url": url })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate URL: {}", e)).into_response(),
    }
}

/// DELETE /api/media/:key - Delete a file
pub async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let media_store = match &state.media_store {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Media store not configured").into_response(),
    };

    match media_store.delete(&key).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete failed: {}", e)).into_response(),
    }
}
