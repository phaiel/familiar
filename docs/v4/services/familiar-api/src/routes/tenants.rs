//! Tenant/Family Routes
//!
//! Handles creating and managing tenants (families).

use utoipa::ToSchema;
use axum::{
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sea_orm::EntityTrait;
use std::sync::Arc;
use uuid::Uuid;

use familiar_core::types::kafka::{EnvelopeV1, Payload, TenantId, UserId};
use familiar_core::entities::db::tenant::Entity as Tenant;

use crate::state::AppState;
use super::ErrorResponse;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Response for async task submission
#[derive(Debug, Serialize, ToSchema)]
pub struct TaskSubmittedResponse {
    /// Task ID for polling status
    pub task_id: Uuid,
    /// URL to poll for task status
    pub poll_url: String,
    /// Human-readable message
    pub message: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Hash a token for lookup
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/tenants - Create a new tenant/family
/// 
/// Routes through Kafka/Redpanda for async processing via EnvelopeV1 pattern.
/// Falls back to Windmill if Kafka is unavailable.
pub async fn create_tenant_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateTenantRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(family_name = %req.name, "Create tenant");

    // Get auth token
    let token = extract_bearer_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "Missing authorization token".to_string(), code: "ERROR".to_string() }),
        )
    })?;

    // Get database store for session validation
    let store = match &state.store {
        Some(s) => s,
        None => return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse { error: "Database not available".to_string(), code: "ERROR".to_string() }),
        )),
    };

    // Hash the token and validate session to get user_id
    let token_hash = hash_token(&token);
    let user_id = store.validate_session(&token_hash).await.map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: format!("Session error: {}", e), code: "SESSION_ERROR".to_string() }),
        )
    })?.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "Invalid or expired session".to_string(), code: "ERROR".to_string() }),
        )
    })?;

    // === NEW: Try EnvelopeProducer first (async path) ===
    if let Some(producer) = &state.envelope_producer {
        let task_id = Uuid::new_v4();
        let placeholder_tenant_id = TenantId::new(); // Will be created by worker
        
        let envelope = EnvelopeV1::command(
            placeholder_tenant_id,
            UserId::from(user_id),
            task_id.to_string(),
            Payload::CreateFamily {
                family_name: req.name.clone(),
            },
        );
        
        match producer.send_command(&envelope).await {
            Ok(()) => {
                tracing::info!(task_id = %task_id, "CreateFamily command sent via Kafka");
                
                // Return task ID for async polling
                return Ok(Json(serde_json::json!({
                    "task_id": task_id,
                    "poll_url": format!("/api/tasks/{}", task_id),
                    "message": "Family creation request submitted. Poll the provided URL for status."
                })));
            }
            Err(e) => {
                tracing::error!("Kafka send failed: {}", e);
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse { error: "Service temporarily unavailable. Please try again.".to_string(), code: "SERVICE_UNAVAILABLE".to_string() }),
                ));
            }
        }
    }

    // No envelope producer configured
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse { error: "Service not configured".to_string(), code: "SERVICE_NOT_CONFIGURED".to_string() }),
    ))
}

/// GET /api/tenants/:id - Get tenant details
pub async fn get_tenant_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get auth token
    let _token = extract_bearer_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "Missing authorization token".to_string(), code: "ERROR".to_string() }),
        )
    })?;

    // Get database store
    let store = match &state.store {
        Some(s) => s,
        None => return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse { error: "Database not available".to_string(), code: "ERROR".to_string() }),
        )),
    };

    // Get tenant using SeaORM
    let tenant = Tenant::find_by_id(id)
        .one(store.db())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: "Database error".to_string(), code: "ERROR".to_string() }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "Tenant not found".to_string(), code: "ERROR".to_string() }),
            )
        })?;

    Ok(Json(TenantResponse {
        id: tenant.id.to_string(),
        name: tenant.name,
        created_at: tenant.created_at.to_rfc3339(),
    }))
}