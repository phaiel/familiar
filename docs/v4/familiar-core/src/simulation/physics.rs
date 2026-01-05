//! Physics Simulation - QFT-based field dynamics
//!
//! Implements Quantum Field Theory concepts for the cognitive manifold.
//! Excitations minimize action by following field gradients.
//!
//! ## Numerical Precision
//!
//! All simulation math operates in f64 space using FieldExcitation's workspace fields.
//! This avoids precision loss from repeated i64<->f64 conversions:
//!
//! ```rust,ignore
//! // Load from DB and hydrate
//! let mut excitations: Vec<FieldExcitation> = db.load_all()?;
//! for e in &mut excitations { e.hydrate(); }
//!
//! // Simulation loop - pure f64 math
//! for _ in 0..1000 {
//!     for e in &mut excitations {
//!         let gradient = calculate_field_gradient(e.position_f64().unwrap(), &excitations);
//!         minimize_action(e, gradient, DT);
//!     }
//! }
//!
//! // Dehydrate and persist
//! for e in &mut excitations { e.dehydrate(); }
//! db.save_all(&excitations)?;
//! ```

use familiar_contracts::FieldExcitation;
use crate::config::{SystemManifest, SystemDomain, SystemTrigger};

/// Default simulation timestep (seconds)
pub const DEFAULT_DT: f64 = 0.016; // ~60 FPS

/// QFT-based system: Excitations move to minimize the "Action" of the field.
/// In Quantum Field Theory, particles follow the "Path of Least Action" by moving
/// down the gradient of the field potential.
pub fn get_field_potential_manifest() -> SystemManifest {
    SystemManifest {
        id: "sys_physics_field_potential".to_string(),
        domain: SystemDomain::Physics,
        description: "Applies QFT field dynamics: excitations minimize action by following field gradients.".to_string(),
        reads: vec!["FieldExcitation".to_string()],
        writes: vec!["FieldExcitation".to_string()],
        trigger: SystemTrigger::Schedule("*/1 * * * * *".to_string()), // Every second
    }
}

/// QFT Style: The excitation moves to minimize the "Action" of the field.
/// 
/// **Requires:** `target.hydrate()` must be called before simulation.
/// Uses f64 workspace to avoid numerical drift from i64 truncation.
///
/// # Arguments
/// * `target` - The excitation to update (must be hydrated)
/// * `field_gradient` - The slope of the VAE manifold at this point
/// * `dt` - Timestep in seconds (use DEFAULT_DT for standard tick)
///
/// # Panics
/// Debug builds panic if target is not hydrated.
pub fn minimize_action(
    target: &mut FieldExcitation,
    field_gradient: [f64; 3],
    dt: f64,
) {
    debug_assert!(target.is_hydrated(), "Call hydrate() before simulation");
    
    // Get current velocity from f64 workspace
    let Some(vel) = target.velocity_f64() else {
        return; // Not hydrated, skip
    };
    
    // In QFT, excitations follow the "Path of Least Action"
    // We move the excitation down the gradient of the field
    
    // Convert gradient to acceleration (scaled by temperature for Brownian motion)
    // Higher temperature = more random motion (quantum fluctuations)
    let temp_factor = target.temperature.value();
    let acceleration = [
        field_gradient[0] * -1.0 * (1.0 + temp_factor),
        field_gradient[1] * -1.0 * (1.0 + temp_factor),
        field_gradient[2] * -1.0 * (1.0 + temp_factor),
    ];
    
    // Update velocity: v' = v + a*dt
    target.set_velocity_f64([
        vel[0] + acceleration[0] * dt,
        vel[1] + acceleration[1] * dt,
        vel[2] + acceleration[2] * dt,
    ]);
    
    // Update position: p' = p + v*dt (using new velocity)
    target.apply_velocity(dt);
}

/// Calculate the field gradient at a position in the VAE manifold.
/// 
/// The gradient points "uphill" - excitations will move in the opposite direction.
///
/// # Arguments
/// * `position` - Current position in f64 space (from workspace)
/// * `neighbors` - Other excitations that contribute to the field
pub fn calculate_field_gradient(
    position: [f64; 3],
    neighbors: &[FieldExcitation],
) -> [f64; 3] {
    let mut gradient = [0.0, 0.0, 0.0];
    
    for neighbor in neighbors {
        let Some(neighbor_pos) = neighbor.position_f64() else {
            continue; // Not hydrated, skip
        };
        
        // Vector from neighbor to this position
        let dx = position[0] - neighbor_pos[0];
        let dy = position[1] - neighbor_pos[1];
        let dz = position[2] - neighbor_pos[2];
        
        // Distance squared (avoid sqrt for performance)
        let dist_sq = dx * dx + dy * dy + dz * dz;
        
        // Skip if too close (avoid division by near-zero)
        if dist_sq < 1e-10 {
            continue;
        }
        
        // Field contribution scales with amplitude, decays with distance squared
        // (gravity-like 1/r^2 falloff)
        let amplitude = neighbor.amplitude.value();
        let scale = amplitude / dist_sq;
        
        // Accumulate gradient (points away from high-amplitude regions)
        gradient[0] += dx * scale;
        gradient[1] += dy * scale;
        gradient[2] += dz * scale;
    }
    
    gradient
}

/// Calculate the field potential at a given point in the VAE manifold.
/// This represents the "energy landscape" that excitations move through.
///
/// # Arguments
/// * `position` - Position in f64 space
/// * `neighbors` - Other excitations that contribute to the field
pub fn calculate_field_potential(
    position: [f64; 3],
    neighbors: &[FieldExcitation],
) -> f64 {
    let mut potential = 0.0;
    
    for neighbor in neighbors {
        let Some(neighbor_pos) = neighbor.position_f64() else {
            continue;
        };
        
        // Distance to neighbor
        let dx = position[0] - neighbor_pos[0];
        let dy = position[1] - neighbor_pos[1];
        let dz = position[2] - neighbor_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        
        if dist < 1e-10 {
            continue;
        }
        
        // Potential from this neighbor (amplitude / distance)
        potential += neighbor.amplitude.value() / dist;
    }
    
    potential
}

