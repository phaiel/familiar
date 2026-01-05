# Database, Caching, and Consistency Strategy

*Part of the [Familiar Cognitive Physics Engine Canon](../00_overview.html) - System Architecture*

## 1. Overview

This document defines the core data management strategy for the cognitive physics engine. It explains how the system achieves both high performance for real-time physics simulation and perfect, durable historical records. The strategy is built on three pillars: a **Three-Table Database Architecture**, a **High-Performance In-Memory Cache**, and an **Event-Driven Consistency Model**.

## 2. The Canonical Three-Table Database Architecture

The database is the ultimate source of truth and is designed to separate an entity's immutable history from its fluid, mutable present. This is a non-negotiable architectural rule.

1.  **`entities` Table:** A lightweight registry of all entity IDs, their types, and creation timestamps.
2.  **`entity_versions` Table:** An **immutable, append-only log**. This table stores the *definitional* or *semantic* content of an entity. When a `Motif`'s name changes, a new versioned row is inserted here. This guarantees a perfect audit trail of an entity's meaning over time.
3.  **`entity_physics_state` Table:** A **mutable** table holding the transient, calculated state of the simulation. This is where the fluid VAE coordinates, energy levels, and other physics-related values are stored. These values are updated frequently and in-place for maximum performance.

## 3. The In-Memory Octree: A High-Performance Physics Cache

The physics engine cannot query the database for every interaction—it would be far too slow. To achieve real-time performance, the engine maintains an **in-memory octree**.

-   **Purpose:** The octree is a spatial index built exclusively from the `entity_physics_state` data. Its sole purpose is to accelerate physics calculations (e.g., fast neighbor finding, Barnes-Hut simulations) by providing a near-instantaneous way to query the spatial relationships between entities.
-   **Ephemeral Nature:** The octree is **not** a persistent data structure. It is a performance-critical cache. It is rebuilt or updated as needed to reflect the current state of the physics simulation.
-   **Initial Hydration (Cold Start):** When a physics engine service starts, it performs a one-time, bulk `SELECT` from the `entity_physics_state` table to load the positions of all relevant entities and build the initial octree in RAM. This is an accepted, one-time operational cost required to enable subsequent high performance.

## 4. Event-Driven Eventual Consistency

The system must handle new entity creation without blocking user interaction or halting the physics simulation. This is achieved through an event-driven, CQRS-like pattern defined in the `write_intent_pattern.md`.

### The Write and Synchronization Flow

This flow illustrates how the database, cache, and services work together when a new `Moment` is created.

1.  **Fast Path Write (User-Facing):** An agent calls a GraphQL mutation like `createMoment`. The API service immediately performs two actions:
    *   Inserts a new record into the `entities` table.
    *   Inserts a version `1` record into the **immutable `entity_versions` table** with the core semantic content.
    *   **Crucially, it does NOT wait for any physics calculations.**

2.  **Immediate User Feedback:** The API returns a success message to the user instantly (e.g., "Moment Saved!"). The user perceives the operation as complete.

3.  **Asynchronous Event Publication:** Concurrently, the API service publishes a `physics.moment.process.v1` event to an event bus (e.g., Redpanda). This event contains the new `Moment`'s ID and the data needed to calculate its physics state.

4.  **Physics Engine Consumption:** The physics engine service, which is subscribed to the event bus, consumes this event.

5.  **Physics Calculation and State Update:** The physics engine:
    *   Performs the necessary calculations to determine the new `Moment`'s initial VAE coordinates, energy, etc.
    *   Writes these new values to the **mutable `entity_physics_state` table**.

6.  **Cache Update:** Immediately after writing to the database, the physics engine **updates its own in-memory octree**, inserting the new `Moment` into the live simulation.

### The Consistency Model

This architecture results in **eventual consistency**. There is a brief, expected delay between the user receiving confirmation (Step 2) and the `Moment` appearing in the live physics simulation (Step 6).

This is not a flaw, but a deliberate and critical design trade-off that provides:
-   **High Performance:** The live simulation is not blocked by new writes.
-   **Excellent User Experience:** The user is never blocked waiting for complex, back-end calculations.
-   **Durability:** The database remains the consistent source of truth.
-   **Scalability:** The write-handling API services and the physics engine services can be scaled independently. 