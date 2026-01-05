//! Familiar Entity Types
//!
//! Types for entities spawned from conversation analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::{TenantId, UserId, MessageId, ChannelId};

/// Type of familiar entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FamiliarEntityType {
    Moment,
    Pulse,
    Intent,
    Thread,
    Bond,
    Motif,
    Filament,
    Focus,
}

impl FamiliarEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Moment => "MOMENT",
            Self::Pulse => "PULSE",
            Self::Intent => "INTENT",
            Self::Thread => "THREAD",
            Self::Bond => "BOND",
            Self::Motif => "MOTIF",
            Self::Filament => "FILAMENT",
            Self::Focus => "FOCUS",
        }
    }
}

/// HILT (Human in the Loop) status for entity approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    Pending,
    Approved,
    Rejected,
    AutoSpawned,
}

impl Default for EntityStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl EntityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::AutoSpawned => "auto_spawned",
        }
    }
}

/// Physics state for an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityPhysics {
    pub valence: f64,
    pub arousal: f64,
    pub significance: f64,
    pub epistemic: f64,
}

/// A familiar entity spawned from conversation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FamiliarEntity {
    pub id: Uuid,  // EntityId (could be created)
    pub tenant_id: TenantId,
    pub entity_type: FamiliarEntityType,
    pub content: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub physics: Option<EntityPhysics>,
    /// Reference to Qdrant vector
    #[serde(default)]
    pub qdrant_point_id: Option<Uuid>,
    #[serde(default)]
    pub qdrant_collection: Option<String>,
    #[serde(default)]
    pub source_message_id: Option<MessageId>,
    #[serde(default)]
    pub source_channel_id: Option<ChannelId>,
    pub status: EntityStatus,
    #[serde(default)]
    pub reviewed_by: Option<UserId>,
    #[serde(default)]
    pub reviewed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new familiar entity
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateEntityInput {
    pub tenant_id: TenantId,
    pub entity_type: FamiliarEntityType,
    pub content: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub physics: Option<EntityPhysics>,
    #[serde(default)]
    pub source_message_id: Option<MessageId>,
    #[serde(default)]
    pub source_channel_id: Option<ChannelId>,
    #[serde(default)]
    pub status: Option<EntityStatus>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Input for updating entity status (HILT)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateEntityStatusInput {
    pub status: EntityStatus,
    #[serde(default)]
    pub reviewed_by: Option<UserId>,
}

/// Options for listing entities
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListEntitiesOptions {
    #[serde(default)]
    pub entity_type: Option<FamiliarEntityType>,
    #[serde(default)]
    pub status: Option<EntityStatus>,
    #[serde(default)]
    pub limit: Option<i64>,
}

