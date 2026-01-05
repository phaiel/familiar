//! Familiar Entity API Routes
//!
//! Routes for managing familiar entities (MOments, PULSEs, etc.)
//!
//! Routes:
//! - GET /api/tenants/:id/entities - List entities for a tenant
//! - POST /api/entities - Create a new entity
//! - PATCH /api/entities/:id/status - Update entity status (HILT)

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::info;
use utoipa::ToSchema;

use familiar_core::types::{
    FamiliarEntity, CreateEntityInput, UpdateEntityStatusInput,
    FamiliarEntityType, EntityStatus, ListEntitiesOptions,
};

use crate::state::AppState;
use super::{ErrorResponse, SuccessResponse};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListEntitiesQuery {
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct EntityListResponse {
    pub entities: Vec<FamiliarEntity>,
    pub count: usize,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/tenants/:tenant_id/entities - List entities for a tenant
pub async fn list_entities_handler(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<ListEntitiesQuery>,
) -> impl IntoResponse {
    info!(tenant_id = %tenant_id, "Listing entities");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    let entity_type = query.entity_type.and_then(|et| match et.to_uppercase().as_str() {
        "MOMENT" => Some(FamiliarEntityType::Moment),
        "PULSE" => Some(FamiliarEntityType::Pulse),
        "INTENT" => Some(FamiliarEntityType::Intent),
        "THREAD" => Some(FamiliarEntityType::Thread),
        "BOND" => Some(FamiliarEntityType::Bond),
        "MOTIF" => Some(FamiliarEntityType::Motif),
        "FILAMENT" => Some(FamiliarEntityType::Filament),
        "FOCUS" => Some(FamiliarEntityType::Focus),
        _ => None,
    });

    let status = query.status.and_then(|s| match s.as_str() {
        "pending" => Some(EntityStatus::Pending),
        "approved" => Some(EntityStatus::Approved),
        "rejected" => Some(EntityStatus::Rejected),
        "auto_spawned" => Some(EntityStatus::AutoSpawned),
        _ => None,
    });

    let options = ListEntitiesOptions {
        entity_type,
        status,
        limit: query.limit,
    };

    match store.get_familiar_entities(tenant_id, options).await {
        Ok(entities) => {
            let count = entities.len();
            Json(EntityListResponse { entities, count }).into_response()
        }
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// POST /api/entities - Create a new entity
pub async fn create_entity_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateEntityInput>,
) -> impl IntoResponse {
    info!(tenant_id = %input.tenant_id, entity_type = ?input.entity_type, "Creating entity");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    match store.create_familiar_entity(input).await {
        Ok(entity) => Json(entity).into_response(),
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// PATCH /api/entities/:id/status - Update entity status (HILT approval/rejection)
pub async fn update_entity_status_handler(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Json(input): Json<UpdateEntityStatusInput>,
) -> impl IntoResponse {
    info!(entity_id = %entity_id, status = ?input.status, "Updating entity status");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    match store.update_entity_status(entity_id, input).await {
        Ok(()) => {
            Json(SuccessResponse {
                success: true,
                message: Some("Entity status updated".to_string()),
            }).into_response()
        }
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}
