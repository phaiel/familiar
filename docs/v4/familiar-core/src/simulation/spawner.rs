//! Entity Spawner - Schema-driven entity creation from WeaveUnits
//!
//! Takes classified WeaveUnits and spawns concrete simulation entities.
//! All logic is schema-driven - no magic numbers or hardcoded values.

use crate::{
    UUID, Timestamp, QuantizedCoord,
    HeddleEntityType, RawPhysicsHint,
    Identity, ContentPayload, FieldExcitation, QuantumState, CognitiveOptics,
    WeaveUnit, TaskDynamics, RelationalDynamics, BondPhysics,
    Moment, Intent, Thread, Bond, Pulse, Motif, Filament, Focus,
};
use super::entity_spawn::EntitySpawn;

// ============================================================================
// Physics Generation from Hints (Schema-Driven)
// ============================================================================

/// Generate physics components from raw LLM hints.
/// Uses "Vacuum State" defaults for missing values.
pub fn generate_physics(hint: &Option<RawPhysicsHint>, _tenant_id: UUID) -> (FieldExcitation, QuantumState, CognitiveOptics) {
    // Extract values with vacuum state defaults
    let valence = hint.as_ref().and_then(|h| h.valence).unwrap_or(0.0);
    let arousal = hint.as_ref().and_then(|h| h.arousal).unwrap_or(0.5);
    let significance = hint.as_ref().and_then(|h| h.significance).unwrap_or(0.5);
    let clarity = hint.as_ref().and_then(|h| h.clarity).unwrap_or(0.5);
    let intrusiveness = hint.as_ref().and_then(|h| h.intrusiveness).unwrap_or(0.0);
    let volatility = hint.as_ref().and_then(|h| h.volatility).unwrap_or(0.5);

    // Map to VAE coordinates (i64 for QuantizedCoord)
    // Valence: -1 to +1 -> normalized
    // Arousal: 0 to 1 -> normalized
    // Epistemic (clarity): 0 to 1 -> normalized
    let v_coord = QuantizedCoord::from_normalized(valence).value();
    let a_coord = QuantizedCoord::from_normalized(arousal * 2.0 - 1.0).value(); // Map 0-1 to -1 to +1
    let e_coord = QuantizedCoord::from_normalized(clarity * 2.0 - 1.0).value();
    
    let physics = FieldExcitation::new(
        [v_coord, a_coord, e_coord],  // position
        [0, 0, 0],                     // velocity (at rest initially)
        significance,                  // amplitude = significance
        arousal,                       // energy = arousal
        volatility,                    // temperature = volatility
    ).unwrap_or_else(|_| {
        // Vacuum state fallback
        FieldExcitation::new([0, 0, 0], [0, 0, 0], 0.5, 0.5, 0.5).unwrap()
    });

    // Quantum state (simple default embedding for now)
    // In production, this would come from an LLM embedding
    let quantum = QuantumState::from_embedding(vec![0.0; 8]);

    // Cognitive optics from physics hints
    let optics = CognitiveOptics::new(
        intrusiveness,  // Emissivity = intrusiveness
        clarity,        // Albedo = clarity (memorable)
        0.5,            // Neutral roughness
        0.0,            // Transparent
    ).unwrap_or_default();

    (physics, quantum, optics)
}

// ============================================================================
// Spawning Logic (Schema-Driven)
// ============================================================================

/// Spawn a single entity from a classification
fn spawn_entity(
    entity_type: HeddleEntityType,
    content: &str,
    tenant_id: UUID,
    physics: FieldExcitation,
    quantum: QuantumState,
) -> EntitySpawn {
    let identity = Identity {
        id: UUID::new(),
        tenant_id,
        created_at: Timestamp::now(),
    };

    let content_payload = ContentPayload {
        text: content.to_string(),
        metadata: std::collections::HashMap::new(),
    };

    match entity_type {
        HeddleEntityType::MOMENT => EntitySpawn::Moment(Moment {
            identity,
            physics,
            quantum,
            content: content_payload,
            moment_type: crate::MomentType::Event,
        }),
        HeddleEntityType::INTENT => EntitySpawn::Intent(Intent {
            identity,
            physics,
            content: content_payload,
            dynamics: TaskDynamics::default(),
            target_date: None,
        }),
        HeddleEntityType::THREAD => EntitySpawn::Thread(Thread {
            identity,
            physics,
            quantum,
            content: content_payload,
            thread_type: crate::ThreadType::Concept,
        }),
        HeddleEntityType::BOND => EntitySpawn::Bond(Bond {
            identity,
            physics,
            bond_physics: BondPhysics::default(),
            content: content_payload,
            dynamics: RelationalDynamics::default(),
            head_thread_id: UUID::new(), // Placeholder: Needs Stitch resolution
            tail_thread_id: UUID::new(),
        }),
        HeddleEntityType::PULSE => EntitySpawn::Pulse(Pulse {
            identity,
            physics,
            content: content_payload,
            state_type: crate::InternalStateType::Observation,
        }),
        HeddleEntityType::MOTIF => EntitySpawn::Motif(Motif {
            identity,
            physics,
            quantum,
            content: content_payload,
            source_moments: vec![],
        }),
        HeddleEntityType::FILAMENT => EntitySpawn::Filament(Filament {
            identity,
            physics,
            quantum,
            content: content_payload,
            source_pulses: vec![],
        }),
        HeddleEntityType::FOCUS => EntitySpawn::Focus(Focus {
            identity,
            physics,
            quantum,
            content: content_payload,
            active_since: Timestamp::now(),
        }),
    }
}

