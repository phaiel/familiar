use serde::{Deserialize, Serialize};

use crate::components::{Identity, FieldExcitation, QuantumState, ContentPayload};
use crate::types::ThreadType;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Thread {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub quantum: QuantumState,
    
    #[serde(flatten)]
    pub content: ContentPayload, // For description/metadata

    pub thread_type: ThreadType,
}

