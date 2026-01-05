# Schema Dependency Graph - Visual Reference

**Purpose:** Quick visual reference for schema dependencies and build order

## Dependency Flow (Bottom-Up Build Order)

```
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 5: ENTITIES (13 schemas)                              │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  Motif.schema.json          Thread.schema.json              │
│  Bond.schema.json           Moment.schema.json              │
│  Intent.schema.json         Focus.schema.json               │
│  Filament.schema.json       Tenant.schema.json              │
│  Course.schema.json         Shuttle.schema.json             │
│  Stitch.schema.json         GenericThread.schema.json       │
│  PersonThread.schema.json                                   │
│                                                              │
│  Dependencies: ALL (Levels 0-4)                             │
│  Current Success: 0%  →  Target: 90%+                       │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            │ depends on
                            │
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 4: BASE & COMPONENTS (40+ schemas)                    │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  _base/                        components/                  │
│  ├─ BaseEntity                 ├─ QuantumState              │
│  ├─ BaseComponent              ├─ UniversalPhysicsState     │
│  ├─ BaseMetadata               ├─ BondContent               │
│  ├─ BasePhysics                ├─ MotifContent              │
│  ├─ BaseEvent                  ├─ ThreadContent             │
│  ├─ BaseCognitiveEntity        ├─ ConsolidationState        │
│  ├─ BaseSystemEntity           ├─ EntanglementState         │
│  └─ ...                        └─ ...                       │
│                                                              │
│  Dependencies: Levels 0-3                                   │
│  Current Success: ~60%  →  Target: 85%                      │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            │ depends on
                            │
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 3: FIELDS (30+ schemas)                               │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  Identifiers:          Timestamps:         Physics:         │
│  ├─ EntityId           ├─ CreatedAt        ├─ Energy        │
│  ├─ TenantId           ├─ CompletedAt      ├─ QuantumCoh..  │
│  ├─ UserId             ├─ StartDate        ├─ BondDamping.. │
│  └─ ...                └─ ...              └─ ...           │
│                                                              │
│  Metadata:             Status:             Constraints:     │
│  ├─ Name               ├─ Status           ├─ ConsolidRt..  │
│  ├─ Description        ├─ Priority         ├─ ExplrBias..   │
│  ├─ Label              ├─ AccessType       └─ ...           │
│  └─ Theme              └─ ...                               │
│                                                              │
│  Dependencies: Levels 0-2                                   │
│  Current Success: ~85%  →  Target: 95%                      │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            │ depends on
                            │
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 2: COMPLEX TYPES (10 schemas)                         │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  Quantum Physics:                 Classification:           │
│  ├─ DensityMatrix                 ├─ FilamentType           │
│  │   (2x2 ComplexNumber[][])      ├─ MotifType              │
│  ├─ EntanglementMap               └─ ...                    │
│  │   (Map<UUID, NormalizedValue>) │                         │
│  ├─ PhysicsConstants              Cognitive:                │
│  ├─ AbstractionLevel              ├─ CognitivePerspective   │
│  └─ ...                           └─ ...                    │
│                                                              │
│  Dependencies: Levels 0-1                                   │
│  Current Success: ~60%  →  Target: 95%                      │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            │ depends on
                            │
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 1: SIMPLE TYPES (15 schemas)                          │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  Math Types:               Enums:                           │
│  ├─ ComplexNumber          ├─ RelationshipType             │
│  │   {real, imaginary}     ├─ EntityType                   │
│  ├─ Vec3 [f64; 3]          ├─ MomentType                   │
│  └─ Vec6 [f64; 6]          ├─ ThreadType                   │
│                            ├─ BondState                     │
│  Lifecycle:                ├─ ThreadState                   │
│  ├─ BondStateReason        ├─ BondStateReason              │
│  └─ ThreadStateReason      └─ ThreadStateReason            │
│                                                              │
│  Dependencies: Level 0 only                                 │
│  Current Success: ~90%  →  Target: 100%                     │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            │ depends on
                            │
┌─────────────────────────────────────────────────────────────┐
│ LEVEL 0: PRIMITIVES (9 schemas)                             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                              │
│  UUID.json                  ← type: string, format: uuid    │
│  Timestamp.json             ← type: string, format: date    │
│  NormalizedValue.json       ← number [0.0, 1.0]             │
│  SignedNormalizedValue.json ← number [-1.0, 1.0]            │
│  AnyValue.json              ← type: any                     │
│  KeyValue.json              ← type: object                  │
│  StringValueMap.json        ← type: object (string map)     │
│  TaskList.json              ← type: array                   │
│  NullableTimestamp.json     ← type: [string, null]          │
│                                                              │
│  Dependencies: NONE (foundational)                          │
│  Current Success: 100%  →  Target: 100%                     │
└─────────────────────────────────────────────────────────────┘
```

