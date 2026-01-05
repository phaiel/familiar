use serde::{Deserialize, Serialize};

use crate::components::{Identity, FieldExcitation, BondPhysics, ContentPayload, RelationalDynamics};
use crate::primitives::UUID;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Bond {
    #[serde(flatten)]
    pub identity: Identity,

    /// The midpoint physics state of the bond
    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub bond_physics: BondPhysics,

    #[serde(flatten)]
    pub content: ContentPayload,

    // Replaced Enum with Vector Component
    #[serde(flatten)]
    pub dynamics: RelationalDynamics,
    
    // Connections
    pub head_thread_id: UUID,
    pub tail_thread_id: UUID,
}
