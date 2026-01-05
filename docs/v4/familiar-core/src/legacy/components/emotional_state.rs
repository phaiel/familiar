use serde::{Deserialize, Serialize};

use crate::primitives::SignedNormalizedFloat;

/// Represents an emotional state using the PAD (Pleasure-Arousal-Dominance) Model.
/// Scientific Basis: Mehrabian & Russell (1974).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EmotionalState {
    /// **Pleasure (Valence)**
    /// -1.0: Agony/Despair (Negative)
    ///  0.0: Neutral
    /// +1.0: Ecstasy/Joy (Positive)
    pub valence: SignedNormalizedFloat,

    /// **Arousal (Activation)**
    /// -1.0: Sleep/Coma (Low Energy)
    ///  0.0: Alert
    /// +1.0: Panic/Frenzy (High Energy)
    pub arousal: SignedNormalizedFloat,

    /// **Dominance (Control)**
    /// -1.0: Submissive/Overwhelmed
    ///  0.0: Balanced
    /// +1.0: Dominant/In-Control
    pub dominance: SignedNormalizedFloat,
}

impl EmotionalState {
    pub fn new(v: f64, a: f64, d: f64) -> Result<Self, String> {
        Ok(Self {
            valence: SignedNormalizedFloat::new(v)?,
            arousal: SignedNormalizedFloat::new(a)?,
            dominance: SignedNormalizedFloat::new(d)?,
        })
    }
    
    // Standard Mappings
    pub fn joy() -> Self { Self::new(0.8, 0.6, 0.4).unwrap() }
    pub fn anger() -> Self { Self::new(-0.6, 0.8, 0.5).unwrap() } // Neg valence, High arousal, High dominance
    pub fn fear() -> Self { Self::new(-0.8, 0.9, -0.6).unwrap() } // Neg valence, High arousal, Low dominance
    pub fn sadness() -> Self { Self::new(-0.6, -0.4, -0.3).unwrap() } // Neg valence, Low arousal
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0).unwrap()
    }
}