## Concrete Example: Building a Motif

### Dependency Chain for Motif.schema.json

```
Motif (Level 5)
├─ BaseCognitiveEntity (Level 4)
│  ├─ BaseEntity (Level 4)
│  │  ├─ EntityId (Level 3)
│  │  │  └─ UUID (Level 0) ✓
│  │  ├─ TenantId (Level 3)
│  │  │  └─ UUID (Level 0) ✓
│  │  └─ CreatedAt (Level 3)
│  │     └─ Timestamp (Level 0) ✓
│  │
│  └─ EntityType (Level 1)
│     └─ enum string ✓
│
├─ MotifContent (Level 4)
│  ├─ MotifType (Level 2)
│  │  └─ enum string ✓
│  └─ Theme (Level 3)
│     └─ string ✓
│
├─ QuantumState (Level 4)
│  ├─ DensityMatrix (Level 2)
│  │  └─ ComplexNumber[][] (Level 1)
│  │     └─ {real: number, imaginary: number} ✓
│  ├─ coherence_score (Level 0)
│  │  └─ NormalizedValue (Level 0) ✓
│  └─ EntanglementMap (Level 2)
│     ├─ UUID (Level 0) ✓
│     └─ NormalizedValue (Level 0) ✓
│
└─ ConsolidationState (Level 4)
   └─ ConsolidationRate (Level 3)
      └─ NormalizedValue (Level 0) ✓
```

**Build Order for Motif:**
1. Level 0: UUID, Timestamp, NormalizedValue ✓
2. Level 1: ComplexNumber, EntityType ✓
3. Level 2: DensityMatrix, EntanglementMap, MotifType ✓
4. Level 3: EntityId, TenantId, CreatedAt, Theme, ConsolidationRate ✓
5. Level 4: BaseEntity, BaseCognitiveEntity, MotifContent, QuantumState, ConsolidationState ✓
6. Level 5: Motif ✓

## Problem Zones (Where Rust Generation Fails)

### 🔴 Critical: enum + const Conflicts (Level 5)

```json
// In Motif.schema.json
{
  "entity_type": {
    "$ref": "snippets/types/classification/EntityType.json",  // enum of all types
    "const": "Motif"  // ← CONFLICT: specific value
  }
}

// EntityType.json contains:
{
  "type": "string",
  "enum": ["Focus", "Filament", "Motif", "Intent", "Moment", "Bond", "Thread"]
}
```

**Why it fails:** Rust generators can't reconcile "must be one of 7 values" with "must be this specific value"

**Fix in recursive build:**
```python
# Preprocessing removes enum when const is present
if 'enum' in obj and 'const' in obj:
    del obj['enum']  # Keep const for specificity
```

### ⚠️  Warning: Constrained Numerics (Levels 0-3)

```json
// NormalizedValue.json
{
  "type": "number",
  "minimum": 0.0,
  "maximum": 1.0
}
```

**Why it fails:** typify panics on `Option::unwrap()` when processing constraints

**Fix in recursive build:**
```python
# Transform to newtype hint
schema['x-rust-newtype'] = True
schema['x-rust-validation'] = {
    'min': schema.pop('minimum'),
    'max': schema.pop('maximum')
}
```

**Generated Rust:**
```rust
#[derive(Debug, Clone, Copy)]
pub struct NormalizedValue(f64);

impl NormalizedValue {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if value >= 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(ValidationError::OutOfRange { /* ... */ })
        }
    }
}
```

## Rust Module Structure (Target)

