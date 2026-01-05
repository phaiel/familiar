//! Channel and Message API Routes
//!
//! Routes for multi-tenant conversation persistence.
//!
//! Routes:
//! - GET /api/tenants/:id/channels - List channels for a tenant
//! - POST /api/channels - Create a new channel
//! - GET /api/channels/:id - Get channel by ID
//! - GET /api/channels/:id/messages - Get messages in a channel
//! - POST /api/channels/:id/messages - Send a message (no AI)

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
    Channel, CreateChannelInput, ChannelType, ListChannelsOptions,
    Message, CreateMessageInput, ListMessagesOptions,
    ConversationMessage,
};
use familiar_core::primitives::{UserId, ChannelId};

use crate::state::AppState;
use super::ErrorResponse;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListChannelsQuery {
    #[serde(default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    #[serde(default)]
    pub limit: Option<i64>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct ChannelListResponse {
    pub channels: Vec<Channel>,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageListResponse {
    pub messages: Vec<Message>,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationHistoryResponse {
    pub history: Vec<ConversationMessage>,
    pub count: usize,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/tenants/:tenant_id/channels - List channels for a tenant
pub async fn list_channels_handler(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<ListChannelsQuery>,
) -> impl IntoResponse {
    info!(tenant_id = %tenant_id, "Listing channels");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    let options = ListChannelsOptions {
        channel_type: query.channel_type.and_then(|ct| match ct.as_str() {
            "family" => Some(ChannelType::Family),
            "shared" => Some(ChannelType::Shared),
            "personal" => Some(ChannelType::Personal),
            _ => None,
        }),
        owner_id: query.owner_id.map(|id| id.into()),
    };

    match store.get_channels(tenant_id, options).await {
        Ok(channels) => {
            let count = channels.len();
            Json(ChannelListResponse { channels, count }).into_response()
        }
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// POST /api/channels - Create a new channel
pub async fn create_channel_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateChannelInput>,
) -> impl IntoResponse {
    info!(tenant_id = %input.tenant_id, name = %input.name, "Creating channel");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    match store.create_channel(input).await {
        Ok(channel) => Json(channel).into_response(),
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// GET /api/channels/:id - Get channel by ID
pub async fn get_channel_handler(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
) -> impl IntoResponse {
    info!(channel_id = %channel_id, "Getting channel");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    match store.get_channel(channel_id).await {
        Ok(Some(channel)) => Json(channel).into_response(),
        Ok(None) => {
            Json(ErrorResponse {
                error: "Channel not found".to_string(),
                code: "NOT_FOUND".to_string(),
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

/// GET /api/channels/:id/messages - Get messages in a channel
pub async fn list_messages_handler(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    info!(channel_id = %channel_id, "Listing messages");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    let options = ListMessagesOptions {
        limit: query.limit,
        before: None,
        after: None,
    };

    match store.get_messages(ChannelId::from(channel_id), options).await {
        Ok(messages) => {
            let count = messages.len();
            Json(MessageListResponse { messages, count }).into_response()
        }
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// GET /api/channels/:id/history - Get conversation history (for LLM context)
pub async fn get_history_handler(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    info!(channel_id = %channel_id, "Getting conversation history");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    let limit = query.limit.unwrap_or(20);

    match store.get_conversation_history(ChannelId::from(channel_id), limit).await {
        Ok(history) => {
            let count = history.len();
            Json(ConversationHistoryResponse { history, count }).into_response()
        }
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}

/// POST /api/channels/:id/messages - Send a message (no AI call)
pub async fn send_message_handler(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
    Json(mut input): Json<CreateMessageInput>,
) -> impl IntoResponse {
    info!(channel_id = %channel_id, role = ?input.role, "Sending message");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            }).into_response();
        }
    };

    // Override channel_id from path
    input.channel_id = channel_id.into();

    match store.create_message(input).await {
        Ok(message) => Json(message).into_response(),
        Err(e) => {
            Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            }).into_response()
        }
    }
}