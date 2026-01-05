//! Channel Types
//!
//! Types for conversation channel management.

use serde::{Deserialize, Serialize};

use crate::primitives::{ChannelId, TenantId, UserId};
use crate::types::base::EntityMeta;

/// Type of channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Personal channel for a single user
    Personal,
    /// Family channel shared by all tenant members
    Family,
    /// Shared channel for specific members
    Shared,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Personal
    }
}

impl ChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Family => "family",
            Self::Shared => "shared",
        }
    }
}

/// A channel (conversation space)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Channel {
    /// Entity metadata (id, tenant_id, timestamps)
    #[serde(flatten)]
    pub meta: EntityMeta<ChannelId>,
    /// Owner of the channel (None for family channels)
    #[serde(default)]
    pub owner_id: Option<UserId>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub channel_type: ChannelType,
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// Input for creating a new channel
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateChannelInput {
    pub tenant_id: TenantId,
    #[serde(default)]
    pub owner_id: Option<UserId>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub channel_type: Option<ChannelType>,
}

/// Options for listing channels
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListChannelsOptions {
    #[serde(default)]
    pub channel_type: Option<ChannelType>,
    #[serde(default)]
    pub owner_id: Option<UserId>,
}