```rust
// After successful build, target structure:

familiar/
├── Cargo.toml (workspace)
└── src/
    └── generated/
        ├── primitives/          // Level 0
        │   ├── lib.rs
        │   ├── uuid.rs
        │   ├── timestamp.rs
        │   └── normalized_value.rs
        │
        ├── types/               // Levels 1-2
        │   ├── lib.rs
        │   ├── complex_number.rs
        │   ├── vec3.rs
        │   ├── density_matrix.rs
        │   └── ...
        │
        ├── fields/              // Level 3
        │   ├── lib.rs
        │   ├── entity_id.rs
        │   ├── created_at.rs
        │   └── ...
        │
        ├── components/          // Level 4
        │   ├── lib.rs
        │   ├── quantum_state.rs
        │   ├── bond_content.rs
        │   └── ...
        │
        └── entities/            // Level 5
            ├── lib.rs
            ├── motif.rs
            ├── thread.rs
            ├── bond.rs
            └── ...
```

## Import Graph (Rust Dependencies)

```rust
// entities/motif.rs (Level 5)
use crate::primitives::{UUID, Timestamp, NormalizedValue};  // Level 0
use crate::types::{ComplexNumber, DensityMatrix};           // Levels 1-2
use crate::fields::{EntityId, TenantId, CreatedAt};         // Level 3
use crate::components::{QuantumState, MotifContent};        // Level 4

pub struct Motif {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub created_at: CreatedAt,
    pub quantum_state: QuantumState,
    pub content: MotifContent,
    // ...
}
```

## Success Metrics by Level

| Level | Name | Count | Current | Target | Priority |
|-------|------|-------|---------|--------|----------|
| 0 | Primitives | 9 | 100% ✅ | 100% | ⭐ |
| 1 | Simple Types | 15 | ~90% | 100% | ⭐⭐ |
| 2 | Complex Types | 10 | ~60% ⚠️  | 95% | ⭐⭐⭐ |
| 3 | Fields | 30+ | ~85% | 95% | ⭐⭐ |
| 4 | Components | 40+ | ~60% ⚠️  | 85% | ⭐⭐⭐ |
| 5 | **Entities** | 13 | **0% 🔴** | **90%** | **⭐⭐⭐⭐** |
| **Total** | | **117+** | **88.1%** | **95%** | |

## Quick Command Reference

```bash
# 1. Analyze schema hierarchy
cd /Users/erictheiss/familiar/docs/v3
python3 scripts/recursive_schema_build.py --help

# 2. Run recursive build
python3 scripts/recursive_schema_build.py \
  --schemas-dir schemas \
  --output-dir rust_generated

# 3. Check statistics
# (Script outputs detailed stats per level)

# 4. Test generated code
cd rust_generated/level_0_primitives
cargo init --lib
cargo test

# 5. Integrate into workspace
cp -r rust_generated/* ../../src/generated/
```

## Timeline to 95%+ Success

```
Week 1: Foundation
├─ Day 1-2: Test recursive build script
├─ Day 3-4: Iterate on preprocessing
└─ Day 5: Verify Level 0-3 (should be 95%+)

Week 2: Components
├─ Day 1-2: Custom templates for Level 4
├─ Day 3-4: Test component generation
└─ Day 5: Verify Level 4 (target 85%+)

Week 3: Entities
├─ Day 1-2: Custom templates for Level 5
├─ Day 3-4: Test entity generation
└─ Day 5: Verify Level 5 (target 90%+)

Week 4: Polish
├─ Day 1-2: Improve code quality
├─ Day 3-4: Add validation and tests
└─ Day 5: Final verification (target 95%+)
```

## Key Insight: The Hierarchy Was Always There

The schema structure **already has a perfect dependency hierarchy** - it just wasn't being utilized by the build process.

**Before (Flat Build):**
- Process all schemas in arbitrary order
- Hope dependencies are resolved
- 88% success, 0% entities

**After (Recursive Build):**
- Honor the natural dependency hierarchy
- Build primitives first, entities last
- Expected 95%+ success, 90%+ entities

The schemas don't need to change - **the build process** needs to respect their structure.

---

**Next Action:** Run the script and verify the dependency graph is correct:
```bash
python3 scripts/recursive_schema_build.py
```

