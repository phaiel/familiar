//! Simulation Context for ECS-based physics simulation
//!
//! ## Thread Safety
//!
//! `hecs::World` is NOT `Sync`, meaning it cannot be shared across threads safely.
//! This is by design - ECS worlds should be owned by a single thread (the simulation thread).
//!
//! ## Usage Patterns
//!
//! **Pattern A: Dedicated Simulation Thread (Recommended)**
//! ```rust,ignore
//! // Spawn a dedicated thread for simulation
//! let (tx, rx) = tokio::sync::mpsc::channel(100);
//! std::thread::spawn(move || {
//!     let mut ctx = SimulationContext::new();
//!     while let Ok(cmd) = rx.blocking_recv() {
//!         ctx.process_command(cmd);
//!     }
//! });
//! ```
//!
//! **Pattern B: Single-Threaded Runtime**
//! ```rust,ignore
//! // Use tokio's LocalSet for single-threaded async
//! let local = tokio::task::LocalSet::new();
//! local.run_until(async move {
//!     let ctx = SimulationContext::new();
//!     // All simulation work happens on this thread
//! }).await;
//! ```
//!
//! **Pattern C: Query Interface (for API access)**
//! If the API needs to query world state, use message passing:
//! - Send query requests to the simulation thread
//! - Receive query results via channel
//! - Never share the World directly

use hecs::{World, Entity};
use crate::components::{FieldExcitation, QuantumState, RelationalDynamics, CognitiveOptics, SimLOD};

/// ECS-based simulation context
///
/// **IMPORTANT**: This type is intentionally NOT `Send` or `Sync`.
/// The simulation must run on a single dedicated thread.
/// See module documentation for usage patterns.
///
/// The `_not_send` marker ensures this type cannot be sent across threads,
/// even if `hecs::World`'s implementation changes in the future.
pub struct SimulationContext {
    world: World,
    // Marker to ensure !Send + !Sync (PhantomData<*const ()> is !Send + !Sync)
    _not_send: std::marker::PhantomData<*const ()>,
}

impl SimulationContext {
    /// Create a new simulation context
    ///
    /// Must be called on the thread that will own the simulation.
    pub fn new() -> Self {
        Self { 
            world: World::new(),
            _not_send: std::marker::PhantomData,
        }
    }
    
    /// Get a reference to the underlying ECS world
    ///
    /// Use this for queries and iteration.
    pub fn world(&self) -> &World {
        &self.world
    }
    
    /// Get a mutable reference to the underlying ECS world
    ///
    /// Use this for spawning, despawning, and component modification.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Load an entity from the DB into the Simulation (Conscious Tier by default)
    pub fn spawn_active_entity(
        &mut self, 
        physics: FieldExcitation, 
        quantum: QuantumState,
        relations: Option<RelationalDynamics>,
        optics: Option<CognitiveOptics>
    ) -> Entity {
        // hecs allows dynamic composition of components at runtime
        let lod = SimLOD::default(); // Tier::Conscious
        
        // Basic builder pattern to spawn with optional components
        let mut builder = hecs::EntityBuilder::new();
        builder.add(physics);
        builder.add(quantum);
        builder.add(lod);
        
        if let Some(rel) = relations {
            builder.add(rel);
        }
        
        if let Some(opt) = optics {
            builder.add(opt);
        }
        
        self.world.spawn(builder.build())
    }
}

impl Default for SimulationContext {
    fn default() -> Self {
        Self::new()
    }
}

