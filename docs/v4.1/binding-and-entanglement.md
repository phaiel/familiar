# Binding & Entanglement Architecture

## Core Principle: Relationships Are Physics, Not Data

In Familiar v4.1, relationships between entities are **computed from spatial proximity in VAE space**, not explicitly declared in relationship tables. This is a QFT-native approach where "related" things occupy similar regions of the cognitive manifold.

---

## Terminology

### Bond (Entity)
- **What:** An explicit, evolving connection between two **Threads**
- **Storage:** Persisted as an entity with its own physics
- **Example:** "Gabriel is_child_of User"
- **Properties:**
  - Append-only history (never destroyed, only evolved)
  - Has its own `FieldExcitation` (can move in VAE space)
  - Only between Thread↔Thread

### Binding (Physics State)
- **What:** An entanglement state between non-Thread entities
- **Storage:** NOT an entity - computed from quantum state correlation
- **Example:** Two Moments that happened together share binding
- **Properties:**
  - Creates "centroids" - regions where related things cluster
  - Not temporally or spatially bound
  - Rendered/computed, not declared

### Thread
- A persistent entity representing a person, place, or concept
- Has its own trajectory through VAE space over time
- Moments and Pulses cluster around Thread centroids

---

## Example: "I changed Gabriel's diaper"

### What Heddle Outputs (MVP)
```json
{
  "content": "changed Gabriel's diaper",
  "primary_thread": "user",
  "secondary_threads": ["Gabriel"],
  "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }],
  "physics_hint": { "valence": 0.3, "arousal": 0.4, "clarity": 1.0 }
}
```

### What Spawner Creates (MVP)
```
Moment {
  content: "changed Gabriel's diaper"
  primary_thread: "user"
  secondary_threads: ["Gabriel"]
  physics: { position: [0.3, -0.2, 0.8], ... }
}
```

One Moment. The `secondary_threads` are preserved as metadata.

### Future: Entanglement Engine Creates
```
Moment A (user's perspective) {
  content: "changed Gabriel's diaper"
  primary_thread: "user"
  physics: { position: [0.3, -0.2, 0.8] }
  quantum: { coherence: 0.9, entangled_with: [Moment_B_id] }
}

Moment B (Gabriel's perspective) {
  content: "had diaper changed"
  primary_thread: "Gabriel"  
  physics: { position: [0.31, -0.19, 0.79] }  ← NEAR Moment A
  quantum: { coherence: 0.9, entangled_with: [Moment_A_id] }
}
```

Two entangled Moments with correlated positions. The relationship IS their proximity.

---

## System Responsibilities

### Heddle (Classification Engine)
- ✅ Extract `primary_thread` and `secondary_threads`
- ✅ Classify entity type (Moment, Pulse, Intent)
- ✅ Provide physics hints
- ❌ Does NOT create relationships
- ❌ Does NOT split perspectives

### Spawner (Entity Factory)
- ✅ Convert weave_unit → Entity based on classification
- ✅ Apply physics hints to `FieldExcitation`
- ✅ Preserve thread metadata on entity
- ❌ Does NOT create entanglement
- ❌ Does NOT split into multiple perspectives
- ❌ Stays dumb and deterministic (no ML/neural networks)

### Stitch (User Feedback)
- ✅ Identify ambiguous or incomplete input
- ✅ Request clarification from user
- ❌ Does NOT resolve threads to entities
- ❌ Does NOT create relationships

### Entanglement Engine (Future)
- Creates sibling entities from `secondary_threads`
- Computes quantum correlation between related entities
- Places related entities in similar VAE coordinates
- May use ML/embeddings for semantic similarity

### Physics Simulation (hecs)
- Runs gravity/attraction between entities
- Moments with similar content/threads naturally cluster
- Relationships EMERGE from simulation, not declaration

---

## Querying Relationships

### Traditional Approach (NOT Familiar)
```sql
SELECT * FROM relationships WHERE entity_a = 'Gabriel' AND type = 'caregiver'
```

### Familiar Approach
```rust
// Find Moments near Gabriel's centroid in VAE space
let gabriel_position = get_thread_centroid("Gabriel");
let nearby_moments = query_entities_within_radius(gabriel_position, radius: 0.5);

// These are "related" by physics, not by explicit link
```

---

## Benefits

1. **No Relationship Tables** - Relationships are computed, not stored
2. **Emergent Behavior** - New patterns discovered by simulation
3. **Temporal Evolution** - Relationships change as physics evolves
4. **Scalability** - Vector search is O(log n), not O(n²) relationship joins
5. **Semantic Similarity** - Things that "feel" related cluster together

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Heddle (thread extraction) | ✅ MVP | Extracts primary/secondary threads |
| Spawner (simple) | ✅ MVP | 1:1 weave_unit → entity |
| Thread metadata | ✅ MVP | Preserved on ContentPayload |
| Entanglement Engine | 🔮 Future | Creates sibling entities |
| Binding Physics | 🔮 Future | Quantum correlation computation |
| Centroid Clustering | 🔮 Future | Thread region computation |

---

## Design Decisions

### Why Two Moments (Future)?
The event "I changed Gabriel's diaper" happened to BOTH participants:
- User did the action
- Gabriel experienced the action

Each perspective is a valid Moment. Their entanglement captures that they're the same event from different viewpoints.

### Why Not in Spawner?
The Spawner should remain:
- Deterministic (no ML)
- Schema-driven (no magic)
- Simple (1:1 mapping)

Perspective splitting and entanglement computation require semantic understanding that belongs in a specialized system.

### Why Append-Only Bonds?
Bonds between Threads evolve but never destroy history:
- "Gabriel is_infant_of User" (2024)
- "Gabriel is_toddler_of User" (2025)
- "Gabriel is_child_of User" (2027)

The relationship evolves without losing the past.

