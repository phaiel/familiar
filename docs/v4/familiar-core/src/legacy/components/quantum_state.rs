use serde::{Deserialize, Serialize};
use crate::primitives::NormalizedFloat;

/// Represents the wave-nature of an entity.
/// Instead of a full qubit simulation (O(2^N)), we use Complex Vectors (O(N)).
/// This supports Interference, Phase, and Resonance natively in Vector DBs.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QuantumState {
    /// The "Wavefunction" or "Hologram" of the entity.
    /// Stored as a list of (Real, Imaginary) tuples.
    /// This represents Amplitude (Magnitude) and Phase (Angle).
    /// 
    /// In a Vector DB, this is flattened to [Re1, Im1, Re2, Im2...].
    pub amplitudes: Vec<(f64, f64)>,
    
    /// How "sharp" the wave is (0.0 = Cloud/Vague, 1.0 = Particle/Fact).
    /// Low coherence allows for "Tunneling" (associative leaps).
    pub coherence: NormalizedFloat,
    
    /// The dominant frequency of the pattern (for Holonomic resonance).
    pub frequency: Option<f64>,
}

impl QuantumState {
    /// Create a new state from a semantic vector (e.g., LLM embedding).
    /// Initializes with Phase = 0 (Real numbers only).
    pub fn from_embedding(embedding: Vec<f64>) -> Self {
        let amplitudes = embedding.into_iter().map(|v| (v, 0.0)).collect();
        Self {
            amplitudes,
            coherence: NormalizedFloat::new(1.0).unwrap(),
            frequency: None,
        }
    }
}
