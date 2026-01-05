use serde::{Deserialize, Serialize};

/// Sparse container for physics hints extracted from text by the LLM.
/// All fields are Option<f64>. If None, the system applies Vacuum State defaults.
/// This is the "raw" output from classification before collapse.
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHint {
    /// Emotional polarity: -1.0 (Negative) to 1.0 (Positive)
    /// Maps to: VAE Valence axis
    pub valence: Option<f64>,
    
    /// Energy/Activation: 0.0 (Calm) to 1.0 (Excited)
    /// Maps to: VAE Arousal axis
    pub arousal: Option<f64>,
    
    /// Importance/Mass: 0.0 (Mundane) to 1.0 (Critical)
    /// Maps to: FieldExcitation.amplitude
    pub significance: Option<f64>,
    
    /// Specificity/Coherence: 0.0 (Vague) to 1.0 (Factual)
    /// Maps to: QuantumState.coherence and VAE Epistemic axis
    pub clarity: Option<f64>,
    
    /// Intrusiveness: 0.0 (Passive) to 1.0 (Demanding attention)
    /// Maps to: CognitiveOptics.emissivity
    pub intrusiveness: Option<f64>,
    
    /// Volatility: 0.0 (Stable) to 1.0 (Fluctuating)
    /// Maps to: FieldExcitation.temperature
    pub volatility: Option<f64>,
}

impl PhysicsHint {
    /// Get valence with vacuum state default (neutral)
    pub fn valence_or_vacuum(&self) -> f64 {
        self.valence.unwrap_or(0.0)
    }
    
    /// Get arousal with vacuum state default (calm)
    pub fn arousal_or_vacuum(&self) -> f64 {
        self.arousal.unwrap_or(0.0)
    }
    
    /// Get significance with vacuum state default (light/fleeting)
    pub fn significance_or_vacuum(&self) -> f64 {
        self.significance.unwrap_or(0.1)
    }
    
    /// Get clarity with vacuum state default (foggy/vague)
    pub fn clarity_or_vacuum(&self) -> f64 {
        self.clarity.unwrap_or(0.1)
    }
    
    /// Get intrusiveness with vacuum state default (passive)
    pub fn intrusiveness_or_vacuum(&self) -> f64 {
        self.intrusiveness.unwrap_or(0.0)
    }
    
    /// Get volatility with vacuum state default (stable)
    pub fn volatility_or_vacuum(&self) -> f64 {
        self.volatility.unwrap_or(0.1)
    }
}

