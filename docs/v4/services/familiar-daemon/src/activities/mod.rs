//! Activities Module
//!
//! Temporal activities that run on this Rust worker.
//! The TypeScript workflow worker orchestrates these activities.
//!
//! ## Registration Pattern
//!
//! Activities are registered with the worker in main.rs using closures
//! that capture the SharedState (Arc<HotState>). This allows each
//! activity to access the hot resources without re-initialization.

mod fates;

pub use fates::*;

use crate::state::SharedState;
use temporalio_sdk::{ActivityError, Worker};

/// Register all Fates activities with the worker
///
/// Each activity is registered with a name that matches the TypeScript
/// interface in `familiar-workflows`. Names must match exactly.
///
/// The SharedState is captured by cloning the Arc for each activity.
/// This is cheap (just increments refcount) and allows concurrent execution.
pub fn register_fates_activities(worker: &mut Worker, state: SharedState) {
    tracing::info!("Registering Fates activities...");

    // Gate - Classification and Routing
    let s = state.clone();
    worker.register_activity("FatesGate", move |_ctx, input| {
        let state = s.clone();
        async move { 
            fates_gate_activity(state, input)
                .await
                .map_err(ActivityError::from)
        }
    });

    // Morta - Content Segmentation
    let s = state.clone();
    worker.register_activity("FatesMorta", move |_ctx, input| {
        let state = s.clone();
        async move { 
            fates_morta_activity(state, input)
                .await
                .map_err(ActivityError::from)
        }
    });

    // Decima - Entity Extraction
    let s = state.clone();
    worker.register_activity("FatesDecima", move |_ctx, input| {
        let state = s.clone();
        async move { 
            fates_decima_activity(state, input)
                .await
                .map_err(ActivityError::from)
        }
    });

    // Nona - Response Generation
    let s = state.clone();
    worker.register_activity("FatesNona", move |_ctx, input| {
        let state = s.clone();
        async move { 
            fates_nona_activity(state, input)
                .await
                .map_err(ActivityError::from)
        }
    });

    // Pipeline - Full pipeline as single activity
    let s = state.clone();
    worker.register_activity("FatesPipeline", move |_ctx, input| {
        let state = s.clone();
        async move { 
            fates_pipeline_activity(state, input)
                .await
                .map_err(ActivityError::from)
        }
    });

    tracing::info!("Registered 5 Fates activities: Gate, Morta, Decima, Nona, Pipeline");
}

