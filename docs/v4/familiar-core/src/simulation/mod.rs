//! Simulation Module
//!
//! Contains the core simulation logic:
//! - Entity spawning from WeaveUnits
//! - Physics calculations (QFT field dynamics)
//! - Heddle classification and collapse
//! - Thread resolution to prevent ghost entities

pub mod spawner;
pub mod physics;
pub mod entity_spawn;
pub mod resolver;

// Re-export spawner functions
pub use spawner::{
    spawn_from_weave_unit, spawn_from_weave_units,
    entity_id, entity_type_name, entity_content, entity_physics, entity_physics_values,
    generate_physics,
};

// Re-export physics functions
pub use physics::{get_field_potential_manifest, minimize_action, calculate_field_potential};

// Re-export entity spawn types
pub use entity_spawn::{EntitySpawn, get_heddle_manifest, DEFAULT_COLLAPSE_THRESHOLD};

// Re-export resolver types
pub use resolver::{ThreadResolver, ResolverConfig, ResolverError, ResolverResult};
