use serde::{Deserialize, Serialize};
use crate::components::{Identity, FieldExcitation, ContentPayload, TaskDynamics};
use crate::primitives::Timestamp;

/// A specific, bounded future action with potential energy.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Intent {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub content: ContentPayload,

    /// The thermodynamic state of the task (Entropy, Activation).
    #[serde(flatten)]
    pub dynamics: TaskDynamics,

    pub target_date: Option<Timestamp>,
}

