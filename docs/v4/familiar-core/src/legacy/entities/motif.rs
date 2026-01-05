use serde::{Deserialize, Serialize};
use crate::components::{Identity, FieldExcitation, QuantumState, ContentPayload};
use crate::primitives::UUID;

/// A recurring pattern of subjective experiences (External Wave).
/// Formed by the constructive interference of many Moments.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Motif {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    /// The Interference Pattern (Hologram) of the motif.
    #[serde(flatten)]
    pub quantum: QuantumState,

    #[serde(flatten)]
    pub content: ContentPayload,

    /// The source moments that collapsed into this pattern.
    pub source_moments: Vec<UUID>,
}

