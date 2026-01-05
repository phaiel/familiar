use serde::{Deserialize, Serialize};

use crate::components::{Identity, FieldExcitation, ContentPayload};
use crate::types::InternalStateType;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Pulse {
    #[serde(flatten)]
    pub identity: Identity,

    #[serde(flatten)]
    pub physics: FieldExcitation,

    #[serde(flatten)]
    pub content: ContentPayload,

    pub state_type: InternalStateType,
}

