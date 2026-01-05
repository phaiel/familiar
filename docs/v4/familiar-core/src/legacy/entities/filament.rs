use serde::{Deserialize, Serialize};
use crate::components::{Identity, FieldExcitation, QuantumState, ContentPayload};
use crate::primitives::UUID;

/// A recurring internal pattern (Self/Identity).
/// The continuous phase trajectory of the user through the manifold.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Filament {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub quantum: QuantumState,

    #[serde(flatten)]
    pub content: ContentPayload,

    /// The source pulses (internal states) that form this strand.
    pub source_pulses: Vec<UUID>,
}

