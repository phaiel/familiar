//! Physics and VAE space calculations
//!
//! Provides methods for physics simulation types.

use familiar_primitives::{QuantizedCoord, SignedNormalizedFloat};

// =============================================================================
// VAE Position
// =============================================================================

/// A position in VAE latent space
#[derive(Debug, Clone, Copy, Default)]
pub struct VAEPosition {
    pub x: QuantizedCoord,
    pub y: QuantizedCoord,
    pub z: QuantizedCoord,
}

impl VAEPosition {
    pub fn origin() -> Self {
        Self::default()
    }

    pub fn from_normalized(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: QuantizedCoord::from_normalized(x),
            y: QuantizedCoord::from_normalized(y),
            z: QuantizedCoord::from_normalized(z),
        }
    }

    pub fn distance_squared(&self, other: &Self) -> i64 {
        let dx = (self.x - other.x).value();
        let dy = (self.y - other.y).value();
        let dz = (self.z - other.z).value();
        dx * dx + dy * dy + dz * dz
    }
}

// Additional physics impls will be added as types are generated