/// Spawn entities from a WeaveUnit based on its classifications.
/// Returns spawned entities. Only LOG purpose units spawn entities.
pub fn spawn_from_weave_unit(
    unit: &WeaveUnit,
    physics_hint: &Option<RawPhysicsHint>,
    tenant_id: UUID,
    threshold: f64,
) -> Vec<EntitySpawn> {
    // Only spawn for units with LOG purpose
    if !unit.should_spawn() {
        return vec![];
    }
    
    let (physics, quantum, _optics) = generate_physics(physics_hint, tenant_id);
    
    unit.classifications_above(threshold)
        .into_iter()
        .map(|c| spawn_entity(
            c.entity_type,
            &unit.content,
            tenant_id,
            physics.clone(),
            quantum.clone(),
        ))
        .collect()
}

/// Spawn entities from multiple WeaveUnits with their physics hints.
/// Returns (unit_index, spawned_entities) for each unit that produced entities.
pub fn spawn_from_weave_units(
    units: &[WeaveUnit],
    physics_hints: &[Option<RawPhysicsHint>],
    tenant_id: UUID,
    threshold: f64,
) -> Vec<(usize, Vec<EntitySpawn>)> {
    units.iter()
        .zip(physics_hints.iter())
        .enumerate()
        .map(|(idx, (unit, hint))| {
            let entities = spawn_from_weave_unit(unit, hint, tenant_id, threshold);
            (idx, entities)
        })
        .filter(|(_, entities)| !entities.is_empty())
        .collect()
}

/// Get the entity ID from an EntitySpawn
pub fn entity_id(spawn: &EntitySpawn) -> UUID {
    match spawn {
        EntitySpawn::Moment(e) => e.identity.id,
        EntitySpawn::Intent(e) => e.identity.id,
        EntitySpawn::Thread(e) => e.identity.id,
        EntitySpawn::Bond(e) => e.identity.id,
        EntitySpawn::Pulse(e) => e.identity.id,
        EntitySpawn::Motif(e) => e.identity.id,
        EntitySpawn::Filament(e) => e.identity.id,
        EntitySpawn::Focus(e) => e.identity.id,
    }
}

/// Get the entity type name from an EntitySpawn
pub fn entity_type_name(spawn: &EntitySpawn) -> &'static str {
    match spawn {
        EntitySpawn::Moment(_) => "Moment",
        EntitySpawn::Intent(_) => "Intent",
        EntitySpawn::Thread(_) => "Thread",
        EntitySpawn::Bond(_) => "Bond",
        EntitySpawn::Pulse(_) => "Pulse",
        EntitySpawn::Motif(_) => "Motif",
        EntitySpawn::Filament(_) => "Filament",
        EntitySpawn::Focus(_) => "Focus",
    }
}

/// Get the content text from an EntitySpawn
pub fn entity_content(spawn: &EntitySpawn) -> &str {
    match spawn {
        EntitySpawn::Moment(e) => &e.content.text,
        EntitySpawn::Intent(e) => &e.content.text,
        EntitySpawn::Thread(e) => &e.content.text,
        EntitySpawn::Bond(e) => &e.content.text,
        EntitySpawn::Pulse(e) => &e.content.text,
        EntitySpawn::Motif(e) => &e.content.text,
        EntitySpawn::Filament(e) => &e.content.text,
        EntitySpawn::Focus(e) => &e.content.text,
    }
}

/// Get the physics (FieldExcitation) from an EntitySpawn
pub fn entity_physics(spawn: &EntitySpawn) -> &FieldExcitation {
    match spawn {
        EntitySpawn::Moment(e) => &e.physics,
        EntitySpawn::Intent(e) => &e.physics,
        EntitySpawn::Thread(e) => &e.physics,
        EntitySpawn::Bond(e) => &e.physics,
        EntitySpawn::Pulse(e) => &e.physics,
        EntitySpawn::Motif(e) => &e.physics,
        EntitySpawn::Filament(e) => &e.physics,
        EntitySpawn::Focus(e) => &e.physics,
    }
}

/// Extract physics values as simple f64 tuple for serialization
/// Returns (position[3], amplitude, energy, temperature)
pub fn entity_physics_values(spawn: &EntitySpawn) -> ([f64; 3], f64, f64, f64) {
    let physics = entity_physics(spawn);
    let position = [
        physics.position[0].to_normalized(),
        physics.position[1].to_normalized(),
        physics.position[2].to_normalized(),
    ];
    (
        position,
        physics.amplitude.value(),
        physics.energy.value(),
        physics.temperature.value(),
    )
}

