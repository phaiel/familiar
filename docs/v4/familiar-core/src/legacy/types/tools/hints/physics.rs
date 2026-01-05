//! Physics Hint Tool Schema
//!
//! Maps emotional and cognitive content to VAE space coordinates.
//! Used for physics simulation and spatial organization of memories.

use serde::{Deserialize, Serialize};

use crate::types::tools::entity::EntityType;
use crate::types::tools::segmentation::Segment;

// ============================================================================
// Physics Hint Tool
// ============================================================================

/// Input for physics hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHintInput {
    /// The segment to analyze
    pub segment: Segment,
    /// Entity type being spawned
    pub entity_type: EntityType,
    /// Content text
    pub content: String,
    /// Previous entity positions for continuity
    #[serde(default)]
    pub previous_positions: Vec<VAEPosition>,
}

/// Output from physics hint generation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHintOutput {
    /// The computed physics hints
    pub hints: PhysicsHintValues,
    /// Computed VAE position
    pub position: VAEPosition,
    /// Analysis that led to these values
    pub analysis: PhysicsAnalysis,
}

/// Physics hint values
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHintValues {
    /// Emotional valence: -1.0 (very negative) to 1.0 (very positive)
    /// Maps to X-axis in VAE space
    pub valence: f64,
    
    /// Arousal/activation: 0.0 (calm/low energy) to 1.0 (excited/high energy)
    /// Maps to Y-axis in VAE space
    pub arousal: f64,
    
    /// Epistemic certainty: 0.0 (uncertain) to 1.0 (certain)
    /// Maps to Z-axis in VAE space
    pub epistemic: f64,
    
    /// Significance/mass: 0.0 (trivial) to 1.0 (very important)
    /// Affects physics simulation weight
    pub significance: f64,
    
    /// Energy level for simulation
    pub energy: f64,
    
    /// Temperature (volatility/changeability)
    pub temperature: f64,
}

impl Default for PhysicsHintValues {
    fn default() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            epistemic: 0.5,
            significance: 0.5,
            energy: 0.5,
            temperature: 0.5,
        }
    }
}

impl PhysicsHintValues {
    /// Create from simple emotional descriptors
    pub fn from_emotion(emotion: &str, intensity: f64) -> Self {
        let intensity = intensity.clamp(0.0, 1.0);
        
        let (valence, arousal, energy) = match emotion.to_lowercase().as_str() {
            // Positive high-energy
            "excited" | "elated" | "thrilled" => (0.9 * intensity, 0.9 * intensity, 0.8),
            "happy" | "joyful" | "pleased" => (0.8 * intensity, 0.6 * intensity, 0.6),
            "proud" | "accomplished" => (0.7 * intensity, 0.5 * intensity, 0.6),
            
            // Positive low-energy
            "calm" | "peaceful" | "serene" => (0.5 * intensity, 0.2 * intensity, 0.3),
            "content" | "satisfied" => (0.6 * intensity, 0.3 * intensity, 0.4),
            "relaxed" => (0.4 * intensity, 0.2 * intensity, 0.3),
            
            // Negative high-energy
            "angry" | "furious" | "outraged" => (-0.8 * intensity, 0.9 * intensity, 0.9),
            "frustrated" | "annoyed" => (-0.5 * intensity, 0.7 * intensity, 0.7),
            "anxious" | "worried" | "stressed" => (-0.4 * intensity, 0.8 * intensity, 0.7),
            "scared" | "afraid" | "terrified" => (-0.7 * intensity, 0.9 * intensity, 0.8),
            
            // Negative low-energy
            "sad" | "depressed" | "down" => (-0.7 * intensity, 0.2 * intensity, 0.3),
            "tired" | "exhausted" => (-0.3 * intensity, 0.1 * intensity, 0.2),
            "bored" | "apathetic" => (-0.2 * intensity, 0.1 * intensity, 0.2),
            "lonely" | "isolated" => (-0.5 * intensity, 0.2 * intensity, 0.3),
            
            // Neutral
            "neutral" | "normal" | "okay" => (0.0, 0.5, 0.5),
            "curious" | "interested" => (0.3 * intensity, 0.6 * intensity, 0.6),
            "surprised" | "shocked" => (0.0, 0.8 * intensity, 0.7),
            
            _ => (0.0, 0.5, 0.5),
        };
        
        Self {
            valence,
            arousal,
            epistemic: 0.5,
            significance: intensity * 0.7,
            energy,
            temperature: arousal * 0.8,
        }
    }

