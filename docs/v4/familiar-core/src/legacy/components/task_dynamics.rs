use serde::{Deserialize, Serialize};

use crate::primitives::NormalizedFloat;

/// Models the state of a Task/Intent using Cybernetics and Thermodynamics principles.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TaskDynamics {
    /// **Completion (Inverse Discrepancy)**
    /// Cybernetics: 1.0 - Error Signal.
    /// How close the system is to the Goal State.
    pub completion: NormalizedFloat,

    /// **Entropy (Disorder)**
    /// Information Theory: The uncertainty or "noise" in the process.
    /// High Entropy: Exploring, Brainstorming, Chaotic.
    /// Low Entropy: Executing, Refined, Linear.
    pub entropy: NormalizedFloat,

    /// **Activation Energy (Invested Work)**
    /// Thermodynamics: The cumulative work performed on the system.
    /// Used to calculate "Sunk Cost" gravity.
    pub activation_energy: NormalizedFloat,
    
    pub primary_status_label: Option<String>,
}

impl TaskDynamics {
    pub fn new(completion: f64, entropy: f64, energy: f64) -> Result<Self, String> {
        Ok(Self {
            completion: NormalizedFloat::new(completion)?,
            entropy: NormalizedFloat::new(entropy)?,
            activation_energy: NormalizedFloat::new(energy)?,
            primary_status_label: None,
        })
    }

    pub fn from_legacy_status(status: &str) -> Self {
        match status {
            "Pending" => Self::new(0.0, 0.0, 0.0).unwrap().with_label("Pending"), // Cold, zero error reduction
            "InProgress" => Self::new(0.3, 0.8, 0.4).unwrap().with_label("InProgress"), // High entropy (doing), some energy
            "Completed" => Self::new(1.0, 0.0, 1.0).unwrap().with_label("Completed"), // Zero error, zero entropy (static), full energy
            "Cancelled" => Self::new(0.0, 0.0, 0.1).unwrap().with_label("Cancelled"), // Zero error (abandoned), low energy
            _ => Self::default(),
        }
    }
    
    pub fn with_label(mut self, label: &str) -> Self {
        self.primary_status_label = Some(label.to_string());
        self
    }
}

impl Default for TaskDynamics {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0).unwrap()
    }
}
