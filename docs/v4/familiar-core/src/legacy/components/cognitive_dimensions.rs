use serde::{Deserialize, Serialize};

use crate::primitives::NormalizedFloat;

/// Defines the "personality" of an entity using the Five-Factor Model (OCEAN).
/// These dimensions directly modulate the physics constants of the simulation.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CognitiveDimensions {
    /// **Openness to Experience**
    /// Physics: **Simulation Temperature**.
    /// High: High exploration, high volatility, accepts new connections easily.
    /// Low: Rigid structure, low noise tolerance.
    pub openness: NormalizedFloat,

    /// **Conscientiousness**
    /// Physics: **Spring Stiffness / Rigidity**.
    /// High: Strong restorative forces, low drift, adheres to intent.
    /// Low: Loose coupling, high drift.
    pub conscientiousness: NormalizedFloat,

    /// **Extraversion**
    /// Physics: **Social Gravity Mass**.
    /// High: Large attractive radius, radiates energy (Arousal) to neighbors.
    /// Low: Small radius, absorbs energy.
    pub extraversion: NormalizedFloat,

    /// **Agreeableness**
    /// Physics: **Damping / Friction**.
    /// High: High damping (absorbs shocks/conflict), reduces system energy.
    /// Low: Low damping (elastic collisions), amplifies conflict.
    pub agreeableness: NormalizedFloat,

    /// **Neuroticism (Emotional Stability)**
    /// Physics: **Volatility / Reactivity**.
    /// High: Low inertia (easily moved by small forces), unstable equilibrium.
    /// Low: High inertia (stable), resilient to perturbations.
    pub neuroticism: NormalizedFloat,
}

impl CognitiveDimensions {
    pub fn new(o: f64, c: f64, e: f64, a: f64, n: f64) -> Result<Self, String> {
        Ok(Self {
            openness: NormalizedFloat::new(o)?,
            conscientiousness: NormalizedFloat::new(c)?,
            extraversion: NormalizedFloat::new(e)?,
            agreeableness: NormalizedFloat::new(a)?,
            neuroticism: NormalizedFloat::new(n)?,
        })
    }
}

impl Default for CognitiveDimensions {
    fn default() -> Self {
        // Balanced personality
        Self::new(0.5, 0.5, 0.5, 0.5, 0.5).unwrap()
    }
}

