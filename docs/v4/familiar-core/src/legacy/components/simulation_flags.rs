use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SimulationTier {
    Conscious,   // Update every tick (High Priority)
    Subconscious, // Update every 100 ticks (Medium Priority)
    DeepStorage,  // Update only during "Dream" cycles (Low Priority)
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SimLOD {
    pub tier: SimulationTier,
    pub last_update: u64, // Tick count
}

impl Default for SimLOD {
    fn default() -> Self {
        Self {
            tier: SimulationTier::Conscious, // Default to hot simulation
            last_update: 0,
        }
    }
}

