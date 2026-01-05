//! Identity component for entities

use serde::{Deserialize, Serialize};

use crate::primitives::{UUID, Timestamp};

/// Core identity component that all entities possess
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Identity {
    /// Unique identifier for this entity
    pub id: UUID,
    /// Tenant (family/user) this entity belongs to
    pub tenant_id: UUID,
    /// When this entity was created
    pub created_at: Timestamp,
}

impl Identity {
    pub fn new(tenant_id: UUID) -> Self {
        Self {
            id: UUID::new(),
            tenant_id,
            created_at: Timestamp::now(),
        }
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new(UUID::new())
    }
}

