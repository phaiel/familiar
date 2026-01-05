//! Impl module for FieldExcitation type
//!
//! This module contains behavior for the generated FieldExcitation type.
//! The type is imported from familiar-contracts.

use familiar_contracts::prelude::*;

/// FieldExcitation implementation
/// 
/// Provides methods for working with the physics simulation workspace.
/// The workspace avoids precision loss from repeated i64<->f64 conversions
/// during simulation ticks.
impl FieldExcitation {
    /// Create a new field excitation from raw values.
    /// 
    /// Position and velocity are 3D vectors [x, y, z].
    /// Amplitude, energy, and temperature are normalized [0.0, 1.0].
    pub fn new(
        position: [i64; 3],
        velocity: [i64; 3],
        amplitude: f64,
        energy: f64,
        temperature: f64,
    ) -> Result<Self, String> {
        Ok(Self {
            position: vec![
                QuantizedCoord::new(position[0]),
                QuantizedCoord::new(position[1]),
                QuantizedCoord::new(position[2]),
            ],
            velocity: vec![
                QuantizedCoord::new(velocity[0]),
                QuantizedCoord::new(velocity[1]),
                QuantizedCoord::new(velocity[2]),
            ],
            amplitude: NormalizedFloat::new(amplitude)?,
            energy: NormalizedFloat::new(energy)?,
            temperature: NormalizedFloat::new(temperature)?,
            position_workspace: None,
            velocity_workspace: None,
        })
    }

    /// Create a default field excitation at origin with zero velocity.
    pub fn default_at_origin() -> Self {
        Self {
            position: vec![
                QuantizedCoord::new(0),
                QuantizedCoord::new(0),
                QuantizedCoord::new(0),
            ],
            velocity: vec![
                QuantizedCoord::new(0),
                QuantizedCoord::new(0),
                QuantizedCoord::new(0),
            ],
            amplitude: NormalizedFloat::new(0.5).unwrap(),
            energy: NormalizedFloat::new(0.5).unwrap(),
            temperature: NormalizedFloat::new(0.5).unwrap(),
            position_workspace: None,
            velocity_workspace: None,
        }
    }

    /// Populate f64 workspace from quantized values.
    /// 
    /// Call this after loading from database, before running simulation.
    /// The workspace avoids precision loss from repeated i64<->f64 conversions.
    pub fn hydrate(&mut self) {
        if self.position.len() >= 3 {
            self.position_workspace = Some(vec![
                self.position[0].to_normalized(),
                self.position[1].to_normalized(),
                self.position[2].to_normalized(),
            ]);
        }
        if self.velocity.len() >= 3 {
            self.velocity_workspace = Some(vec![
                self.velocity[0].to_normalized(),
                self.velocity[1].to_normalized(),
                self.velocity[2].to_normalized(),
            ]);
        }
    }

    /// Flush f64 workspace back to quantized values.
    /// 
    /// Call this before persisting to database, after simulation completes.
    pub fn dehydrate(&mut self) {
        if let Some(pos) = &self.position_workspace {
            if pos.len() >= 3 {
                self.position = vec![
                    QuantizedCoord::from_normalized(pos[0]),
                    QuantizedCoord::from_normalized(pos[1]),
                    QuantizedCoord::from_normalized(pos[2]),
                ];
            }
        }
        if let Some(vel) = &self.velocity_workspace {
            if vel.len() >= 3 {
                self.velocity = vec![
                    QuantizedCoord::from_normalized(vel[0]),
                    QuantizedCoord::from_normalized(vel[1]),
                    QuantizedCoord::from_normalized(vel[2]),
                ];
            }
        }
        self.position_workspace = None;
        self.velocity_workspace = None;
    }

    /// Check if workspace is hydrated (ready for simulation).
    pub fn is_hydrated(&self) -> bool {
        self.position_workspace.is_some() && self.velocity_workspace.is_some()
    }

    /// Get f64 position for simulation math.
    /// 
    /// Returns None if not hydrated - call hydrate() first.
    pub fn position_f64(&self) -> Option<[f64; 3]> {
        self.position_workspace.as_ref().and_then(|p| {
            if p.len() >= 3 {
                Some([p[0], p[1], p[2]])
            } else {
                None
            }
        })
    }

    /// Get f64 velocity for simulation math.
    pub fn velocity_f64(&self) -> Option<[f64; 3]> {
        self.velocity_workspace.as_ref().and_then(|v| {
            if v.len() >= 3 {
                Some([v[0], v[1], v[2]])
            } else {
                None
            }
        })
    }

    /// Set f64 position during simulation.
    /// 
    /// Panics if not hydrated (debug builds only).
    pub fn set_position_f64(&mut self, pos: [f64; 3]) {
        debug_assert!(self.is_hydrated(), "Call hydrate() before simulation");
        self.position_workspace = Some(vec![pos[0], pos[1], pos[2]]);
    }

    /// Set f64 velocity during simulation.
    pub fn set_velocity_f64(&mut self, vel: [f64; 3]) {
        debug_assert!(self.is_hydrated(), "Call hydrate() before simulation");
        self.velocity_workspace = Some(vec![vel[0], vel[1], vel[2]]);
    }

    /// Apply a velocity update to position (simulation tick).
    /// 
    /// This operates entirely in f64 space, avoiding quantization until dehydrate().
    pub fn apply_velocity(&mut self, dt: f64) {
        if let (Some(pos), Some(vel)) = (&self.position_workspace, &self.velocity_workspace) {
            if pos.len() >= 3 && vel.len() >= 3 {
                self.position_workspace = Some(vec![
                    pos[0] + vel[0] * dt,
                    pos[1] + vel[1] * dt,
                    pos[2] + vel[2] * dt,
                ]);
            }
        }
    }

    /// Apply acceleration to velocity (simulation tick).
    pub fn apply_acceleration(&mut self, accel: [f64; 3], dt: f64) {
        if let Some(vel) = &self.velocity_workspace {
            if vel.len() >= 3 {
                self.velocity_workspace = Some(vec![
                    vel[0] + accel[0] * dt,
                    vel[1] + accel[1] * dt,
                    vel[2] + accel[2] * dt,
                ]);
            }
        }
    }
}

// Trait impl: Component (for ECS-like patterns)
impl Component for FieldExcitation {}
