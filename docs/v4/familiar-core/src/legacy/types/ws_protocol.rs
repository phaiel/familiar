//! WebSocket Protocol Schema
//!
//! Defines the messages exchanged over WebSocket for job monitoring and updates.
//! Uses `ts-rs` to generate TypeScript interfaces for the frontend.
//!
//! Schema-first, ECS-inspired: Messages are entities, blocks are components.

use serde::{Deserialize, Serialize};
use crate::components::ui::BlockMessage;

/// A message sent from the server to the client via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Connection status / Handshake
    Status {
        status: String,
        job_id: String,
    },
    /// Typing indicator (agent is processing)
    Typing {
        job_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Job progress update (with optional Block Kit content)
    Progress {
        status: String, // e.g., "Running", "Queued"
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocks: Option<BlockMessage>,
    },
    /// Job completed successfully (with Block Kit content)
    Complete {
        job_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocks: Option<BlockMessage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>, // Full result for debugging
    },
    /// Error occurred (with Block Kit content)
    Error {
        job_id: Option<String>,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocks: Option<BlockMessage>,
    },
    /// Message status update (sent, delivered, read)
    MessageStatus {
        message_id: String,
        status: MessageStatusType,
    },
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatusType {
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
}

