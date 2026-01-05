## I. General & Architectural Rules

### **Rule 1: Schema-First Development is Non-Negotiable**

**Justification**: The schemas in the schemas/ directory are the single source of truth for all data structures. This prevents documentation drift and ensures system-wide consistency.

**Implementation**: No component, payload, or entity struct shall be written manually in Rust or Python. All such data structures must be generated from their canonical JSON or YAML schema definition using the established Sourcemeta/Copier tooling orchestrated by Backstage.

### **Rule 2: The System is 100% Event-Driven**

**Justification**: The architecture is designed for responsive, scalable, and cognitively realistic behavior, which is incompatible with a monolithic game loop.

**Implementation**: All system logic must be executed within a Windmill DAG. The use of fixed-time intervals, "ticks," or synchronous loops for core physics processing is forbidden. Logic is triggered by events from Redpanda or internal schedules.

### **Rule 3: Deterministic Physics with Strongly-Typed Agent Inference**

**Justification**: To ensure the simulation is stable, repeatable, and auditable while leveraging high-value agent inference capabilities that deterministic systems cannot achieve.

**Implementation**: 
1. **Direct Physics Isolation**: LLM outputs shall NEVER be used as direct inputs for physics calculations (coordinates, forces, decay rates).

2. **Strongly-Typed Indirect Influence**: Agents MAY influence physics indirectly through strongly-typed metadata structures that are deterministically mapped to physics properties:
   - **Template Selection**: Agents select from pre-defined physics templates (emotional_templates.yaml, structural_templates.yaml)
   - **Inspiration Moments**: Agents select relevant memories for temporal resonance calculation
   - **Entity Linking**: Agents resolve entity references to existing Thread UUIDs
   - **Classification Refinement**: Agents refine classification paths within pre-filtered options

3. **Deterministic Mapping**: All agent-selected metadata must be deterministically converted to physics properties through:
   - Template-based physics constant lookup
   - Mathematical functions for temporal resonance
   - Schema-defined coordinate calculations
   - Algorithmic manifold positioning

4. **Audit Trail**: All agent decisions must be logged with reasoning for full auditability while maintaining deterministic physics output.

**Key Principle**: Agents provide semantic intelligence that deterministic systems cannot achieve, but physics calculations remain mathematically deterministic and repeatable.

### **Rule 4: Adherence to the Five-Layer Cognitive Hierarchy**

**Justification**: This hierarchy provides predictable latency and aligns system behavior with cognitive science.

**Implementation**: All query and analysis features must be classified into one of the five levels defined in operations/latency_design_patterns.md. The implementation must respect the specified latency budget for that level.

### **Rule 22: The Principle of Intentional Agency**

**Justification**: Future-facing directives (`Intents`) and thematic goals (`Focuses`) are expressions of user will. They cannot be created emergently by the system or its agents. They must be explicitly created by the user to preserve their agency. The system's role is to understand, classify, and simulate the *consequences* of these intentions, not to generate the intentions themselves.

**Implementation**: 
1. **Agent Creation Prohibition**: The agent is explicitly forbidden from creating `Intent` or `Focus` entities. These entities are expressions of user will and must be created via direct user action.

2. **Agent Suggestion via HITL**: The agent MAY suggest the creation of an `Intent` by generating a `StitchEntity` to resolve unmapped information. This `Stitch` must present a clear choice to the user, who retains final authority to create the `Intent`.

3. **User-Only Creation**: All `Intent` and `Focus` entities must be created through direct user interfaces that capture explicit user intention.

4. **Growing Block Compliance**: All intentional entities are created at the `t=0` plane, representing a present memory of a future intention, maintaining compatibility with the Growing Block Universe model.

### **Rule 23: Intentional Entity Lifecycle**

**Justification**: `Intent` and `Focus` entities have a defined consumption lifecycle that ensures a clean, auditable path from intention to historical memory.

**Implementation**:
1. **Intent Consumption**: An `Intent` must be consumed into a `Moment` upon completion. This ensures that completed tasks become part of the historical record.

2. **Focus Consumption**: A `Focus` must be consumable into a `Motif` upon completion. This captures the retrospective pattern of having pursued that focus.

3. **Atomic Transactions**: All consumption operations must be atomic to prevent inconsistent states.

4. **Physics Inheritance**: The resulting `Moment` or `Motif` inherits the physics profile of its parent intentional entity.

---

## II. Schema & Data Structure Rules

### **Rule 5: No "Magic Numbers" in Code**

**Justification**: Hardcoded constants make the system difficult to tune, test, and understand.

