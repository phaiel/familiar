//! Weave endpoint - Process user input through Kafka
//!
//! This is the main classification endpoint. It:
//! 1. Receives raw user input (text or multimodal blocks)
//! 2. Uploads binary media to MinIO (if configured)
//! 3. Emits CourseStartCommand to Kafka via EnvelopeProducer
//! 4. Returns job reference for WebSocket progress streaming

use std::sync::Arc;
use axum::{extract::State, Extension, Json, response::IntoResponse};
use tracing::info;
use utoipa::ToSchema;

// Wire contracts (from familiar-core)
use familiar_core::types::kafka::{EnvelopeV1, Payload, TenantId, UserId, CourseId};

// Auth middleware types
use crate::middleware::AuthenticatedUser;

// Domain types (from familiar-core)
use familiar_core::{
    primitives::UUID,
    entities::api::{
        WeaveRequest, CourseResponse,
        multimodal::{WeaveBlock, MediaRef, MediaType},
    },
    entities::Course,
};

use crate::state::AppState;

use base64::prelude::*;

// ============================================================================
// Media Processing
// ============================================================================

/// Process blocks: Upload binaries to MinIO and replace with presigned URLs
async fn process_multimodal_blocks(
    state: &AppState,
    blocks: Vec<WeaveBlock>,
) -> Result<(Vec<WeaveBlock>, Vec<MediaRef>), String> {
    let mut processed_blocks = Vec::new();
    let mut media_refs = Vec::new();
    
    // If no media store, just pass blocks through (Windmill might handle base64 or fail)
    let media_store = match &state.media_store {
        Some(s) => Some(s),
        None => None,
    };

    for block in blocks {
        match block {
            WeaveBlock::Image(mut img) => {
                // If base64 and media store available, upload
                let should_upload = img.source.starts_with("data:") && media_store.is_some();
                
                if should_upload {
                    let source_clone = img.source.clone();
                    let parts: Vec<&str> = source_clone.split(',').collect();
                    
                    if parts.len() == 2 {
                        let header = parts[0]; // data:image/png;base64
                        let data_b64 = parts[1];
                        let mime_type = header.split(';').next().unwrap_or("data:image/jpeg").split(':').nth(1).unwrap_or("image/jpeg").to_string();
                        
                        match BASE64_STANDARD.decode(data_b64) {
                            Ok(data) => {
                                let key = format!("{}.{}", uuid::Uuid::new_v4(), mime_type.split('/').nth(1).unwrap_or("bin"));
                                
                                if let Some(store) = media_store {
                                    // Upload
                                    if let Err(e) = store.upload(&key, data, &mime_type).await {
                                        tracing::warn!("Failed to upload image to MinIO: {}", e);
                                    } else {
                                        // Get presigned URL
                                        match store.get_presigned_url(&key, std::time::Duration::from_secs(3600)).await {
                                            Ok(url) => {
                                                img.source = url; // Replace source with URL
                                                
                                                // Create MediaRef
                                                media_refs.push(MediaRef {
                                                    id: uuid::Uuid::new_v4().to_string(),
                                                    media_type: MediaType::Image,
                                                    bucket_key: Some(key),
                                                    normalized_text: img.alt_text.clone().unwrap_or_default(),
                                                    metadata: serde_json::json!({ "mime_type": mime_type }),
                                                });
                                            },
                                            Err(e) => tracing::warn!("Failed to get presigned URL: {}", e),
                                        }
                                    }
                                }
                            },
                            Err(e) => tracing::warn!("Failed to decode base64 image: {}", e),
                        }
                    }
                }
                processed_blocks.push(WeaveBlock::Image(img));
            },
            WeaveBlock::Audio(mut audio) => {
                let should_upload = audio.source.starts_with("data:") && media_store.is_some();
                
                if should_upload {
                    let source_clone = audio.source.clone();
                    let parts: Vec<&str> = source_clone.split(',').collect();
                    
                    if parts.len() == 2 {
                        let header = parts[0];
                        let data_b64 = parts[1];
                        let mime_type = header.split(';').next().unwrap_or("data:audio/mp3").split(':').nth(1).unwrap_or("audio/mp3").to_string();
                        
                        match BASE64_STANDARD.decode(data_b64) {
                            Ok(data) => {
                                let ext = mime_type.split('/').nth(1).unwrap_or("bin");
                                let key = format!("{}.{}", uuid::Uuid::new_v4(), ext);
                                
                                if let Some(store) = media_store {
                                    if let Ok(_) = store.upload(&key, data, &mime_type).await {
                                        if let Ok(url) = store.get_presigned_url(&key, std::time::Duration::from_secs(3600)).await {
                                            audio.source = url;
                                            
                                            media_refs.push(MediaRef {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                media_type: MediaType::Audio,
                                                bucket_key: Some(key),
                                                normalized_text: audio.transcript.clone().unwrap_or_default(),
                                                metadata: serde_json::json!({ "duration": audio.duration_secs }),
                                            });
                                        }
                                    }
                                }
                            },
                            Err(_) => {},
                        }
                    }
                }
                processed_blocks.push(WeaveBlock::Audio(audio));
            },
            // Other blocks pass through
            b => processed_blocks.push(b),
        }
    }
    
    Ok((processed_blocks, media_refs))
}

