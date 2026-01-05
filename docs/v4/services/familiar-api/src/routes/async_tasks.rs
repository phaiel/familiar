//! Async Tasks API Routes
//!
//! Handles polling for async task status and results.
//! Used by UI to track progress of async Kafka command executions.

use utoipa::ToSchema;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::{EntityTrait, QueryOrder, ColumnTrait, QueryFilter, QuerySelect, Set, JsonValue};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tracing::info;

use familiar_core::entities::db::task::async_task::{
    self, Entity as AsyncTask, ActiveModel as AsyncTaskActiveModel, 
    Column as AsyncTaskColumn, TaskStatus as DbTaskStatus,
};
use familiar_core::primitives::{TaskId, UserId, TenantId};

use crate::state::AppState;
use super::ErrorResponse;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<DbTaskStatus> for TaskStatus {
    fn from(status: DbTaskStatus) -> Self {
        match status {
            DbTaskStatus::Pending => TaskStatus::Pending,
            DbTaskStatus::Queued => TaskStatus::Queued,
            DbTaskStatus::Running => TaskStatus::Running,
            DbTaskStatus::Completed => TaskStatus::Completed,
            DbTaskStatus::Failed => TaskStatus::Failed,
            DbTaskStatus::Cancelled => TaskStatus::Cancelled,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskPollResponse {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub task_type: String,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<async_task::Model> for TaskPollResponse {
    fn from(model: async_task::Model) -> Self {
        Self {
            task_id: model.id.as_uuid(),
            status: model.status.into(),
            task_type: model.task_type,
            correlation_id: model.correlation_id,
            output: model.output.map(|j| serde_json::Value::from(j)),
            error_message: model.error_message,
            attempt_count: model.attempt_count,
            created_at: model.created_at,
            started_at: model.started_at,
            completed_at: model.completed_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub input: serde_json::Value,
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskCreatedResponse {
    pub task_id: Uuid,
    pub correlation_id: String,
    pub status: TaskStatus,
    pub poll_url: String,
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    None
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/tasks/:id - Poll task status
pub async fn get_task_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> impl IntoResponse {
    info!(task_id = %task_id, "Polling task status");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Validate session (optional - may want public polling for some tasks)
    let _token = extract_session_token(&headers);

    // Query task using SeaORM
    let task = match AsyncTask::find_by_id(task_id)
        .one(store.db())
        .await
    {
        Ok(Some(task)) => task,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "Task not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    Json(TaskPollResponse::from(task)).into_response()
}

/// POST /api/tasks - Create a new async task (internal use)
/// 
/// This is typically called internally by other routes when they want to
/// trigger an async operation. The route creates the task record and
/// triggers the appropriate Windmill flow.
pub async fn create_task_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    info!(task_type = %req.task_type, "Creating async task");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Validate session
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    let token_hash = hash_token(&token);
    let user_id = match store.validate_session(&token_hash).await {
        Ok(Some(id)) => id,
        _ => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid session".to_string(),
                code: "INVALID_SESSION".to_string(),
            })).into_response();
        }
    };

    // Validate task type
    match req.task_type.as_str() {
        "onboarding.signup" | "onboarding.magic_link" | "onboarding.create_family" | "onboarding.accept_invitation" => {},
        _ => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: format!("Unknown task type: {}", req.task_type),
                code: "UNKNOWN_TASK_TYPE".to_string(),
            })).into_response();
        }
    };

    // Generate correlation ID
    let correlation_id = Uuid::new_v4().to_string();
    let task_id = TaskId::new();

    // Create task record using SeaORM
    let new_task = AsyncTaskActiveModel {
        id: Set(task_id),
        task_type: Set(req.task_type.clone()),
        correlation_id: Set(correlation_id.clone()),
        status: Set(DbTaskStatus::Queued), // Tasks go directly to queued (processed via Kafka)
        input: Set(Some(JsonValue::from(req.input))),
        output: Set(None),
        error_message: Set(None),
        attempt_count: Set(0),
        user_id: Set(UserId::from(user_id)),
        tenant_id: Set(req.tenant_id.map(TenantId::from)),
        created_at: Set(chrono::Utc::now()),
        started_at: Set(None),
        completed_at: Set(None),
        version: Set(0), // Initial version for optimistic locking
    };

    match AsyncTask::insert(new_task).exec(store.db()).await {
        Ok(_) => {},
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "CREATE_TASK_FAILED".to_string(),
            })).into_response();
        }
    }

    info!(task_id = %task_id, task_type = %req.task_type, "Task created - will be processed via Kafka");

    let task_uuid = task_id.as_uuid();
    Json(TaskCreatedResponse {
        task_id: task_uuid,
        correlation_id,
        status: TaskStatus::Queued,
        poll_url: format!("/api/tasks/{}", task_uuid),
    }).into_response()
}

/// GET /api/tasks - List tasks for current user
pub async fn list_tasks_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Validate session
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    let token_hash = hash_token(&token);
    let user_id = match store.validate_session(&token_hash).await {
        Ok(Some(id)) => id,
        _ => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid session".to_string(),
                code: "INVALID_SESSION".to_string(),
            })).into_response();
        }
    };

    // Query tasks using SeaORM
    let tasks = match AsyncTask::find()
        .filter(AsyncTaskColumn::UserId.eq(user_id))
        .order_by_desc(AsyncTaskColumn::CreatedAt)
        .limit(50)
        .all(store.db())
        .await
    {
        Ok(tasks) => tasks,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    let responses: Vec<TaskPollResponse> = tasks.into_iter().map(TaskPollResponse::from).collect();

    Json(responses).into_response()
}
