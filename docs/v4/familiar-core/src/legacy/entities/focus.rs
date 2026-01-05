use serde::{Deserialize, Serialize};
use crate::components::{Identity, FieldExcitation, QuantumState, ContentPayload};
use crate::primitives::Timestamp;

/// A user-declared thematic goal or life chapter.
/// Acts as a "Attractor Basin" in the VAE manifold.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Focus {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub quantum: QuantumState,

    #[serde(flatten)]
    pub content: ContentPayload,

    pub active_since: Timestamp,
}

