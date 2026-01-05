use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContentPayload {
    pub text: String,
    pub metadata: HashMap<String, String>,
}

