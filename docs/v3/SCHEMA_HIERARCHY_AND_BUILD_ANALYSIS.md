# Schema Hierarchy & Recursive Build Process Analysis

**Date:** 2025-01-06  
**Purpose:** Analyze schema dependency hierarchy and improve jsonschema-to-rust pipeline compatibility

## Executive Summary

The Familiar v3 schema system has a well-structured 6-level hierarchy from atomic primitives to full entities. This document maps the dependency tree, proposes a recursive build process, and provides recommendations for improving Rust code generation compatibility.

---

## 1. Schema Dependency Hierarchy

### Level 0: Atomic Primitives (No Dependencies)
**Location:** `snippets/types/primitives/`

These are the foundation - they reference NO other schemas:

```
UUID.json                    → type: string, format: uuid
Timestamp.json               → type: string, format: date-time
NormalizedValue.json         → type: number, minimum: 0.0, maximum: 1.0
SignedNormalizedValue.json   → type: number, minimum: -1.0, maximum: 1.0
AnyValue.json                → type: any
KeyValue.json                → type: object (key-value pairs)
StringValueMap.json          → type: object (string map)
TaskList.json                → type: array
NullableTimestamp.json       → type: [string, null], format: date-time
```

**Dependency Count:** 0 external refs  
**Rust Generation:** ✅ Should work perfectly with typify/quicktype  
**Build Priority:** **LEVEL 0 - Build First**

---

### Level 1: Simple Composite Types (Depend on Level 0 only)
**Location:** `snippets/types/physics/`, `snippets/types/classification/`, `snippets/types/social/`

These combine Level 0 primitives:

```
ComplexNumber.json           → {real: number, imaginary: number}
Vec3.json                    → array of 3 numbers
Vec6.json                    → array of 6 numbers
RelationshipType.json        → enum string (Family, Friend, etc.)
EntityType.json              → enum string (Motif, Thread, Bond, etc.)
MomentType.json              → enum string
ThreadType.json              → enum string
BondState.json               → enum string (Active, Dormant, etc.)
ThreadState.json             → enum string
```

**Dependency Count:** 0-1 refs (only to Level 0)  
**Rust Generation:** ✅ Should work well  
**Build Priority:** **LEVEL 1 - Build Second**

---

### Level 2: Complex Physics Types (Depend on Levels 0-1)
**Location:** `snippets/types/physics/`

These build on simple composites:

```
DensityMatrix.json           → 2x2 array of ComplexNumber
EntanglementMap.json         → Map of UUID → NormalizedValue
PhysicsConstants.json        → Object with physics constants
AbstractionLevel.json        → NormalizedValue with constraints
CognitivePerspective.json    → Vec3 with semantic meaning
FilamentType.json            → Enum string
MotifType.json               → Enum string
```

**Dependency Count:** 1-2 refs (Levels 0-1)  
**Rust Generation:** ⚠️ Requires careful ref resolution  
**Build Priority:** **LEVEL 2 - Build Third**

---

### Level 3: Reusable Fields (Depend on Levels 0-2)
**Location:** `snippets/fields/`

Named, semantically meaningful fields that wrap types:

```
EntityId.json                → UUID (type: string, format: uuid)
TenantId.json                → UUID
UserId.json                  → UUID
CreatedAt.json               → $ref: ../types/primitives/Timestamp.json
CompletedAt.json             → $ref: ../types/primitives/Timestamp.json
Energy.json                  → {value: f64, default: 0.1}
QuantumCoherence.json        → NormalizedValue
BondDampingFactor.json       → NormalizedValue with constraints
ConsolidationRate.json       → NormalizedValue
ExplorationBias.json         → SignedNormalizedValue
Name.json                    → string with constraints
Description.json             → string
Label.json                   → string
Status.json                  → enum string
Priority.json                → enum string
Theme.json                   → string
```

**Dependency Count:** 1-2 refs (Levels 0-2)  
**Rust Generation:** ⚠️ Ref preservation critical  
**Build Priority:** **LEVEL 3 - Build Fourth**

---

### Level 4: Base Schemas & Components (Depend on Levels 0-3)
**Location:** `_base/`, `components/`

Foundation schemas for inheritance and ECS components:

