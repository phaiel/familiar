use serde::{Deserialize, Serialize};

use crate::primitives::{NormalizedFloat, SignedNormalizedFloat};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RelationalDynamics {
    // --- PHYSICS DIMENSIONS (Gravity/Springs) ---

    /// **Intimacy (Communal Strength / Rest Length):**
    /// 1.0 = Fusion (Rest Length 0). The entities want to occupy the same space.
    /// 0.0 = Estrangement (Max Rest Length).
    pub intimacy: NormalizedFloat,

    /// **Formalism (Transactionalism / Spring Stiffness):**
    /// 1.0 = Rigid/Brittle (High k). Hard to move, snaps if stretched.
    /// 0.0 = Fluid/Elastic (Low k). Easy to stretch, adapts to change.
    pub formalism: NormalizedFloat,

    /// **Power Dynamic (Authority Gradient / Mass Bias):**
    /// -1.0 = Target is the Anchor (Infinite Mass relative to Source).
    /// 0.0 = Equal Mass (Both move equally).
    /// +1.0 = Source is the Anchor.
    pub power_dynamic: SignedNormalizedFloat,

    // --- OPTICAL DIMENSIONS (Ray Tracing) ---

    /// **Transparency (Refractive Index):**
    /// 1.0 = Clear. Looking at the Source reveals the Target (e.g., "Honest").
    /// 0.0 = Opaque. The relationship hides the Target (e.g., "Secretive").
    pub transparency: NormalizedFloat,

    /// **Color Filter (Spectral Gating):**
    /// If set, this bond only "lights up" under specific emotional light.
    /// e.g., A "Work Friend" bond might only reflect "Professional" (Blue) light.
    /// Stored as a hex code or vector for the shader.
    pub spectral_signature: Option<[f32; 3]>,
    
    pub primary_label: Option<String>, 
}

impl RelationalDynamics {
    pub fn new(intimacy: f64, formalism: f64, power: f64, transparency: f64) -> Result<Self, String> {
        Ok(Self {
            intimacy: NormalizedFloat::new(intimacy)?,
            formalism: NormalizedFloat::new(formalism)?,
            power_dynamic: SignedNormalizedFloat::new(power)?,
            transparency: NormalizedFloat::new(transparency)?,
            spectral_signature: None,
            primary_label: None,
        })
    }

    /// Helper to calculate the physical Spring Constant (k) for the simulation loop
    pub fn spring_constant(&self) -> f64 {
        // Base stiffness + Formalism multiplier
        1.0 + (self.formalism.value() * 10.0)
    }

    /// Helper to calculate the Target Rest Length for the simulation loop
    pub fn rest_length(&self) -> f64 {
        // High intimacy = Low length
        100.0 * (1.0 - self.intimacy.value())
    }

    pub fn from_legacy_type(legacy_type: &str) -> Self {
        match legacy_type {
            "Romantic" => Self::new(0.9, 0.3, 0.0, 0.9).unwrap().with_label("Romantic"),
            "Family" => Self::new(0.8, 0.7, 0.5, 0.8).unwrap().with_label("Family"),
            "Friend" => Self::new(0.6, 0.2, 0.0, 0.9).unwrap().with_label("Friend"),
            "Professional" => Self::new(0.1, 0.9, 0.3, 0.5).unwrap().with_label("Professional"), // Semi-opaque
            "Adversarial" => Self::new(0.1, 0.1, 0.0, 0.0).unwrap().with_label("Adversarial"), // Opaque
            _ => Self::default(),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.primary_label = Some(label.to_string());
        self
    }
}

impl Default for RelationalDynamics {
    fn default() -> Self {
        Self::new(0.0, 0.5, 0.0, 0.5).unwrap()
    }
}
