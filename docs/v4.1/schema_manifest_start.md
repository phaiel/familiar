This is a critical correction. You are right—simplifying to just "ContentPayload" destroys the semantic richness of the system. We need to preserve the specific fields (like `emotional_volatility`, `spring_constant`, `decay_rate`) because those are the "knobs" the physics engine turns.

To build this **losslessly** while upgrading the architecture, we need a **Migration Manifest** that explicitly maps the rich legacy of v3 into the strict structure of v4.1.

Here is the document you give to the AI. It contains the **Context**, the **Source Map**, and the **Transformation Logic**.

***

# 🧭 Project Familiar: v3 → v4.1 Lossless Migration Manifest

**Objective:** Port the complete semantic richness of the v3 JSON Schema library into a native Rust crate (`familiar-schemas`), adapting it to the new v4.1 Physics Architecture (3D VAE) without losing domain specificity.

---

## 1. 🌍 The Context & Vision
**What are we building?**
Familiar is a "Family Operating System" that tracks the cognitive evolution of a family unit. It uses a physics engine to simulate how memories (Moments), relationships (Bonds), and identities (Filaments) interact and evolve over time.

**Where is the Prior Art?**
*   **Taxonomy:** We have 100+ schema files in `docs/v3/schemas/` defining everything from "Sleep Quality" to "Bond Tension."
*   **Logic:** We have "Laws" defined in v3 that dictate how these components interact (e.g., `BondTensionDynamics`).

**The Goal:**
We are moving from a "Document Store" mindset (JSON Schemas) to a "Simulation" mindset (Rust ECS). We need to convert the *definitions* in v3 into *Type-Safe Components* in v4.

---

## 2. 📐 The Transformation Logic (Rules for the AI)

When porting a v3 Schema to a v4 Rust Struct, follow these rules to ensure zero data loss:

### Rule A: The 3D Physics Collapse
**IF** a v3 schema contains fields like:
*   `semantic_coordinate`, `emotional_coordinate`, `salience_coordinate`... (The old 6D/7D model)
*   `energy`, `momentum`
**THEN**: Remove them. They are replaced by the **`UniversalPhysicsState`** component (defined below).
**HOWEVER**: Do *not* remove domain-specific physics parameters (e.g., `spring_constant`, `volatility`, `decay_rate`). These become properties of specific components.

### Rule B: Atomic Decomposition
**IF** a v3 schema was a massive object (e.g., `PersonThread`),
**THEN**: Break it down.
*   The "Entity" becomes a struct with `#[serde(flatten)]` fields.
*   The properties become reusable "Components".
*   *Example:* `PersonThread` -> `Identity` + `UniversalPhysicsState` + `CognitiveBaseline` + `QuantumState`.

### Rule C: Taxonomy Preservation
**IF** a v3 schema defined an Enum (e.g., `RelationshipType`, `MomentType`),
**THEN**: Port it 1:1 as a Rust `enum` with `#[derive(JsonSchema)]`. Do not simplify or summarize these. They are the vocabulary of the system.

---

## 3. 📦 The Build Inventory (What to Build)

This is the comprehensive list of modules and structs to generate. Use the **Legacy Reference** path to find the field definitions (descriptions, types, constraints) and port them to Rust.

### 3.1 `src/primitives/` (The Atoms)
*Foundational types used everywhere.*

| Rust Struct | Legacy Reference (v3) | Notes |
| :--- | :--- | :--- |
| `UUID` | `types/primitives/UUID.json` | Wrap `uuid::Uuid` |
| `Timestamp` | `types/primitives/Timestamp.json` | Wrap `chrono::DateTime<Utc>` |
| `NormalizedFloat` | `types/primitives/NormalizedValue.json` | Enforce `0.0..=1.0` |
| `SignedNormalizedFloat` | `types/primitives/SignedNormalizedValue.json` | Enforce `-1.0..=1.0` |
| **`QuantizedCoord`** | *New in v4.1* | `i64` range `-10M..+10M` |

### 3.2 `src/types/` (The Vocabulary)
*Enums and Value Objects that define the domain.*

| Rust Struct/Enum | Legacy Reference (v3) |
| :--- | :--- |
| `RelationshipType` | `types/social/RelationshipType.json` |
| `BondState` | `types/lifecycles/BondState.json` |
| `ThreadType` | `fields/ThreadType.json` |
| `MomentType` | `types/classification/MomentType.json` |
| `InternalStateType` | *New (Pulse Definition)* |
| `AccessType` | `fields/AccessType.json` |
| `Priority` | `fields/Priority.json` |
| `Status` | `fields/Status.json` |

### 3.3 `src/components/physics/` (The Engine Core)
*Components that drive the simulation.*