#### Base Schemas (`_base/`)
```
BaseMetadata.schema.json     → schema_version, metadata fields
BasePhysics.schema.json      → engine, is_quantum flags
BaseEntity.schema.json       → entity_id, tenant_id, created_at (refs Level 3 fields)
BaseComponent.schema.json    → physics_properties, fields (refs BaseMetadata, BasePhysics)
BaseEvent.schema.json        → event structure
BaseTable.schema.json        → database table structure
BaseTaxonomy.schema.json     → taxonomy structure
BaseWorkflow.schema.json     → workflow structure
```

#### Component Schemas (`components/`)
```
QuantumState.schema.json     → density_matrix (DensityMatrix), coherence_score (NormalizedValue), 
                                entanglement_network (EntanglementMap)
UniversalPhysicsState.schema.json → energy, position, velocity
BondContent.schema.json      → relationship_type, description, history
MotifContent.schema.json     → motif_type, theme, pattern
ThreadContent.schema.json    → thread_type, description, metadata
ConsolidationState.schema.json → consolidation_rate, memory_strength
EntanglementState.schema.json → entangled_entities, strength
TemporalAnchor.schema.json   → start_date, end_date, temporal_scope
```

**Dependency Count:** 2-5 refs (Levels 0-3)  
**Rust Generation:** ⚠️ High risk - allOf, refs, custom extensions  
**Build Priority:** **LEVEL 4 - Build Fifth**

---

### Level 5: Entities (Depend on Levels 0-4)
**Location:** `entities/`

Final concrete entity types:

```
Motif.schema.json            → BaseCognitiveEntity + MotifContent + QuantumState + ConsolidationState
Thread.schema.json           → BaseCognitiveEntity + ThreadContent + UniversalPhysicsState
Bond.schema.json             → BaseCognitiveEntity + BondContent + BondTension + BondPermissions
Moment.schema.json           → BaseCognitiveEntity + MomentContent
Intent.schema.json           → BaseCognitiveEntity + IntentContent
Focus.schema.json            → BaseCognitiveEntity + FocusContent
Filament.schema.json         → BaseCognitiveEntity + FilamentContent
Tenant.schema.json           → BaseSystemEntity + TenantConfig + TenantMembers
```

**Dependency Count:** 5-10 refs (All Levels 0-4)  
**Rust Generation:** ❌ Currently failing - enum+const conflicts  
**Build Priority:** **LEVEL 5 - Build Last**

---

## 2. Recursive Build Process

### Proposed Build Pipeline

