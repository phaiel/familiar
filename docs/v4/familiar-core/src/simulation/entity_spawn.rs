//! Entity Spawn Types
//!
//! Defines the possible entities that can be spawned from the Heddle classification process.

use crate::{Moment, Intent, Thread, Bond, Pulse, Motif, Filament, Focus};

/// An entity spawned from the Heddle collapse process.
/// Wraps all possible entity types in a single enum.
#[derive(Debug, Clone)]
pub enum EntitySpawn {
    Moment(Moment),
    Intent(Intent),
    Thread(Thread),
    Bond(Bond),
    Pulse(Pulse),
    Motif(Motif),
    Filament(Filament),
    Focus(Focus),
}

/// The Heddle System Manifest
pub fn get_heddle_manifest() -> crate::config::SystemManifest {
    crate::config::SystemManifest {
        id: "sys_ingest_heddle".to_string(),
        domain: crate::config::SystemDomain::Ingestion,
        description: "The Heddle: Context-aware segmentation and classification engine.".to_string(),
        reads: vec![],
        writes: vec![
            "Moment".to_string(), "Intent".to_string(), "Thread".to_string(), "Bond".to_string(),
            "Pulse".to_string(), "Motif".to_string(), "Filament".to_string(), "Focus".to_string(),
        ],
        trigger: crate::config::SystemTrigger::Event("raw_input_received".to_string()),
    }
}

/// Default collapse threshold (70% confidence)
pub const DEFAULT_COLLAPSE_THRESHOLD: f64 = 0.7;
