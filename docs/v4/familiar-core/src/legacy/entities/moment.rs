use serde::{Deserialize, Serialize};
use crate::components::{Identity, FieldExcitation, QuantumState, ContentPayload};
use crate::types::MomentType;

/// A classical entity representing a specific, objective event in the past.
/// This is the atomic unit of episodic memory (The Particle).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Moment {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub quantum: QuantumState,

    #[serde(flatten)]
    pub content: ContentPayload,

    pub moment_type: MomentType,
}

