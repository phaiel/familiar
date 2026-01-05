use serde::{Deserialize, Serialize};

use crate::primitives::{NormalizedFloat, QuantizedCoord};

/// Represents an excitation (ripple) in the Cognitive Field.
/// In QFT, entities are not particles moving through space; they are temporary excitations
/// of the field itself. A memory is a "Symmetry Breaking" event that freezes a ripple.
///
/// ## Simulation Workspace Pattern
///
/// To avoid numerical drift from repeated i64<->f64 conversions during simulation:
/// 1. Call `hydrate()` after loading from DB - populates f64 workspace fields
/// 2. Run simulation math using `position_f64()` and `velocity_f64()` 
/// 3. Call `dehydrate()` before persisting - flushes f64 back to quantized
///
/// ```rust,ignore
/// let mut excitation = load_from_db()?;
/// excitation.hydrate();
///
/// // Simulation loop - use f64 workspace
/// for _ in 0..1000 {
///     let pos = excitation.position_f64().unwrap();
///     let vel = excitation.velocity_f64().unwrap();
///     // ... physics math in f64 ...
///     excitation.set_position_f64([new_x, new_y, new_z]);
/// }
///
/// excitation.dehydrate();
/// save_to_db(&excitation)?;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FieldExcitation {
    // --- PERSISTENT FIELDS (quantized for DB storage) ---
    
    /// The location in the 3D VAE Manifold where the field is excited
    pub position: [QuantizedCoord; 3],
    
    /// The rate of change of the field amplitude at this location (Cognitive Drift)
    pub velocity: [QuantizedCoord; 3],
    
    /// **Amplitude (Mass):**
    /// The strength of the field excitation.
    /// High Amplitude: Core Beliefs, Long-term Memories. Hard to annihilate.
    /// Low Amplitude: Fleeting thoughts. Easily absorbed back into the vacuum.
    pub amplitude: NormalizedFloat,

    /// **Energy (Activation):**
    /// The current potential/kinetic energy of the excitation.
    pub energy: NormalizedFloat,

    /// **Temperature (Volatility):**
    /// Internal kinetic energy of the field at this point.
    /// High Temp: The field vibrates (High uncertainty/plasticity).
    /// Low Temp: The field is frozen (Crystallized/Immutable).
    pub temperature: NormalizedFloat,
    
    // --- WORKSPACE FIELDS (f64 for simulation math, not persisted) ---
    
    /// f64 position workspace for simulation (avoids i64 truncation per tick)
    #[serde(skip)]
    position_workspace: Option<[f64; 3]>,
    
    /// f64 velocity workspace for simulation
    #[serde(skip)]
    velocity_workspace: Option<[f64; 3]>,
}

impl FieldExcitation {
    /// Create a new field excitation from raw values
    pub fn new(
        position: [i64; 3], 
        velocity: [i64; 3], 
        amplitude: f64, 
        energy: f64,
        temperature: f64
    ) -> Result<Self, String> {
        Ok(Self {
            position: [
                QuantizedCoord::new(position[0]),
                QuantizedCoord::new(position[1]),
                QuantizedCoord::new(position[2]),
            ],
            velocity: [
                QuantizedCoord::new(velocity[0]),
                QuantizedCoord::new(velocity[1]),
                QuantizedCoord::new(velocity[2]),
            ],
            amplitude: NormalizedFloat::new(amplitude)?,
            energy: NormalizedFloat::new(energy)?,
            temperature: NormalizedFloat::new(temperature)?,
            // Workspace starts empty - call hydrate() for simulation
            position_workspace: None,
            velocity_workspace: None,
        })
    }
    
    // ========================================================================
    // Simulation Workspace Methods
    // ========================================================================
    
    /// Populate f64 workspace from quantized values
    /// 
    /// Call this after loading from database, before running simulation.
    /// The workspace avoids precision loss from repeated i64<->f64 conversions.
    pub fn hydrate(&mut self) {
        self.position_workspace = Some([
            self.position[0].to_normalized(),
            self.position[1].to_normalized(),
            self.position[2].to_normalized(),
        ]);
        self.velocity_workspace = Some([
            self.velocity[0].to_normalized(),
            self.velocity[1].to_normalized(),
            self.velocity[2].to_normalized(),
        ]);
    }
    
    /// Flush f64 workspace back to quantized values
    /// 
    /// Call this before persisting to database, after simulation completes.
    pub fn dehydrate(&mut self) {
        if let Some(pos) = self.position_workspace {
            self.position = [
                QuantizedCoord::from_normalized(pos[0]),
                QuantizedCoord::from_normalized(pos[1]),
                QuantizedCoord::from_normalized(pos[2]),
            ];
        }
        if let Some(vel) = self.velocity_workspace {
            self.velocity = [
                QuantizedCoord::from_normalized(vel[0]),
                QuantizedCoord::from_normalized(vel[1]),
                QuantizedCoord::from_normalized(vel[2]),
            ];
        }
        // Clear workspace after flush
        self.position_workspace = None;
        self.velocity_workspace = None;
    }
    
    /// Check if workspace is hydrated (ready for simulation)
    pub fn is_hydrated(&self) -> bool {
        self.position_workspace.is_some() && self.velocity_workspace.is_some()
    }
    
    /// Get f64 position for simulation math
    /// 
    /// Returns None if not hydrated - call hydrate() first.
    pub fn position_f64(&self) -> Option<[f64; 3]> {
        self.position_workspace
    }
    
    /// Get f64 velocity for simulation math
    pub fn velocity_f64(&self) -> Option<[f64; 3]> {
        self.velocity_workspace
    }
    
    /// Set f64 position during simulation
    /// 
    /// Panics if not hydrated (debug builds only).
    pub fn set_position_f64(&mut self, pos: [f64; 3]) {
        debug_assert!(self.is_hydrated(), "Call hydrate() before simulation");
        self.position_workspace = Some(pos);
    }
    
    /// Set f64 velocity during simulation
    pub fn set_velocity_f64(&mut self, vel: [f64; 3]) {
        debug_assert!(self.is_hydrated(), "Call hydrate() before simulation");
        self.velocity_workspace = Some(vel);
    }
    
    /// Apply a velocity update to position (simulation tick)
    /// 
    /// This operates entirely in f64 space, avoiding quantization until dehydrate().
    pub fn apply_velocity(&mut self, dt: f64) {
        if let (Some(pos), Some(vel)) = (self.position_workspace, self.velocity_workspace) {
            self.position_workspace = Some([
                pos[0] + vel[0] * dt,
                pos[1] + vel[1] * dt,
                pos[2] + vel[2] * dt,
            ]);
        }
    }
    
    /// Apply acceleration to velocity (simulation tick)
    pub fn apply_acceleration(&mut self, accel: [f64; 3], dt: f64) {
        if let Some(vel) = self.velocity_workspace {
            self.velocity_workspace = Some([
                vel[0] + accel[0] * dt,
                vel[1] + accel[1] * dt,
                vel[2] + accel[2] * dt,
            ]);
        }
    }
}