// ============================================================================
// Kafka Command Emission (NEW - preferred)
// ============================================================================

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Emit a CourseStart command to Kafka using EnvelopeV1 pattern
/// Returns the course_id as the job reference
async fn emit_course_start_via_kafka(
    state: &AppState,
    course_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    user_id: uuid::Uuid,
    weave_id: uuid::Uuid,
    content: String,
    context: Option<String>,
    blocks: Option<Vec<WeaveBlock>>,
) -> Result<String, String> {
    let producer = state.envelope_producer.as_ref()
        .ok_or_else(|| "Kafka producer not configured".to_string())?;
    
    // Convert domain blocks to wire format (Vec<serde_json::Value>)
    let wire_blocks = blocks.map(|b| {
        b.into_iter()
            .filter_map(|block| serde_json::to_value(block).ok())
            .collect()
    });
    
    let envelope = EnvelopeV1::command(
        TenantId::from(tenant_id),
        UserId::from(user_id),
        course_id.to_string(), // correlation_id = course_id for course workflows
        Payload::CourseStart {
            weave_id,
            content,
            context,
            blocks: wire_blocks,
            conversation_history: vec![],
        },
    )
    .with_course_id(CourseId::from(course_id));
    
    producer.send_command(&envelope).await.map_err(|e| format!("Failed to emit command: {}", e))?;
    
    info!(
        course_id = %course_id,
        "CourseStart emitted via EnvelopeProducer"
    );
    
    Ok(course_id.to_string())
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct JobAcceptedResponse {
    pub success: bool,
    pub job_id: String,
    pub ws_url: String,
    pub message: String,
    /// Processing method used (kafka)
    #[serde(default)]
    pub method: Option<String>,
}

// ============================================================================
// Handler
// ============================================================================

/// POST /api/weave - Process a user message
/// 
/// Emits CourseStartCommand to Kafka for async processing.
/// Returns job reference for WebSocket progress streaming.
/// 
/// Requires authentication - user_id and tenant_id are extracted from JWT.
pub async fn weave_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(req): Json<WeaveRequest>,
) -> impl IntoResponse {
    let provider_str = req.provider.clone().unwrap_or_else(|| "kafka".to_string());
    
    // User and tenant come from authenticated JWT - no more placeholders!
    let tenant_id = UUID::from_uuid(auth.tenant_id);
    let user_id = auth.user_id;
    let weave_id = uuid::Uuid::new_v4();

    // Create Course from Weave (use weave string or first text block as fallback)
    let weave_text = req.weave.clone().or_else(|| {
        req.blocks.as_ref().and_then(|bs| bs.iter().find_map(|b| match b {
            WeaveBlock::Text(t) => Some(t.content.clone()),
            _ => None,
        }))
    }).unwrap_or_else(|| "Multimodal input".to_string());

    // Create the Course entity (session container)
    // Note: Provider/model tracking moved to Shuttle (transient unit of work)
    let mut course = Course::new(tenant_id)
        .with_title(weave_text.chars().take(100).collect::<String>());
    
    // Commit the initial user message to the course history
    course.commit_user_message(&weave_text);
    
    // Generate a shuttle ID for this request (transient unit of work)
    let shuttle_id = uuid::Uuid::new_v4().to_string();

    // Process multimodal blocks (upload to MinIO)
    let (processed_blocks, _media_refs) = if let Some(blocks) = req.blocks.clone() {
        match process_multimodal_blocks(&state, blocks).await {
            Ok((processed, refs)) => (Some(processed), Some(refs)),
            Err(e) => {
                tracing::error!("Failed to process media: {}", e);
                return Json(CourseResponse::error(
                    &course.id.to_string(),
                    &shuttle_id,
                    &weave_text,
                    &provider_str,
                    format!("Media processing failed: {}", e),
                )).into_response();
            }
        }
    } else {
        (None, None)
    };

    // Emit to Kafka via EnvelopeProducer
    if state.has_envelope_producer() {
        info!(
            course_id = %course.id,
            "Processing weave via Kafka (async)"
        );
        
        match emit_course_start_via_kafka(
            &state,
            course.id.as_uuid(),
            tenant_id.as_uuid(),
            user_id,
            weave_id,
            weave_text.clone(),
            req.context.clone(),
            processed_blocks.clone(),
        ).await {
            Ok(job_id) => {
                info!(
                    course_id = %course.id,
                    "CourseStartCommand emitted to Kafka"
                );
                
                return Json(JobAcceptedResponse {
                    success: true,
                    job_id,
                    ws_url: format!("/api/courses/{}/ws", course.id),
                    message: "Processing started via Kafka. Connect to WebSocket for updates.".to_string(),
                    method: Some("kafka".to_string()),
                }).into_response();
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Kafka emit failed"
                );
                return Json(CourseResponse::error(
                    &course.id.to_string(),
                    &shuttle_id,
                    &weave_text,
                    &provider_str,
                    format!("Failed to emit command: {}", e),
                )).into_response();
            }
        }
    }

    // No envelope producer configured
    Json(CourseResponse::error(
        &course.id.to_string(),
        &shuttle_id,
        &weave_text,
        &provider_str,
        "No message broker configured. Set KAFKA_BOOTSTRAP_SERVERS environment variable.".to_string(),
    )).into_response()
}