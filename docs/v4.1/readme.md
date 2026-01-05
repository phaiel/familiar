1. 🎯 The Intent & Philosophy
Familiar is not a database; it is a simulation of a family's shared mind.
At its root, Familiar is a Family Tracking and Memory Synthesis Application. While traditional apps log events (calendar dates, photos, tasks), Familiar logs resonance (emotional weight, recurring patterns, narrative arcs).
1.1 The Problem: Data Amnesia
Families generate terabytes of data (photos, texts, calendar events) but lack a coherent narrative of change. We have the "what" (events), but lose the "who" (identity evolution) and the "why" (relationships).
1.2 The Solution: Simulating Perception
We do not build a static inventory of objects; we simulate a dynamic Cognitive Manifold.
Subjectivity: A "Pacifier" is not an inventory item; it is a Thread with a specific emotional weight and relationship to a child.
Resonance: We measure how memories vibrate across time. A moment from 3 years ago can "resonate" with a moment today based on shared physics.
Meaning: We map all data into a 3D VAE Space (Valence, Arousal, Epistemic) to calculate distance and gravity between concepts.
2. 🏗️ Architectural Canon
2.1 The Library Strategy: The "God Crate"
We adopt a Code-to-Schema pipeline.
familiar-schemas: A single Rust library crate that defines every primitive, component, entity, and system in the infrastructure.
Immutability: This crate is versioned and published. Once a version is live, the physics of that universe are fixed until the next migration.
Bidirectional Validation: We use schemars to generate JSON Schemas from the Rust code, ensuring external consumers (Frontend/AI) strictly adhere to internal memory layouts.
2.2 The Data Architecture: The 3-Table Model
State is separated by mutability to support the Growing Block Universe.
entities: The immutable registry of UUIDs and Types.
entity_versions: The append-only log of semantic content (Names, text, descriptions).
entity_physics: The hot, mutable table of 3D coordinates and energy levels.
3. ⚙️ System Manifest: The Canon of Logic
Logic is not just code; it is a defined artifact. Every System in the codebase must have a corresponding Rust struct implementing SystemManifest.
3.1 System Definition Schema
code
Rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SystemManifest {
    pub id: String,              // Unique key (e.g., "sys_physics_gravity")
    pub domain: SystemDomain,    // Ingestion, Physics, Analysis, Maintenance
    pub description: String,     // Human readable doc
    pub reads: Vec<String>,      // Components required for READ
    pub writes: Vec<String>,     // Components required for WRITE
    pub trigger: SystemTrigger,  // Event("topic"), Schedule("cron"), OnDemand
}
3.2 The Core System Catalog
sys_ingest_weave (Ingestion): Classifies raw text into Moment or Pulse drafts using LLM.
sys_physics_time_evolution (Physics): Applies velocity to position for UniversalPhysicsState (
x
=
x
+
v
Δ
t
x=x+vΔt
).
sys_consolidate_filament (Consolidation): Aggregates clusters of Pulse entities into a Filament.
4. ⚛️ The Physics Canon: 3D VAE Field Theory
To ensure performant scaling (
O
(
N
log
⁡
N
)
O(NlogN)
 or better), we utilize a quantized 3D coordinate system.
4.1 The 3D Coordinate System
Every entity exists at a coordinate [i64; 3] within a quantized grid.
Valence (V): Emotional Polarity. (Negative ↔ Positive).
Arousal (A): Energy/Activation. (Calm ↔ Excited).
Epistemic (E): Knowledge Certainty. (Abstract/Vague ↔ Concrete/Factual).
4.2 Universal Physics State
This component is attached to every entity that participates in the simulation.
code
Rust
#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct UniversalPhysicsState {
    /// The position in 3D VAE space
    pub position: [i64; 3],
    
    /// The rate of change in position (Cognitive Drift)
    pub velocity: [i64; 3],
    
    /// The "Mass" or significance of the entity. 
    pub mass: f64, // 0.0 to 1.0
    
    /// The current Temperature/Energy. 
    pub energy: f64, // 0.0 to 1.0
}
5. 🧬 The Ontology: Comprehensive Entity Definition
We adopt the Symmetric Seven model, plus the Binding (interaction).
5.1 Domain Mapping
Domain  Particle (Log/Action)   Wave (Pattern/Theme)
External (World)    Moment (History)    Motif (Pattern)
Internal (Self) Pulse (State Log)   Filament (Identity)
Intentional (Future)    Intent (Task)   Focus (Goal/Theme)
Relational (Connection) Binding (Transient) Bond (Persistent)
Object (Noun)   Thread  Thread
5.2 Detailed Entity Specifications
A. The Pulse (Internal Log)
A discrete snapshot of internal state at t=0. It is the raw material for Identity.
Data: InternalStateType (Emotional Shift, Realization), Intensity, VAE_Coordinates.
B. The Intent (Future Action)
A specific, bounded future action with potential energy.
Data: TargetDate, EnergyCost, CompletionCriteria.
Physics: Behave like particles with potential energy.
Lifecycle: Must be transmuted (Complete 
→
→
 Moment, Delete 
→
→
 Entropy).
C. The Focus (Active Theme)
A user-declared thematic goal (e.g., "Health 2025").
Data: ThemeName, Duration, FieldStrength.
Physics: Acts as a Quantum Field Generator. It warps the VAE manifold to make related Intents easier to execute.
D. The Binding (Transient Interaction)
A high-energy, short-lived connection between a Thread and an Event.
Data: Role (Active/Passive), Intensity.
E. Standard Entities
Moment: Objective historical event.
Motif: Recurring pattern of Moments.
Filament: Recurring pattern of Pulses/Self.
Thread: Person, Place, or Concept.
Bond: Persistent spring-damper connection between Threads.
6. 🧬 Design Canon: Atomic Composition
We reject the "God Struct". We adopt Atomic Component Composition.
The Pattern
Entities are defined by composing small, reusable, schema-defined structs.
code
Rust
// 1. Atomic Components (Reusable)
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct Identity { name: String, tenant_id: UUID }

// 2. Entity Definition (Composition)
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct Pulse {
    pub id: UUID,
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(flatten)]
    pub physics: UniversalPhysicsState,
    
    pub state_type: InternalStateType,
}
7. 🛡️ Implementation Law (The Rules)
Schema-First is Non-Negotiable: No code writes to the database unless a struct for that data exists in familiar-schemas.
No Magic Numbers: Physics constants (decay rates, field strengths) must be loaded from a config file or the Schema library.
Agents Observe, They Don't Act: AI Agents (LLMs) output Metadata. A deterministic Rust service translates that Metadata into Physics.
The Tenant Boundary: Every Entity UUID is namespaced or accompanied by a TenantID.
Logic as Data: All Systems must have a defined SystemManifest in the schema library.
8. 🚦 Next Steps (v4.1 Roadmap)
Phase 1: Schema Foundation (Immediate)

Initialize familiar-schemas Rust crate.

Implement Primitives: UUID, NormalizedFloat (0.0-1.0), Timestamp.

Implement Physics: UniversalPhysicsState (3D VAE), QuantumState.

Implement Entities: Pulse, Intent, Focus, Binding.
Phase 2: System Manifests

Create src/systems/manifest.rs and catalog.rs.

Define the core ingestion and physics systems as data.
Phase 3: The Logic

Implement the logic that transmutes an Intent into a Moment.

Implement the logic that collapses a Focus into a Filament.