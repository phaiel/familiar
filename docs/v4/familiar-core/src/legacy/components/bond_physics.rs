use serde::{Deserialize, Serialize};

use crate::primitives::NormalizedFloat;

/// Models the classical physics of a relationship Bond.
/// Based on Spring-Damper mechanics (Hooke's Law).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BondPhysics {
    /// The 'stiffness' of the bond, representing its resistance to change.
    /// Higher values = more rigid relationship (breaks under stress).
    /// Lower values = more flexible relationship (stretches without breaking).
    pub spring_constant: f64,
    
    /// The 'inertia' of the bond, representing how quickly it returns to equilibrium.
    /// Higher values = slower return to rest (lingering effects).
    /// Lower values = quick recovery (resilient).
    pub damping_coefficient: f64,
    
    /// The overall strength or health of the bond.
    /// 0.0 = Broken/Weak, 1.0 = Strong/Healthy
    pub bond_strength: NormalizedFloat,
}

impl BondPhysics {
    pub fn new(spring_constant: f64, damping: f64, strength: f64) -> Result<Self, String> {
        Ok(Self {
            spring_constant,
            damping_coefficient: damping,
            bond_strength: NormalizedFloat::new(strength)?,
        })
    }
    
    /// Calculate the force on the bond given a displacement from rest length.
    /// F = -kx - cv (Hooke's Law with damping)
    pub fn calculate_force(&self, displacement: f64, velocity: f64) -> f64 {
        let spring_force = -self.spring_constant * displacement;
        let damping_force = -self.damping_coefficient * velocity;
        spring_force + damping_force
    }
}

impl Default for BondPhysics {
    fn default() -> Self {
        Self {
            spring_constant: 1.0,           // Moderate stiffness
            damping_coefficient: 0.5,       // Moderate damping
            bond_strength: NormalizedFloat::new(0.5).unwrap(), // Neutral strength
        }
    }
}