    /// Create from entity type defaults
    pub fn for_entity_type(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Moment => Self {
                valence: 0.0,
                arousal: 0.5,
                epistemic: 0.8, // Moments are typically factual
                significance: 0.5,
                energy: 0.5,
                temperature: 0.3,
            },
            EntityType::Pulse => Self {
                valence: 0.0, // Will be updated based on content
                arousal: 0.5,
                epistemic: 0.6, // Feelings are subjective
                significance: 0.6,
                energy: 0.6,
                temperature: 0.6, // More volatile
            },
            EntityType::Intent => Self {
                valence: 0.3, // Future-oriented tends positive
                arousal: 0.6,
                epistemic: 0.4, // Future is uncertain
                significance: 0.7,
                energy: 0.7,
                temperature: 0.4,
            },
            EntityType::Thread => Self {
                valence: 0.0,
                arousal: 0.3,
                epistemic: 0.7,
                significance: 0.8, // Threads are anchors
                energy: 0.3,
                temperature: 0.2, // Stable
            },
            EntityType::Bond => Self {
                valence: 0.0,
                arousal: 0.4,
                epistemic: 0.6,
                significance: 0.8,
                energy: 0.4,
                temperature: 0.3,
            },
            EntityType::Motif | EntityType::Filament => Self {
                valence: 0.0,
                arousal: 0.4,
                epistemic: 0.7,
                significance: 0.7,
                energy: 0.4,
                temperature: 0.2, // Patterns are stable
            },
            EntityType::Focus => Self {
                valence: 0.4,
                arousal: 0.6,
                epistemic: 0.5,
                significance: 0.9,
                energy: 0.7,
                temperature: 0.5,
            },
        }
    }
}

/// VAE space position (quantized for physics)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct VAEPosition {
    /// X: Valence (emotional polarity)
    pub x: i64,
    /// Y: Arousal (energy/activation)
    pub y: i64,
    /// Z: Epistemic (certainty)
    pub z: i64,
}

impl VAEPosition {
    /// Scale factor for quantization (1M = 1.0)
    pub const SCALE: i64 = 1_000_000;

    /// Create from float values
    pub fn from_floats(valence: f64, arousal: f64, epistemic: f64) -> Self {
        Self {
            x: (valence.clamp(-1.0, 1.0) * Self::SCALE as f64) as i64,
            y: (arousal.clamp(0.0, 1.0) * Self::SCALE as f64) as i64,
            z: (epistemic.clamp(0.0, 1.0) * Self::SCALE as f64) as i64,
        }
    }

    /// Convert to float array
    pub fn to_floats(&self) -> [f64; 3] {
        [
            self.x as f64 / Self::SCALE as f64,
            self.y as f64 / Self::SCALE as f64,
            self.z as f64 / Self::SCALE as f64,
        ]
    }
}

impl From<&PhysicsHintValues> for VAEPosition {
    fn from(hints: &PhysicsHintValues) -> Self {
        Self::from_floats(hints.valence, hints.arousal, hints.epistemic)
    }
}

/// Analysis that led to physics values
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsAnalysis {
    /// Detected emotions
    #[serde(default)]
    pub emotions: Vec<DetectedEmotion>,
    /// Significance indicators
    #[serde(default)]
    pub significance_indicators: Vec<String>,
    /// Certainty indicators
    #[serde(default)]
    pub certainty_indicators: Vec<String>,
    /// Overall reasoning
    pub reasoning: String,
}

/// A detected emotion in the content
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedEmotion {
    /// Emotion name
    pub emotion: String,
    /// Intensity (0.0 to 1.0)
    pub intensity: f64,
    /// Evidence for detection
    pub evidence: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_to_physics() {
        let hints = PhysicsHintValues::from_emotion("happy", 0.8);
        assert!(hints.valence > 0.5);
        assert!(hints.arousal > 0.3);

        let hints = PhysicsHintValues::from_emotion("sad", 0.8);
        assert!(hints.valence < 0.0);
        assert!(hints.arousal < 0.5);
    }

    #[test]
    fn test_vae_position() {
        let hints = PhysicsHintValues {
            valence: 0.5,
            arousal: 0.7,
            epistemic: 0.9,
            ..Default::default()
        };

        let pos = VAEPosition::from(&hints);
        assert_eq!(pos.x, 500_000);
        assert_eq!(pos.y, 700_000);
        assert_eq!(pos.z, 900_000);
    }
}
