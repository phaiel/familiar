//! Agentic API Routes
//!
//! Event/Command pattern routes for the multi-agent orchestration system.
//! Uses Kafka EnvelopeV1 for async command processing.
//!
//! Course-Thread Architecture:
//! - course_id: The persistent session/history bucket
//! - shuttle_id: The transient unit of work
//! - thread_id: Reserved for THREAD entity (Person/Concept) - NOT used in API
//!
//! Routes:
//! - POST /api/agentic/command - Send command to agentic system
//! - GET /api/agentic/courses/:id - Get course by ID
//! - WS /api/agentic/stream - Stream agentic events (future)

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use utoipa::ToSchema;
use tracing::info;
use uuid::Uuid;

use familiar_core::types::AgenticCommand;
use familiar_core::types::kafka::{EnvelopeV1, Payload, TenantId, UserId, CourseId, ShuttleId};

use crate::state::AppState;
use super::ErrorResponse;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Response for accepted command
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CommandAcceptedResponse {
    pub accepted: bool,
    pub job_id: String,
    pub course_id: String,
    pub shuttle_id: String,
    pub ws_url: String,
    pub message: String,
}

/// Course summary for list view
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CourseSummaryResponse {
    pub id: String,
    pub preview: String,
    pub message_count: usize,
    pub status: String,
    pub updated_at: String,
}

/// Course detail response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CourseResponse {
    pub id: String,
    pub tenant_id: String,
    pub status: String,
    pub messages: Vec<CourseMessageResponse>,
    pub created_at: String,
    pub updated_at: String,
}

/// Message in a course
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CourseMessageResponse {
    pub id: String,
    pub role: String,
    pub content: Option<String>,
    pub agent_speaker: Option<String>,
    pub shuttle_id: Option<String>,
    pub timestamp: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/agentic/command - Send command to agentic system
///
/// Accepts an AgenticCommand and emits to Kafka for async processing.
/// Returns a job ID for async tracking via WebSocket.
pub async fn command_handler(
    State(state): State<Arc<AppState>>,
    Json(command): Json<AgenticCommand>,
) -> impl IntoResponse {
    info!("Received agentic command");

    // Check EnvelopeProducer availability
    let producer = match &state.envelope_producer {
        Some(p) => p,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Message broker not configured".to_string(),
                code: "SERVICE_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Extract course_id from command if present
    let course_id = match &command {
        AgenticCommand::SendMessage { course_id, .. } => {
            course_id.clone()
                .and_then(|s| CourseId::parse(&s).ok())
                .unwrap_or_else(CourseId::new)
        }
        AgenticCommand::ContinueCourse { course_id, .. } => {
            CourseId::parse(course_id).unwrap_or_else(|_| CourseId::new())
        }
        AgenticCommand::CancelTask { .. } => CourseId::new(),
        AgenticCommand::GetCourseHistory { course_id, .. } => {
            CourseId::parse(course_id).unwrap_or_else(|_| CourseId::new())
        }
    };

    // Generate shuttle_id for this unit of work
    let shuttle_id = ShuttleId::new();
    
    // Extract tenant_id and content from command
    let (tenant_id_str, content, blocks) = match &command {
        AgenticCommand::SendMessage {
            content,
            tenant_id,
            blocks,
            ..
        } => (tenant_id.clone(), content.clone(), blocks.clone()),
        AgenticCommand::ContinueCourse { .. } => {
            (String::new(), String::new(), None)
        }
        _ => (String::new(), String::new(), None),
    };

    // Parse tenant_id or use placeholder
    let tenant_id = TenantId::parse(&tenant_id_str).unwrap_or_else(|_| TenantId::new());
    let user_id = UserId::new(); // Will be populated from auth in production

    // Convert domain blocks to wire format (Vec<serde_json::Value>)
    let wire_blocks = blocks.map(|b| {
        b.into_iter()
            .filter_map(|block| serde_json::to_value(block).ok())
            .collect()
    });

    // Build envelope with CourseStart payload
    let envelope = EnvelopeV1::command(
        tenant_id,
        user_id,
        course_id.to_string(),
        Payload::CourseStart {
            weave_id: Uuid::new_v4(),
            content,
            context: None,
            blocks: wire_blocks,
            conversation_history: vec![],
        },
    ).with_course_id(course_id)
     .with_shuttle_id(shuttle_id);

    info!(course_id = %course_id, shuttle_id = %shuttle_id, "Sending agentic command via Kafka");

    match producer.send_command(&envelope).await {
        Ok(()) => {
            info!(course_id = %course_id, "Agentic command sent");

            Json(CommandAcceptedResponse {
                accepted: true,
                job_id: shuttle_id.to_string(),
                course_id: course_id.to_string(),
                shuttle_id: shuttle_id.to_string(),
                ws_url: format!("/api/courses/{}/ws", course_id),
                message: "Command accepted. Connect to WebSocket for updates.".to_string(),
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to send agentic command");
            
            (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: format!("Failed to send command: {}", e),
                code: "SEND_FAILED".to_string(),
            })).into_response()
        }
    }
}

/// GET /api/agentic/courses/:id - Get course by ID
///
/// Returns course details and all messages.
/// Note: Course storage is currently in-memory/stateless.
/// Future: Will query database for persistent courses.
pub async fn get_course_handler(
    State(_state): State<Arc<AppState>>,
    Path(course_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement course storage/retrieval
    // For now, return a placeholder indicating the course doesn't exist
    // In production, this would query the database

    info!(course_id = %course_id, "Getting course");

    // Return a detailed error as JSON
    Json(serde_json::json!({
        "error": "Course storage not yet implemented",
        "code": "NOT_IMPLEMENTED",
        "details": {
            "course_id": course_id,
            "note": "Course persistence will be added when database schema is extended"
        }
    }))
    .into_response()
}

/// GET /api/agentic/courses - List courses for tenant
///
/// Returns a list of course summaries.
/// Note: Currently a placeholder.
pub async fn list_courses_handler(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // TODO: Implement course listing
    // Would query database for courses belonging to authenticated tenant

    info!("Listing courses");

    Json::<Vec<CourseSummaryResponse>>(vec![]).into_response()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = AgenticCommand::SendMessage {
            course_id: None,
            content: "Hello!".to_string(),
            blocks: None,
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            conversation_history: None,
            flow_path: None,
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("send_message"));
    }
}