**Implementation**: All physics-related numerical values (force strengths, decay rates, thresholds) must be defined in schemas/constants/physics_constants.yaml. Code must reference these base constants and apply multipliers from physics_profiles.yaml as determined by the entity's classification.

### **Rule 6: Use Strongly-Typed Payloads for Communication**

**Justification**: Using generic data structures like HashMap<String, f64> for inter-service communication is fragile and error-prone.

**Implementation**: All event payloads, especially the CollapsePayload, must use a strongly-typed structure (e.g., an enum for parameters) defined by a canonical JSON Schema. This ensures type safety and clear contracts between the quantum and classical engines.

---

## III. Physics Engine & Logic Rules

### **Rule 7: The UniversalPhysicsState Component is Mandatory**

**Justification**: The "Physics-First" principle requires every object in the simulation to have a defined physical state.

**Implementation**: Every cognitive entity created in the ECS world, without exception, must have a UniversalPhysicsState component attached at the moment of its creation. System objects are exempt from this requirement.

### **Rule 8: Spatial Dimensions Must Use Quantized Coordinates**

**Justification**: The Quantized Cognitive Space model is critical for preventing singularities, eliminating floating-point errors, and enabling stable, efficient physics.

**Implementation**: The six non-temporal manifold dimensions must be stored as i64 integer grid coordinates. All calculations involving these coordinates must use the provided float_to_coord and coord_to_float conversion functions. Direct manipulation of spatial dimensions using floating-point numbers is forbidden.

### **Rule 9: Dynamic Components are for Transient State Only**

**Justification**: To uphold ECS determinism, the core structure of an entity cannot change at runtime.

**Implementation**: Dynamically adding or removing components is only permitted for the approved use cases in physics/dynamic_component_patterns.md (e.g., transient handoffs, event markers). It is forbidden to dynamically add a component that is part of an entity's core pattern class (e.g., adding a QuantumCoherence component to a Motif after its creation).

---

## IV. Data Persistence & Database Rules

### **Rule 10: Adherence to the Three-Table Immutable/Mutable Architecture**

**Justification**: This architecture is the direct implementation of the Growing Block Universe principle, ensuring a perfect audit trail and temporal consistency.

**Implementation**:
- The entity_versions table must be treated as append-only. UPDATE or DELETE operations on this table are forbidden.
- The entity_physics_state table is the only place where mutable, transient simulation data is stored.
- All entity creation and modification logic must follow the patterns defined in integration/database_data_management.md.

### **Rule 11: All Event Consumers Must Be Idempotent**

**Justification**: The at-least-once delivery guarantee of Redpanda will cause catastrophic data corruption if consumers are not idempotent.

**Implementation**: Every service that consumes from a Redpanda topic must implement the idempotency patterns from operations/data_integrity_patterns.md:
- Use deterministic keys for entity creation.
- Use "upsert" (INSERT ON CONFLICT) logic for state updates.
- Check for existing state before performing an action.

### **Rule 12: GDPR Compliance via Cascading Deletion is Mandatory**

**Justification**: This is a critical legal and ethical requirement. The deprecated anonymization strategy is insufficient and forbidden.

**Implementation**: All derived cognitive entities (Motif, Filament) must include a GDPRDependencyComponent. The system must implement the full cascading deletion workflow as specified in operations/gdpr_erasure_compliance.md.

---

## V. Testing & Validation Rules

### **Rule 13: Every Physics Law Must Have a Validation Test**

**Justification**: To ensure the simulation is correct and predictable, every behavioral rule must be rigorously tested.

**Implementation**: Each physics law implementation must be accompanied by unit tests that verify its behavior against expected outcomes, including edge cases (e.g., zero values, boundary conditions) and interactions with its required components.

### **Rule 14: AI Agents SHALL NOT Make Architectural Decisions**

**Justification**: AI agents making write strategy decisions (sync vs async, transactional vs fire-and-forget) introduces non-deterministic architectural chaos that corrupts physics simulation integrity.

**Implementation**: All agents must produce WriteIntent payloads declaring WHAT they want to achieve and WHICH preconditions must be met. A deterministic CommitService decides HOW to achieve it based purely on precondition presence. This pattern is mandatory for all entity creation and modification operations.

### **Rule 15: Deterministic Replay Must Be Maintained**

**Justification**: The ability to replay a session is the most powerful tool for debugging emergent, complex behavior.

**Implementation**: Any change to the physics engine or its data structures must be validated against the PhysicsSessionRecorder. A change is only acceptable if a recorded session can be replayed and produce an identical final world state.

