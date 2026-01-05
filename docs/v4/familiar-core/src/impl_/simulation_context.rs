//! Impl module for simulation_context types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SimulationContext

// Methods: new, world, world_mut, spawn_active_entity
impl SimulationContext { # [doc = " Create a new simulation context"] # [doc = ""] # [doc = " Must be called on the thread that will own the simulation."] pub fn new () -> Self { Self { world : World :: new () , _not_send : std :: marker :: PhantomData , } } # [doc = " Get a reference to the underlying ECS world"] # [doc = ""] # [doc = " Use this for queries and iteration."] pub fn world (& self) -> & World { & self . world } # [doc = " Get a mutable reference to the underlying ECS world"] # [doc = ""] # [doc = " Use this for spawning, despawning, and component modification."] pub fn world_mut (& mut self) -> & mut World { & mut self . world } # [doc = " Load an entity from the DB into the Simulation (Conscious Tier by default)"] pub fn spawn_active_entity (& mut self , physics : FieldExcitation , quantum : QuantumState , relations : Option < RelationalDynamics > , optics : Option < CognitiveOptics >) -> Entity { let lod = SimLOD :: default () ; let mut builder = hecs :: EntityBuilder :: new () ; builder . add (physics) ; builder . add (quantum) ; builder . add (lod) ; if let Some (rel) = relations { builder . add (rel) ; } if let Some (opt) = optics { builder . add (opt) ; } self . world . spawn (builder . build ()) } }

// Trait impl: Default
impl Default for SimulationContext { fn default () -> Self { Self :: new () } }