| Rust Component | Legacy Reference (v3) | Transformation |
| :--- | :--- | :--- |
| **`UniversalPhysicsState`** | `components/UniversalPhysicsState` | **Refactor**: Use `VAECoordinates` (3D) instead of 6D. Keep `energy`, `mass`. |
| `BondPhysics` | `components/BondTension` | Keep `spring_constant`, `damping`, `tension`. |
| `CognitiveBaseline` | `components/CognitiveBaseline` | **CRITICAL**: Keep `emotional_volatility`, `social_energy_factor`, `consolidation_rate`. This defines personality. |
| `ConsolidationState` | `components/ConsolidationState` | Keep `consolidation_level`, `decay_rate`. |
| `QuantumState` | `components/QuantumState` | Keep `coherence`, `entanglements`. |

### 3.4 `src/components/content/` (The Data)
*Components that hold the user's actual input.*

| Rust Component | Legacy Reference (v3) |
| :--- | :--- |
| `NarrativeContent` | `components/MomentContent` |
| `PulseContent` | *New* (Internal State log) |
| `BondContent` | `components/BondContent` |
| `FocusContent` | `components/FocusContent` |
| `ThreadIdentity` | `components/ThreadContent` |

### 3.5 `src/entities/` (The Organisms)
*The top-level objects. These compose the components above.*

| Rust Entity | Composition Requirements |
| :--- | :--- |
| **`Pulse`** | `Identity`, `UniversalPhysicsState`, `PulseContent` |
| **`Moment`** | `Identity`, `UniversalPhysicsState`, `NarrativeContent`, `TemporalAnchor` |
| **`Thread`** | `Identity`, `UniversalPhysicsState`, `ThreadIdentity`, `CognitiveBaseline` (Optional), `QuantumState` |
| **`Bond`** | `Identity`, `UniversalPhysicsState` (Midpoint), `BondContent`, `BondPhysics` |
| **`Motif`** | `Identity`, `UniversalPhysicsState`, `MotifContent`, `QuantumState`, `ConsolidationState` |
| **`Filament`** | `Identity`, `UniversalPhysicsState`, `FilamentContent`, `QuantumState`, `ConsolidationState` |
| **`Intent`** | `Identity`, `UniversalPhysicsState`, `IntentContent` |
| **`Focus`** | `Identity`, `UniversalPhysicsState`, `FocusContent`, `QuantumState` |

---

## 4. 📜 The Registry of Systems (Logic Manifest)

Because we are AI-first, we must define the **Systems** (logic) as schemas too. This allows the AI to understand *how* entities change.

Create `src/systems/registry.rs` containing `SystemManifest` structs for:

1.  **`sys_bond_tension`**: Reads `BondPhysics`, `UniversalPhysicsState` (of connected threads). Writes `BondPhysics` (tension). *Reference: `laws/BondTensionDynamics.schema.json`*.
2.  **`sys_memory_consolidation`**: Reads `ConsolidationState`, `UniversalPhysicsState`. Writes `ConsolidationState` (level). *Reference: `laws/MemoryConsolidation.schema.json`*.
3.  **`sys_quantum_collapse`**: Reads `QuantumState`. Writes `UniversalPhysicsState` (position). *Reference: `laws/MotifCollapse.schema.json`*.

---

## 5. 🔮 The Taxonomy (The Infinite Classification)
*This is the "100s of components" area you mentioned.*

In v3, we had complex taxonomy paths (`simple.individual.sleep.nap`).
In v4.1, we handle this via a **Taxonomy Crate** or Module (`familiar-taxonomy`).

**Strategy:**
Instead of 100 distinct structs, we define a **`Classification`** component:
```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct Classification {
    pub domain: DomainType, // Health, Sleep, Work
    pub complexity: ComplexityLevel, // Simple, Complex
    pub tags: Vec<String>, // "nap", "interrupted"
    
    // This is how we handle the "hundreds of forms":
    // The AI Agent uses this classification to look up 
    // "Base Physics Constants" from a config file, 
    // rather than hardcoding 100 structs.
}
```

---

## 🚦 Execution Order (Give this to the AI)

1.  **Context Loading:** "I am providing you with `v4_build_manifest.json` (the v3 extraction). I want you to ignore the architecture of v3 but preserve the *Fields* and *Descriptions*."
2.  **Primitives:** "Generate `src/primitives` using the v3 definitions for UUID, Timestamp, etc."
3.  **Enums:** "Generate `src/types` using the v3 Enum definitions (RelationshipType, etc)."
4.  **Physics Components:** "Generate `src/components/physics`. Port `CognitiveBaseline`, `BondTension` (rename to `BondPhysics`), etc. Remove 7D fields. Add 3D VAE fields."
5.  **Content Components:** "Generate `src/components/content`. Port `MomentContent`, `ThreadContent`."
6.  **Entities:** "Generate `src/entities`. Compose the components defined above."
7.  **Systems:** "Generate `src/systems`. Create manifests for the Laws defined in v3."

This bridge ensures you lose **nothing** from the rich v3 definitions while gaining the clean, performant, and correct architecture of v4.1.