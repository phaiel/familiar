//! WebSocket Handler for Course Monitoring
//!
//! Provides real-time updates using Kafka trace streaming.
//! Connects to familiar.trace topic and filters by course_id.
//!
//! ## Stream Interleaving
//!
//! The handler interleaves backfill events with live Kafka events to ensure
//! the socket remains responsive even during slow database queries:
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐
//! │ DB Backfill │ ──► │              │
//! └─────────────┘     │  Merged      │ ──► WebSocket
//! ┌─────────────┐     │  Stream      │
//! │ Live Kafka  │ ──► │              │
//! └─────────────┘     └──────────────┘
//! ```

use axum::{
    extract::{State, Path, WebSocketUpgrade, Query, ws::{WebSocket, Message}},
    response::IntoResponse,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::state::AppState;
use familiar_core::{
    types::WsMessage,
    components::ui::{BlockMessageExt, BlockMessage},
    entities::db::trace::course_trace::{Entity as CourseTrace, Column as CourseTraceColumn},
};
use familiar_core::types::kafka::{EnvelopeV1, Payload, TraceKind, TraceStatus, TenantId, UserId};
use sea_orm::{EntityTrait, QueryOrder, ColumnTrait, QueryFilter, QuerySelect, DatabaseConnection};
use tracing::{info, warn, debug};

// ============================================================================
// WebSocket Event Types
// ============================================================================

/// Event sent to UI for agentic updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AgenticWsEvent {
    #[serde(rename = "agent_speaking")]
    AgentSpeaking {
        agent: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    #[serde(rename = "tool_call")]
    ToolCall {
        agent: String,
        tool: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<serde_json::Value>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        agent: String,
        thought: String,
    },
    #[serde(rename = "message_received")]
    MessageReceived {
        agent: String,
        content: String,
        course_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        shuttle_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_type: Option<serde_json::Value>,
    },
    #[serde(rename = "task_completed")]
    TaskCompleted {
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "error")]
    Error {
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQueryParams {
    /// Sequence number to start backfill from (for reconnects)
    #[serde(default)]
    pub since_seq: Option<u64>,
}

// ============================================================================
// WebSocket Handler (Kafka-based)
// ============================================================================

/// WebSocket handler for course monitoring via Kafka
/// Route: /api/courses/:course_id/ws?since_seq=N
pub async fn course_ws_handler(
    ws: WebSocketUpgrade,
    Path(course_id): Path<String>,
    Query(params): Query<WsQueryParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_course_socket(socket, course_id, params.since_seq, state))
}

/// Internal event type for the merged stream
enum MergedEvent {
    /// Event from database backfill
    Backfill(AgenticWsEvent),
    /// Event from live Kafka stream
    Live(EnvelopeV1),
    /// Backfill completed marker
    BackfillComplete,
    /// Error during backfill
    BackfillError(String),
}

/// Handle course WebSocket with interleaved backfill and live streaming
/// 
/// Unlike sequential backfill-then-stream, this spawns backfill as a separate
/// task and merges events via a channel, ensuring the socket remains responsive.
async fn handle_course_socket(
    mut socket: WebSocket,
    course_id: String,
    since_seq: Option<u64>,
    state: Arc<AppState>,
) {
    info!(course_id = %course_id, since_seq = ?since_seq, "Course WebSocket connected");
    
    let course_uuid = match uuid::Uuid::parse_str(&course_id) {
        Ok(u) => u,
        Err(e) => {
            let msg = WsMessage::Error {
                job_id: Some(course_id),
                message: format!("Invalid course_id: {}", e),
                blocks: None,
            };
            let _ = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
            return;
        }
    };
    
    // Create merged event channel
    let (tx, mut rx) = mpsc::channel::<MergedEvent>(256);
    
    // Spawn backfill task (non-blocking)
    if let Some(seq) = since_seq {
        if let Some(store) = &state.store {
            let backfill_tx = tx.clone();
            let db = store.db().clone();
            let backfill_course_id = course_id.clone();
            
            tokio::spawn(async move {
                debug!(course_id = %backfill_course_id, since_seq = seq, "Starting backfill task");
                
                match run_backfill(&db, course_uuid, seq).await {
                    Ok(traces) => {
                        info!(course_id = %backfill_course_id, trace_count = traces.len(), "Backfill complete");
                        for trace in traces {
                            let ws_event = db_trace_to_ws_event(&trace);
                            if backfill_tx.send(MergedEvent::Backfill(ws_event)).await.is_err() {
                                break; // Channel closed, socket disconnected
                            }
                        }
                        let _ = backfill_tx.send(MergedEvent::BackfillComplete).await;
                    }
                    Err(e) => {
                        warn!(error = %e, "Backfill failed");
                        let _ = backfill_tx.send(MergedEvent::BackfillError(e.to_string())).await;
                    }
                }
            });
        }
    }
    
    // Spawn live Kafka streaming task
    let kafka_handle = if let Some(consumer) = &state.kafka_consumer {
        let live_tx = tx.clone();
        let kafka_rx = consumer.subscribe_course(course_uuid).await;
        let live_course_id = course_id.clone();
        
        Some(tokio::spawn(async move {
            let mut kafka_rx = kafka_rx;
            loop {
                match kafka_rx.recv().await {
                    Ok(envelope) => {
                        let is_terminal = is_terminal_envelope(&envelope);
                        if live_tx.send(MergedEvent::Live(envelope)).await.is_err() {
                            break; // Channel closed
                        }
                        if is_terminal {
                            break;
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, course_id = %live_course_id, "Kafka recv error, channel may have lagged");
                        // Don't break - the channel might recover
                    }
                }
            }
        }))
    } else {
        None
    };
    
    // Drop our sender so channel closes when tasks complete
    drop(tx);
    
    // If no Kafka consumer and no backfill, send error and exit
    if kafka_handle.is_none() && since_seq.is_none() {
        let msg = WsMessage::Error {
            job_id: Some(course_id.clone()),
            message: "Message broker not configured".to_string(),
            blocks: Some(BlockMessage::error("Kafka consumer not available")),
        };
        let _ = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
        return;
    }
    
    // Send initial status
    let msg = WsMessage::Status {
        job_id: course_id.clone(),
        status: "streaming".to_string(),
    };
    let _ = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
    
    // Main event loop - interleave backfill and live events
    let mut backfill_complete = since_seq.is_none(); // No backfill needed if no since_seq
    
    loop {
        tokio::select! {
            // Receive merged events
            event = rx.recv() => {
                match event {
                    Some(MergedEvent::Backfill(ws_event)) => {
                        if let Err(e) = socket.send(Message::Text(serde_json::to_string(&ws_event).unwrap())).await {
                            warn!(error = %e, "Failed to send backfill event");
                            break;
                        }
                    }
                    Some(MergedEvent::Live(envelope)) => {
                        let ws_event = trace_to_ws_event(&envelope);
                        if let Err(e) = socket.send(Message::Text(serde_json::to_string(&ws_event).unwrap())).await {
                            warn!(error = %e, "Failed to send live event");
                            break;
                        }
                        if is_terminal_envelope(&envelope) {
                            break;
                        }
                    }
                    Some(MergedEvent::BackfillComplete) => {
                        backfill_complete = true;
                        debug!(course_id = %course_id, "Backfill stream complete");
                    }
                    Some(MergedEvent::BackfillError(e)) => {
                        backfill_complete = true;
                        warn!(error = %e, "Backfill error (continuing with live stream)");
                    }
                    None => {
                        // All senders dropped - no more events
                        if backfill_complete {
                            info!(course_id = %course_id, "Event stream ended");
                        }
                        break;
                    }
                }
            }
            
            // Check for client messages (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!(course_id = %course_id, "Client disconnected");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Cleanup
    if let Some(handle) = kafka_handle {
        handle.abort();
    }
    if let Some(consumer) = &state.kafka_consumer {
        consumer.unsubscribe_course(course_uuid);
    }
}

/// Run database backfill query (isolated for spawn)
async fn run_backfill(
    db: &DatabaseConnection,
    course_uuid: uuid::Uuid,
    since_seq: u64,
) -> Result<Vec<familiar_core::entities::db::trace::course_trace::Model>, sea_orm::DbErr> {
    CourseTrace::find()
        .filter(CourseTraceColumn::CourseId.eq(course_uuid))
        .filter(CourseTraceColumn::Seq.gt(since_seq as i64))
        .order_by_asc(CourseTraceColumn::Seq)
        .limit(1000) // Cap backfill to prevent overwhelming client
        .all(db)
        .await
}

/// Convert an EnvelopeV1 trace to a WebSocket event
fn trace_to_ws_event(envelope: &EnvelopeV1) -> AgenticWsEvent {
    match &envelope.payload {
        Payload::Trace {
            kind,
            status,
            agent,
            message,
            tool_name,
            tool_args,
            tool_result,
            tokens,
            ..
        } => {
            let agent_name = agent.clone().unwrap_or_else(|| "system".to_string());
            
            match kind {
                TraceKind::Step => AgenticWsEvent::Thinking {
                    agent: agent_name,
                    thought: message.clone(),
                },
                TraceKind::Thought => AgenticWsEvent::Thinking {
                    agent: agent_name,
                    thought: message.clone(),
                },
                TraceKind::Tool => {
                    if *status == TraceStatus::Started {
                        AgenticWsEvent::ToolCall {
                            agent: agent_name,
                            tool: tool_name.clone().unwrap_or_default(),
                            arguments: tool_args.clone(),
                        }
                    } else {
                        AgenticWsEvent::ToolResult {
                            tool: tool_name.clone().unwrap_or_default(),
                            result: tool_result.clone().map(serde_json::Value::String),
                        }
                    }
                }
                TraceKind::Token => AgenticWsEvent::AgentSpeaking {
                    agent: agent.clone().unwrap_or_else(|| "assistant".to_string()),
                    message: tokens.clone(),
                },
                TraceKind::Error => AgenticWsEvent::Error {
                    error: message.clone(),
                    code: None,
                },
                TraceKind::Metric => AgenticWsEvent::Thinking {
                    agent: agent_name,
                    thought: message.clone(),
                },
            }
        }
        // Handle completion events
        Payload::CourseCompleted { response, .. } => AgenticWsEvent::TaskCompleted {
            result: Some(serde_json::Value::String(response.clone())),
        },
        Payload::CourseFailed { error, .. } => AgenticWsEvent::Error {
            error: error.clone(),
            code: None,
        },
        _ => AgenticWsEvent::Error {
            error: "Unexpected payload type".to_string(),
            code: Some("UNEXPECTED_PAYLOAD".to_string()),
        },
    }
}

/// Check if this envelope represents a terminal trace (result/completion)
fn is_terminal_envelope(envelope: &EnvelopeV1) -> bool {
    matches!(
        &envelope.payload,
        Payload::CourseCompleted { .. } | Payload::CourseFailed { .. } | Payload::CourseCancelled { .. }
    ) || matches!(
        &envelope.payload,
        Payload::Trace { status: TraceStatus::Completed | TraceStatus::Failed | TraceStatus::Cancelled, kind: TraceKind::Step, .. }
    )
}

/// Convert a database trace row to a WebSocket event (for backfill)
fn db_trace_to_ws_event(trace: &familiar_core::entities::db::trace::course_trace::Model) -> AgenticWsEvent {
    let kind = match trace.kind.as_str() {
        "step" => TraceKind::Step,
        "thought" => TraceKind::Thought,
        "tool" => TraceKind::Tool,
        "token" => TraceKind::Token,
        "error" => TraceKind::Error,
        "metric" => TraceKind::Metric,
        _ => TraceKind::Step,
    };
    
    let status = match trace.status.as_str() {
        "started" => TraceStatus::Started,
        "in_progress" => TraceStatus::InProgress,
        "completed" => TraceStatus::Completed,
        "failed" => TraceStatus::Failed,
        "cancelled" => TraceStatus::Cancelled,
        _ => TraceStatus::Started,
    };
    
    // Extract fields from payload JSON
    let payload = &trace.payload;
    let agent = payload.get("agent").and_then(|v| v.as_str()).unwrap_or("system").to_string();
    let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tool_name = payload.get("tool_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let tool_args = payload.get("tool_args").cloned();
    let tool_result = payload.get("tool_result").and_then(|v| v.as_str()).map(|s| s.to_string());
    let tokens = payload.get("tokens").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    match kind {
        TraceKind::Step | TraceKind::Thought => AgenticWsEvent::Thinking {
            agent,
            thought: message,
        },
        TraceKind::Tool => {
            if status == TraceStatus::Started {
                AgenticWsEvent::ToolCall {
                    agent,
                    tool: tool_name.unwrap_or_default(),
                    arguments: tool_args,
                }
            } else {
                AgenticWsEvent::ToolResult {
                    tool: tool_name.unwrap_or_default(),
                    result: tool_result.map(serde_json::Value::String),
                }
            }
        }
        TraceKind::Token => AgenticWsEvent::AgentSpeaking {
            agent,
            message: tokens,
        },
        TraceKind::Error => AgenticWsEvent::Error {
            error: message,
            code: None,
        },
        TraceKind::Metric => AgenticWsEvent::Thinking {
            agent,
            thought: message,
        },
    }
}