```python
#!/usr/bin/env python3
"""
Recursive Schema Build Pipeline
Builds schemas from lowest level (primitives) to highest (entities)
"""

from pathlib import Path
from typing import Dict, List, Set
import json

class RecursiveSchemaBuild:
    def __init__(self, schemas_dir: Path):
        self.schemas_dir = schemas_dir
        self.dependency_graph: Dict[str, Set[str]] = {}
        self.build_order: List[List[str]] = []  # List of levels
        self.rust_outputs: Dict[str, Path] = {}
        
    def analyze_dependencies(self):
        """Scan all schemas and build dependency graph"""
        
        # Level 0: Primitives (no dependencies)
        level_0 = self._find_schemas_with_no_refs(
            "snippets/types/primitives/*.json"
        )
        
        # Level 1: Simple types (depend only on Level 0)
        level_1 = self._find_schemas_depending_on(
            ["snippets/types/physics/*.json",
             "snippets/types/classification/*.json",
             "snippets/types/social/*.json"],
            allowed_deps=level_0
        )
        
        # Level 2: Complex types (depend on Levels 0-1)
        level_2 = self._find_schemas_depending_on(
            ["snippets/types/physics/*.json"],
            allowed_deps=level_0 + level_1
        )
        
        # Level 3: Fields (depend on Levels 0-2)
        level_3 = self._find_schemas_depending_on(
            ["snippets/fields/*.json"],
            allowed_deps=level_0 + level_1 + level_2
        )
        
        # Level 4: Base & Components (depend on Levels 0-3)
        level_4 = self._find_schemas_depending_on(
            ["_base/*.schema.json", "components/*.schema.json"],
            allowed_deps=level_0 + level_1 + level_2 + level_3
        )
        
        # Level 5: Entities (depend on all previous levels)
        level_5 = self._find_schemas_depending_on(
            ["entities/*.schema.json"],
            allowed_deps=level_0 + level_1 + level_2 + level_3 + level_4
        )
        
        self.build_order = [level_0, level_1, level_2, level_3, level_4, level_5]
        
    def build_level(self, level_num: int, schemas: List[str]) -> bool:
        """Build all schemas in a level"""
        print(f"\n{'='*60}")
        print(f"Building Level {level_num}: {len(schemas)} schemas")
        print(f"{'='*60}")
        
        success_count = 0
        fail_count = 0
        
        for schema_path in schemas:
            try:
                # Step 1: Bundle schema (resolve $refs within this level and below)
                bundled = self._bundle_schema(schema_path, level_num)
                
                # Step 2: Clean for Rust generation
                cleaned = self._clean_for_rust(bundled)
                
                # Step 3: Generate Rust code
                rust_code = self._generate_rust(cleaned, level_num)
                
                # Step 4: Store output
                self.rust_outputs[schema_path] = rust_code
                success_count += 1
                print(f"  ✅ {schema_path}")
                
            except Exception as e:
                fail_count += 1
                print(f"  ❌ {schema_path}: {e}")
                
        print(f"\nLevel {level_num} Results: {success_count} success, {fail_count} failed")
        return fail_count == 0
        
    def _bundle_schema(self, schema_path: str, level_num: int) -> dict:
        """Bundle schema, resolving refs to lower levels only"""
        # Only dereference refs to lower levels (already built)
        # Preserve refs within same level (will be module imports in Rust)
        pass
        
    def _clean_for_rust(self, schema: dict) -> dict:
        """Remove Rust-incompatible patterns"""
        # Remove: enum + const conflicts
        # Remove: format constraints that cause typify failures
        # Remove: custom x- extensions
        # Preserve: type information, structure, validation rules
        pass
        
    def _generate_rust(self, schema: dict, level_num: int) -> str:
        """Generate Rust code with appropriate strategy per level"""
        
        # Strategy depends on level:
        # Levels 0-2: Use quicktype/typify (simple data types)
        # Level 3: Use quicktype with type aliases
        # Level 4: Use custom templates (complex with allOf)
        # Level 5: Use custom templates (entities with ECS structure)
        
        if level_num <= 2:
            return self._generate_with_quicktype(schema)
        elif level_num == 3:
            return self._generate_with_type_aliases(schema)
        elif level_num >= 4:
            return self._generate_with_templates(schema)
            
    def build_all(self) -> bool:
        """Execute full recursive build"""
        self.analyze_dependencies()
        
        for level_num, schemas in enumerate(self.build_order):
            success = self.build_level(level_num, schemas)
            if not success:
                print(f"\n❌ Build failed at Level {level_num}")
                return False
                
        print(f"\n✅ All {sum(len(level) for level in self.build_order)} schemas built successfully")
        return True
```

### Build Execution Order

```bash
# Level 0: Primitives (9 schemas)
UUID.json, Timestamp.json, NormalizedValue.json, SignedNormalizedValue.json...
↓
# Level 1: Simple Types (15 schemas)  
ComplexNumber.json, Vec3.json, RelationshipType.json, EntityType.json...
↓
# Level 2: Complex Physics (10 schemas)
DensityMatrix.json, EntanglementMap.json, PhysicsConstants.json...
↓
# Level 3: Fields (30+ schemas)
EntityId.json, CreatedAt.json, Energy.json, QuantumCoherence.json...
↓
# Level 4: Base & Components (40+ schemas)
BaseEntity.schema.json, BaseComponent.schema.json, QuantumState.schema.json...
↓
# Level 5: Entities (13 schemas)
Motif.schema.json, Thread.schema.json, Bond.schema.json...
```

---

## 3. Rust Compatibility Recommendations

### Current Issues

1. **enum + const Conflicts** (Level 5 failures)
   ```json
   "entity_type": {
     "type": "string",
     "enum": ["Focus", "Filament", "Motif", ...],  // From EntityType
     "const": "Bond"  // From specific entity
   }
   ```
   **Problem:** Rust generators can't handle enum + const combination  
   **Solution:** Remove `enum` when `const` is present during bundling

2. **Constrained Numeric Types** (Level 2-3 failures)
   ```json
   {
     "type": "number",
     "minimum": 0.0,
     "maximum": 1.0
   }
   ```
   **Problem:** typify panics on Option::unwrap()  
   **Solution:** Use newtype pattern with validation in Rust

3. **$ref Preservation vs. Dereferencing**
   - **Current:** Aggressive dereferencing creates duplicated types
   - **Better:** Preserve refs to shared types, generate Rust module imports

4. **Custom Extensions** (`x-rust-type`, `physics_properties`, etc.)
   - **Current:** Ignored or cause errors
   - **Better:** Parse and use for code generation hints

