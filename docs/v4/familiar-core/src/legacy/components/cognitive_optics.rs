use serde::{Deserialize, Serialize};

use crate::primitives::NormalizedFloat;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CognitiveOptics {
    /// **Emissivity (Luminance):**
    /// The object's internal energy source.
    /// High: "Stars" (Trauma, Obsessions, Core Values). They glow in the dark.
    /// Low: "Planets" (Facts). They need external attention to be seen.
    pub emissivity: NormalizedFloat,

    /// **Albedo (Reflectivity):**
    /// How well the object reflects the User's Attention.
    /// 1.0 = Mirror (Perfect Recall).
    /// 0.0 = Vantablack (Repressed/Forgotten).
    pub albedo: NormalizedFloat,

    /// **Roughness (Scattering):**
    /// 0.0 = Smooth/Polished. Rays bounce predictably (Logical/Linear thought).
    /// 1.0 = Rough/Matte. Rays scatter randomly (Creative/Associative thought).
    pub roughness: NormalizedFloat,

    /// **Occlusion (Density):**
    /// How much this object blocks rays behind it.
    /// High: A massive trauma that hides childhood memories behind it.
    pub occlusion: NormalizedFloat,
}

impl CognitiveOptics {
    pub fn new(emissivity: f64, albedo: f64, roughness: f64, occlusion: f64) -> Result<Self, String> {
        Ok(Self {
            emissivity: NormalizedFloat::new(emissivity)?,
            albedo: NormalizedFloat::new(albedo)?,
            roughness: NormalizedFloat::new(roughness)?,
            occlusion: NormalizedFloat::new(occlusion)?,
        })
    }
}

impl Default for CognitiveOptics {
    fn default() -> Self {
        Self {
            emissivity: NormalizedFloat::new(0.0).unwrap(), // Dark by default
            albedo: NormalizedFloat::new(0.5).unwrap(),     // Grey
            roughness: NormalizedFloat::new(0.5).unwrap(),  // Semi-gloss
            occlusion: NormalizedFloat::new(0.1).unwrap(),  // Mostly transparent
        }
    }
}

