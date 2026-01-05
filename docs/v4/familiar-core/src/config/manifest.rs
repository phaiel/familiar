use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemDomain {
    Ingestion,
    Physics,
    Analysis,
    Maintenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemTrigger {
    Event(String),
    Schedule(String), // cron expression
    OnDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SystemManifest {
    pub id: String,
    pub domain: SystemDomain,
    pub description: String,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub trigger: SystemTrigger,
}