### Recommended Rust Generation Strategy

#### Level 0-1: Pure Data Types
**Tool:** quicktype or typify  
**Output:** Simple Rust structs with Serde

```rust
// Generated from UUID.json
pub type UUID = uuid::Uuid;

// Generated from Timestamp.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);

// Generated from RelationshipType.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Family,
    Friend,
    Romantic,
    Professional,
    Acquaintance,
    Adversarial,
}
```

#### Level 2-3: Constrained Types with Validation
**Tool:** Custom templates (newtype pattern)  
**Output:** Rust newtypes with validation

```rust
// Generated from NormalizedValue.json
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NormalizedValue(f64);

impl NormalizedValue {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if value >= 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(ValidationError::OutOfRange {
                field: "NormalizedValue",
                min: 0.0,
                max: 1.0,
                actual: value,
            })
        }
    }
    
    pub fn get(&self) -> f64 {
        self.0
    }
}
```

#### Level 4: Components with allOf
**Tool:** Custom Jinja2 templates  
**Output:** Rust structs with proper inheritance

```rust
// Generated from QuantumState.schema.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    // From BaseComponent
    pub physics_properties: PhysicsProperties,
    
    // Component-specific fields
    pub density_matrix: DensityMatrix,
    pub coherence_score: NormalizedValue,
    pub entanglement_network: EntanglementMap,
}

impl Component for QuantumState {
    fn physics_properties(&self) -> &PhysicsProperties {
        &self.physics_properties
    }
}
```

#### Level 5: Full Entities
**Tool:** Custom Jinja2 templates with ECS patterns  
**Output:** Rust structs with component composition

```rust
// Generated from Motif.schema.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motif {
    // From BaseEntity
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub created_at: Timestamp,
    
    // Entity type (fixed for Motif)
    pub entity_type: MotifEntityType,  // Separate type, not enum
    
    // Components (ECS pattern)
    pub content: MotifContent,
    pub consolidation: ConsolidationState,
    pub gdpr: GDPRDependency,
    
    // Physics state
    pub universal_physics: UniversalPhysicsState,
    pub quantum_state: QuantumState,
    
    // Infrastructure
    pub infrastructure: QuantumComputeStack,
}

// Separate type for Motif's entity_type (not an enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotifEntityType;

impl MotifEntityType {
    pub fn as_str(&self) -> &'static str {
        "Motif"
    }
}

impl Default for MotifEntityType {
    fn default() -> Self {
        Self
    }
}
```

### Schema Preprocessing Pipeline

Before Rust generation, preprocess schemas to fix incompatibilities:

```python
def preprocess_for_rust(schema: dict, level: int) -> dict:
    """Make schema Rust-compatible"""
    
    # 1. Fix enum + const conflicts
    if 'const' in schema and 'enum' in schema:
        # Keep const, remove enum
        del schema['enum']
        
    # 2. Transform constrained numbers to newtype hints
    if schema.get('type') == 'number' and ('minimum' in schema or 'maximum' in schema):
        schema['x-rust-newtype'] = True
        schema['x-rust-validation'] = {
            'min': schema.pop('minimum', None),
            'max': schema.pop('maximum', None),
        }
        
    # 3. Remove format constraints that break typify
    if schema.get('type') == 'string' and level <= 3:
        # For lower levels, remove format (we'll use x-rust-type instead)
        if 'format' in schema and schema['format'] in ['uuid', 'date-time']:
            schema['x-rust-type'] = {
                'uuid': 'uuid::Uuid',
                'date-time': 'chrono::DateTime<chrono::Utc>',
            }[schema['format']]
            del schema['format']
            
    # 4. Preserve refs to shared types (don't dereference)
    # (Handled in bundling step)
    
    return schema
```

---

## 4. Implementation Roadmap

### Phase 1: Fix Critical Issues (Week 1)
- [ ] Implement recursive build script with level-by-level processing
- [ ] Add schema preprocessing for enum+const conflicts
- [ ] Fix constrained numeric type handling
- [ ] **Target:** 100% entity generation success

### Phase 2: Improve Quality (Week 2)
- [ ] Implement newtype pattern for constrained types
- [ ] Add validation code generation
- [ ] Improve ref preservation for shared types
- [ ] **Target:** Clean, idiomatic Rust code

### Phase 3: Advanced Features (Week 3)
- [ ] ECS component trait generation
- [ ] Custom derive macros for entities
- [ ] Integration with physics engine
- [ ] **Target:** Production-ready code generation

