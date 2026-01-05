//! Entity Metadata Base Type
//!
//! Common metadata for all persistent entities in the system.
//! Provides identity, tenant association, and lifecycle timestamps.

use serde::{Deserialize, Serialize};

use crate::components::Timestamps;
use crate::primitives::TenantId;

/// Common metadata for all persistent entities
/// 
/// This base type captures the essential fields every database entity needs:
/// - `id`: Unique identifier (type-specific, e.g., `UserId`, `ChannelId`)
/// - `tenant_id`: Which family/organization this entity belongs to
/// - `timestamps`: Creation and modification timestamps
///
/// ## Usage
///
/// Entities can flatten this into their struct:
/// ```rust,ignore
/// pub struct Channel {
///     #[serde(flatten)]
///     pub meta: EntityMeta<ChannelId>,
///     pub name: String,
///     // ... channel-specific fields
/// }
/// ```
///
/// Or reference individual fields if flattening doesn't work for your schema.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityMeta<Id> {
    /// Unique identifier for this entity
    pub id: Id,
    /// Tenant (family) this entity belongs to
    pub tenant_id: TenantId,
    /// Lifecycle timestamps (created_at, updated_at)
    #[serde(flatten)]
    pub timestamps: Timestamps,
}

impl<Id: Default> EntityMeta<Id> {
    /// Create new entity metadata with a default ID
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            id: Id::default(),
            tenant_id,
            timestamps: Timestamps::now(),
        }
    }
}

impl<Id> EntityMeta<Id> {
    /// Create entity metadata with a specific ID
    pub fn with_id(id: Id, tenant_id: TenantId) -> Self {
        Self {
            id,
            tenant_id,
            timestamps: Timestamps::now(),
        }
    }

    /// Create from database values
    pub fn from_db(
        id: Id, 
        tenant_id: TenantId, 
        created_at: chrono::DateTime<chrono::Utc>, 
        updated_at: chrono::DateTime<chrono::Utc>
    ) -> Self {
        Self {
            id,
            tenant_id,
            timestamps: Timestamps::from_db(created_at, updated_at),
        }
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.timestamps.touch();
    }
}

/// Metadata for entities without a tenant association (system-level entities)
/// 
/// Used for entities like:
/// - System configuration
/// - Migration tracking
/// - Global settings
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SystemEntityMeta<Id> {
    /// Unique identifier for this entity
    pub id: Id,
    /// Lifecycle timestamps (created_at, updated_at)
    #[serde(flatten)]
    pub timestamps: Timestamps,
}

impl<Id: Default> SystemEntityMeta<Id> {
    /// Create new system entity metadata with a default ID
    pub fn new() -> Self {
        Self {
            id: Id::default(),
            timestamps: Timestamps::now(),
        }
    }
}

impl<Id> SystemEntityMeta<Id> {
    /// Create system entity metadata with a specific ID
    pub fn with_id(id: Id) -> Self {
        Self {
            id,
            timestamps: Timestamps::now(),
        }
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.timestamps.touch();
    }
}

impl<Id: Default> Default for SystemEntityMeta<Id> {
    fn default() -> Self {
        Self::new()
    }
}