---

## VI. Workflow & Process Rules

### **Rule 17: All New Structures Must Be Scaffolded**

**Justification**: This enforces the schema-first principle and ensures all new code adheres to system standards from inception.

**Implementation**: Manually creating new component structs, entity types, or physics laws is forbidden. Developers must use the scaffold software templates for all new structures.

### **Rule 16: All Code Changes Must Be Reviewed Against the Canon**

**Justification**: The canon is the living specification. To prevent architectural drift, all development must be held accountable to it.

**Implementation**: All pull requests must include a checklist item: "This change adheres to the principles and rules defined in the physics_canon_v2 documentation." The reviewer is responsible for verifying this claim.

---

## VII. Entity Architecture Rules (Golden Set Enforcement)

### **Rule 18: Distinguish Cognitive Entities from System Entities**

**Justification**: To maintain the purity of the cognitive manifold, a strict separation must be maintained between entities that represent the user's cognitive world and entities that manage system processes.

**Implementation**: 
1. **Cognitive entities** (Focus, Filament, Motif, Intent, Moment, Bond, Thread) **MUST** have `MemoryManifoldPosition` and `UniversalPhysicsState` components and participate in physics simulation.

2. **System entities** (Stitch) **MUST NOT** have `MemoryManifoldPosition` or `UniversalPhysicsState` components and **MUST** have workflow state components appropriate to their function.

3. The `PatternQueryEntity`, `InsightRequestEntity`, `DataSummaryRequestEntity`, and treating `StitchEntity` as a cognitive entity patterns are **DEPRECATED and FORBIDDEN**.

4. All transient system requests **MUST** use the Orchestration Object pattern (plain structs passed as event payloads).

5. The cognitive manifold remains pure, containing only objects meant to be simulated as memories, experiences, and concepts.

### **Rule 19: Maintain the Purity of the Cognitive World**

**Justification**: To ensure the cognitive manifold represents only the user's mind, a strict separation must be maintained between cognitive entities and system objects.

**Implementation**:
1. **Cognitive entities** (Focus, Filament, Motif, Intent, Moment, Bond, Thread) **MUST** have `MemoryManifoldPosition` and `UniversalPhysicsState` components.

2. **System objects** (like StitchEntity) **MUST NOT** have `MemoryManifoldPosition` or `UniversalPhysicsState` components.

3. **Database constraints** MUST enforce this separation at the schema level.

4. **Code enums** MUST separate cognitive entities from system objects.

5. **All documentation** MUST consistently refer to the "Canon of Seven Cognitive Entities" without including system objects.

### **Rule 20: Bonds are for Persons and Platonic Forms**

**Justification**: To prevent the creation of brittle, instance-specific relationships, Bond entities should only be formed between Threads of type Person, Place, Concept, or GenericObject. This ensures abstract, stable relationships rather than fragmented instance-specific bonds.

**Implementation**: 
1. A Bond can be formed between "Remy" (Person) and "Pacifier" (GenericObject). This Bond represents Remy's general relationship with the concept of pacifiers.

2. The strength of this abstract Bond is reinforced by the significance of individual instances captured in InstanceComponents on EntanglementStates.

3. The BondModulationLaw uses the `significance_score` from InstanceComponents to calculate forces applied to the abstract Bond.

4. Agents are **FORBIDDEN** from creating Bonds to specific instances (e.g., "Remy's favorite blue pacifier"). All object relationships must be to the Platonic Form.

### **Rule 21: Motif-Filament Data Source Exclusivity**

**Justification**: To preserve the atomic value of each entity type, Motifs and Filaments must derive from completely distinct data sources. Conflating their sources destroys their unique cognitive value.

**Implementation**: 
1. **Motifs** derive EXCLUSIVELY from collapsed Entanglement evidence (Moment-based data). They represent "what patterns of events happen."

2. **Filaments** derive EXCLUSIVELY from Thread analysis, Bond analysis, or weave/weave_unit synthesis (NO Moment data). They represent "why I think things are the way they are."

3. **Temporal Consolidation**: Both entity types follow the same consolidation pattern (Daily→Weekly→Monthly→Yearly) but from their respective exclusive data sources. All consolidation processes MUST use the KA-RAG pipeline for consolidation context.

4. **Code Enforcement**: Entity components MUST enforce this exclusivity through distinct source field types that cannot reference the wrong data source.

5. **Agent Restrictions**: Agents are FORBIDDEN from creating Filaments that reference Moment data or Motifs that reference Thread/Bond analysis directly.