### Phase 4: Automation (Week 4)
- [ ] CI/CD integration for schema changes
- [ ] Automatic dependency analysis
- [ ] Schema validation pipeline
- [ ] **Target:** Fully automated schema-to-rust pipeline

---

## 5. Testing Strategy

### Unit Tests per Level

```bash
# Test Level 0: Primitives
cargo test --package familiar_types_primitives

# Test Level 1: Simple Types  
cargo test --package familiar_types_physics
cargo test --package familiar_types_classification

# Test Level 2: Complex Types
cargo test --package familiar_types_quantum

# Test Level 3: Fields
cargo test --package familiar_fields

# Test Level 4: Components
cargo test --package familiar_components

# Test Level 5: Entities
cargo test --package familiar_entities
```

### Integration Tests

```rust
#[test]
fn test_motif_with_quantum_state() {
    let motif = Motif {
        entity_id: EntityId::new(),
        quantum_state: QuantumState {
            density_matrix: DensityMatrix::new([[
                ComplexNumber { real: 1.0, imaginary: 0.0 },
                ComplexNumber { real: 0.0, imaginary: 0.0 },
            ], [
                ComplexNumber { real: 0.0, imaginary: 0.0 },
                ComplexNumber { real: 0.0, imaginary: 0.0 },
            ]]),
            coherence_score: NormalizedValue::new(1.0).unwrap(),
            entanglement_network: EntanglementMap::default(),
        },
        // ... other fields
    };
    
    assert_eq!(motif.quantum_state.coherence_score.get(), 1.0);
}
```

---

## 6. Success Metrics

| Metric | Current | Target (Phase 1) | Target (Phase 4) |
|--------|---------|------------------|------------------|
| Primitive Success | 100% | 100% | 100% |
| Type Success | 90% | 100% | 100% |
| Field Success | 85% | 100% | 100% |
| Component Success | 60% | 95% | 100% |
| Entity Success | 0% | 100% | 100% |
| **Overall Success** | **88.1%** | **98%** | **100%** |
| Build Time | N/A | <30s | <10s |
| Code Quality | Manual | Good | Excellent |

---

## 7. Key Insights

### What Works Well
1. ✅ **Clear hierarchy** - 6 levels with minimal circular dependencies
2. ✅ **Atomic primitives** - Strong foundation with zero dependencies
3. ✅ **Consistent patterns** - $ref usage is predictable and logical
4. ✅ **Semantic separation** - types/fields/components/entities well-organized

### What Needs Improvement
1. ⚠️ **enum + const conflicts** - Need preprocessing step
2. ⚠️ **Constrained numerics** - Need newtype pattern in Rust
3. ⚠️ **Over-dereferencing** - Need smarter ref preservation
4. ⚠️ **Custom extensions** - Need better parsing and utilization

### Recommended Next Steps
1. **Immediate:** Implement recursive build script (this document's approach)
2. **Next:** Add schema preprocessing for Rust compatibility
3. **Then:** Create custom templates for Levels 4-5
4. **Finally:** Full automation and CI/CD integration

---

## Appendix: Dependency Graph Visualization

```
Level 0: Primitives (9)
  ├─ UUID
  ├─ Timestamp  
  ├─ NormalizedValue
  └─ ...

Level 1: Simple Types (15)
  ├─ ComplexNumber → (no refs)
  ├─ Vec3 → (no refs)
  ├─ RelationshipType → (enum)
  └─ ...

Level 2: Complex Types (10)
  ├─ DensityMatrix → ComplexNumber (Level 1)
  ├─ EntanglementMap → UUID (Level 0), NormalizedValue (Level 0)
  └─ ...

Level 3: Fields (30+)
  ├─ EntityId → UUID (Level 0)
  ├─ CreatedAt → Timestamp (Level 0)
  ├─ QuantumCoherence → NormalizedValue (Level 0)
  └─ ...

Level 4: Components (40+)
  ├─ QuantumState → DensityMatrix (L2), NormalizedValue (L0), EntanglementMap (L2)
  ├─ BondContent → RelationshipType (L1), Description (L3)
  └─ ...

Level 5: Entities (13)
  ├─ Motif → MotifContent (L4), QuantumState (L4), ConsolidationState (L4)
  ├─ Thread → ThreadContent (L4), UniversalPhysicsState (L4)
  └─ ...
```

---

**End of Analysis**

