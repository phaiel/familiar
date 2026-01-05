# Emergent Structures: Observation, Dynamics, and History

This document outlines the architecture for higher-order cognitive structures (`Motifs`, `Filaments`), their dynamic evolution, and the system's ability to recall past states.

## 1. The Principle of Emergence

-   **Primary Entities (`Moments`/`Pulses`):** The fundamental building blocks representing single events.
-   **Emergent Structures (`Motifs`/`Filaments`):** Entities representing themes or patterns that connect multiple primary entities. A `Motif` is a recognized pattern *across* a set of `Moments`.

## 2. `Motifs` as Wave Functions

A `Motif` is not a single point. It is a **wave function**—a probability distribution over a set of constituent `Moments`. In the ECS, a `Motif` has a `Potentiality` component which stores the list of constituent `Moment` IDs and the parameters of its wave function (e.g., a center-of-mass and covariance matrix derived from its constituents' positions).

## 3. The "Collapse on Observation" Pattern

A `Motif`'s uncertain nature becomes definite only when observed in relation to a specific anchor `Moment`. This is a query-time operation, not a permanent state change. The octree is the **enabling technology** that makes this pattern performant by turning a slow linear scan into a near-instantaneous spatial lookup.

## 4. The Dual-Force Physics Engine (The Engine of Change)

The evolution of the cognitive space is driven by two distinct but interacting types of forces.

### 4.1. Local Field Forces (Bottom-Up Clustering)

-   **Mechanism:** Every `Moment` projects a small, local potential field around itself, causing them to drift towards each other and form organic clusters.
-   **The Octree's Role:** The octree is the fast, in-memory index that makes the field calculations efficient, typically using a **Barnes-Hut simulation** algorithm (`O(N log N)`).

### 4.2. Hierarchical Entanglement Forces (Top-Down Coherence)

-   **Mechanism:** A `Motif` establishes quantum-inspired **entanglement** with its constituent `Moments`, allowing it to exert a top-down, coherent force on all of them simultaneously.

## 5. Versioning and Dynamics: The Three-Table Model

The system's ability to handle both fluid dynamics and perfect historical recall is enabled by a non-negotiable database architecture that separates mutable state from immutable history.

### The Canonical Three-Table Architecture

1.  **`entities` Table:** A simple, stable registry of all entity IDs and their types.
2.  **`entity_versions` Table:** An **immutable, append-only** log. This table stores the *definitional* content of an entity. When the name of a `Motif` changes from "Anxious about Deadlines" to "Resolved Deadline Anxiety," a **new row is inserted** into this table with the new content and a new version timestamp.
3.  **`entity_physics_state` Table:** A **mutable** table holding the transient, calculated state of the simulation. This is where the fluid VAE coordinates (`x,y,z`) of a `Moment` or `Motif` are stored. These values can be updated frequently by the physics engine without creating new versions.

### The Role of the Octree (Corrected)

The octree is **not** a persistent or versioned data structure. It is a **high-performance, in-memory spatial index** built entirely from the data in the **mutable `entity_physics_state` table**. Its sole purpose is to accelerate the physics calculations (neighbor finding, Barnes-Hut simulation) for the "live" state of the universe. It is rebuilt or updated as needed to reflect the current physical reality.

### Querying the Past (Corrected)

This architecture makes temporal queries elegant and efficient.

-   **The Query:** A user asks, "How did I view the world last month?"
-   **The Mechanism:**
    1. The system queries the **`entity_versions` table** to find the version of the "Resolved Deadline Anxiety" `Motif` that was active "last month." This gives the historical *semantic content* (e.g., its name was "Anxious about Deadlines").
    2. To reconstruct the historical *physical view*, the system would need to access a separate event log of historical physics states (e.g., `classical_physics_events` or a snapshot table).
    3. The query joins the historical semantic data from `entity_versions` with the historical position data from the physics logs.

This model provides maximum performance for the live simulation (by mutating physics state in-place) while guaranteeing a perfect, auditable, and immutable history of an entity's meaning and evolution. 