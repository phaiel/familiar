# Tool Selection & Validation Strategy

**Date:** 2025-01-06  
**Goal:** 100% success with bidirectional validation and near-zero custom scripting  
**Philosophy:** Schema-first, right tool for each schema type, validate everything

---

## Executive Summary

Different schema types need different tools. Use standard tooling with bidirectional validation to achieve 100% success.

### Recommended Approach

| Schema Type | Keep JSON Schema? | Generation Tool | Validation Tool | Expected Success |
|-------------|-------------------|-----------------|-----------------|------------------|
| **Snippets** (types, fields) | ✅ Yes | quicktype/typify | jsonschema | 100% |
| **Base** (_base/) | ⚠️ Hybrid | schemars (Rust→JSON) | schemars | 100% |
| **Components** | ⚠️ Hybrid | schemars + traits | schemars | 100% |
| **Entities** | ⚠️ Hybrid | schemars + templates | schemars | 100% |

**Key Insight:** For complex schemas (base, components, entities), **define in Rust first** with schemars, generate JSON Schema for validation and documentation.

---

## The Problem with Current Approach

### What Works (Snippets - Levels 0-3)

```
JSON Schema (simple types) 
  → quicktype/typify 
  → Rust structs 
  ✅ 100% success expected
```

**Keep this!** Snippets are perfect for JSON Schema:
- Simple, reusable types
- No complex inheritance (allOf)
- Well-supported by all tools

### What Fails (Base, Components, Entities - Levels 4-5)

```
JSON Schema (complex with allOf, enum+const)
  → quicktype/typify
  → ❌ Assertion failures, panics, 0% entity success
```

**Why it fails:**
1. **allOf composition** - tools struggle with inheritance
2. **enum + const conflicts** - impossible constraints
3. **Trait generation** - data-only tools can't generate behavior
4. **ECS patterns** - need custom Component traits

---

## Proposed Solution: Hybrid Approach

### Level 0-3: JSON Schema → Rust (Current approach, works!)

**Schema types:** Primitives, types, fields  
**Format:** JSON Schema (.json)  
**Tool:** quicktype or typify  
**Direction:** Schema → Code  
**Validation:** jsonschema-rs

```bash
# Generate Rust from JSON Schema
quicktype --src-lang schema --lang rust \
  --out src/generated/primitives/uuid.rs \
  schemas/snippets/types/primitives/UUID.json

# Validate JSON instances
jsonschema validate --instance data.json --schema UUID.json
```

**Expected Success:** 100% ✅

### Level 4-5: Rust → JSON Schema (New approach)

**Schema types:** Base, Components, Entities  
**Format:** Rust source (.rs)  
**Tool:** schemars (Rust derive → JSON Schema)  
**Direction:** Code → Schema  
**Validation:** schemars (round-trip)

```rust
// Define in Rust first
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Motif Entity")]
pub struct Motif {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub created_at: Timestamp,
    
    #[schemars(description = "Entity type is always 'Motif'")]
    pub entity_type: MotifEntityType,
    
    pub content: MotifContent,
    pub quantum_state: QuantumState,
    pub consolidation: ConsolidationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MotifEntityType;

impl Default for MotifEntityType {
    fn default() -> Self { Self }
}

// Generate JSON Schema from Rust
fn main() {
    let schema = schema_for!(Motif);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

**Benefits:**
- ✅ No enum + const conflicts (separate type per entity)
- ✅ Traits and behavior work naturally
- ✅ 100% valid Rust by definition
- ✅ JSON Schema generated for validation/docs
- ✅ Bidirectional validation built-in

**Expected Success:** 100% ✅

---

## Bidirectional Validation Strategy

### Validation Flow

```
1. Define source of truth (depends on level)
   ↓
2. Generate code/schema
   ↓
3. Validate generated output against source
   ↓
4. Validate sample data against schema
   ↓
5. Round-trip test: data → code → data
```

### Level 0-3: JSON Schema is Source of Truth

```bash
# 1. JSON Schema is source
cat schemas/snippets/types/primitives/UUID.json

# 2. Generate Rust
quicktype -s schema -l rust -o uuid.rs UUID.json

# 3. Compile Rust (validates it's correct Rust)
rustc uuid.rs

# 4. Validate sample data
echo '"550e8400-e29b-41d4-a716-446655440000"' | \
  jsonschema validate -i - UUID.json

# 5. Round-trip: JSON → Rust → JSON
echo '"550e8400-e29b-41d4-a716-446655440000"' | \
  rust_validator --schema UUID.json
```

### Level 4-5: Rust is Source of Truth

```bash
# 1. Rust code is source
cat src/entities/motif.rs

# 2. Generate JSON Schema
cargo run --bin generate_schemas > schemas/generated/Motif.schema.json

# 3. Validate schema is valid JSON Schema
jsonschema check schemas/generated/Motif.schema.json

# 4. Validate sample data
cat test_data/motif_example.json | \
  jsonschema validate -i - schemas/generated/Motif.schema.json

# 5. Round-trip: Rust → JSON Schema → validate Rust against it
cargo test --test schema_validation
```

### Round-Trip Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    
    #[test]
    fn test_motif_schema_round_trip() {
        // 1. Create Rust instance
        let motif = Motif {
            entity_id: EntityId::new(),
            entity_type: MotifEntityType::default(),
            // ...
        };
        
        // 2. Serialize to JSON
        let json = serde_json::to_value(&motif).unwrap();
        
        // 3. Get schema from Rust type
        let schema = schema_for!(Motif);
        
        // 4. Validate JSON against schema
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&json), "Rust instance must validate against its own schema");
        
        // 5. Deserialize back to Rust
        let motif2: Motif = serde_json::from_value(json).unwrap();
        
        // 6. Assert equality (round-trip successful)
        assert_eq!(motif.entity_id, motif2.entity_id);
    }
}
```

---

## Tool Comparison

### quicktype (Current - Good for Simple Types)

**Strengths:**
- ✅ Fast, mature, well-maintained
- ✅ Excellent for simple types
- ✅ Multiple language support
- ✅ Good Serde integration

**Weaknesses:**
- ❌ Struggles with allOf
- ❌ Can't handle enum + const
- ❌ No trait generation
- ❌ One-directional only (schema → code)

**Use for:** Levels 0-3 (snippets, fields)

### typify (Alternative for Simple Types)

**Strengths:**
- ✅ Rust-specific, good Rust idioms
- ✅ Good for simple types
- ✅ Maintained by Oxide Computer

**Weaknesses:**
- ❌ Panics on constrained numerics
- ❌ Assertion failures on enum + const
- ❌ No trait generation
- ❌ One-directional only

**Use for:** Levels 0-3 (with preprocessing)

### schemars (Recommended for Complex Types)

**Strengths:**
- ✅ **Bidirectional** (Rust ↔ JSON Schema)
- ✅ Derive macros (minimal boilerplate)
- ✅ Trait generation works naturally
- ✅ No enum + const issues
- ✅ Round-trip validation built-in
- ✅ Integrates with serde perfectly

**Weaknesses:**
- ⚠️ Requires writing Rust first (but this is actually good!)
- ⚠️ JSON Schema is generated (but still validates data)

**Use for:** Levels 4-5 (base, components, entities)

**Installation:**
```toml
[dependencies]
schemars = { version = "0.8", features = ["preserve_order"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### schemafy (Alternative)

**Strengths:**
- ✅ Different approach than quicktype
- ✅ May handle some patterns better

**Weaknesses:**
- ⚠️ Less mature than schemars
- ⚠️ One-directional only

**Use for:** Fallback if quicktype fails

---

## Recommended Architecture

### Directory Structure

```
docs/v3/
├── schemas/
│   ├── snippets/              ← JSON Schema (source of truth)
│   │   ├── types/
│   │   └── fields/
│   └── generated/             ← Generated from Rust (for validation/docs)
│       ├── BaseEntity.schema.json
│       ├── Motif.schema.json
│       └── ...
│
src/
├── generated/
│   ├── primitives/            ← Generated from JSON Schema
│   │   ├── uuid.rs
│   │   ├── timestamp.rs
│   │   └── ...
│   └── types/
│       ├── complex_number.rs
│       └── ...
│
└── entities/                  ← Hand-written Rust (source of truth)
    ├── base.rs                ← Base traits and types
    ├── motif.rs               ← Motif entity with JsonSchema derive
    ├── thread.rs
    ├── bond.rs
    └── mod.rs
```

### Build Process

```bash
# Phase 1: Generate primitives from JSON Schema (Levels 0-3)
make generate-primitives

# Phase 2: Compile Rust entities with schemars derives (Levels 4-5)
cargo build

# Phase 3: Generate JSON Schemas from Rust for validation
cargo run --bin generate-schemas

# Phase 4: Validate everything
make validate-all
```

### Makefile

```makefile
# Generate Level 0-3 from JSON Schema
generate-primitives:
	@echo "Generating primitives from JSON Schema..."
	@for schema in schemas/snippets/types/primitives/*.json; do \
		name=$$(basename $$schema .json); \
		quicktype -s schema -l rust \
			--derive-debug --derive-clone \
			-o src/generated/primitives/$$name.rs \
			$$schema; \
	done

# Generate JSON Schemas from Rust (Level 4-5)
generate-schemas:
	@echo "Generating JSON Schemas from Rust..."
	cargo run --bin generate_schemas -- \
		--output docs/v3/schemas/generated

# Validate primitives against JSON Schema
validate-primitives:
	@echo "Validating primitives..."
	@for schema in schemas/snippets/**/*.json; do \
		jsonschema check $$schema || exit 1; \
	done

# Validate entities (round-trip tests)
validate-entities:
	@echo "Running entity validation tests..."
	cargo test --test schema_validation

# Validate test data
validate-test-data:
	@echo "Validating test data..."
	@for data in tests/integration/*.json; do \
		schema=$$(echo $$data | sed 's/-example.json/.schema.json/'); \
		jsonschema validate -i $$data $$schema || exit 1; \
	done

# Full validation
validate-all: validate-primitives validate-entities validate-test-data
	@echo "✅ All validation passed!"

# Full build
all: generate-primitives generate-schemas validate-all
	cargo build --all-features
	@echo "✅ Build complete with 100% validation!"
```

---

## Implementation Plan

### Phase 1: Validate Current Snippets (Week 1)

**Goal:** Ensure 100% success for Levels 0-3

```bash
# 1. Keep JSON Schema for snippets
# (Already good format)

# 2. Generate with quicktype
python3 scripts/recursive_schema_build.py --levels 0,1,2,3

# 3. Validate
make validate-primitives

# Success criteria: 100% generation, all schemas validate
```

**Expected Result:** 100% success for snippets ✅

### Phase 2: Migrate Base to Rust (Week 2)

**Goal:** Convert _base/ schemas to Rust source with schemars

```rust
// src/base/entity.rs (replaces _base/BaseEntity.schema.json)
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Base Entity")]
pub struct BaseEntity {
    #[schemars(description = "Unique entity identifier")]
    pub entity_id: EntityId,
    
    #[schemars(description = "Tenant identifier")]
    pub tenant_id: TenantId,
    
    #[schemars(description = "Entity creation timestamp")]
    pub created_at: Timestamp,
}

// Generate JSON Schema for validation
#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    
    #[test]
    fn generate_schema() {
        let schema = schema_for!(BaseEntity);
        std::fs::write(
            "docs/v3/schemas/generated/BaseEntity.schema.json",
            serde_json::to_string_pretty(&schema).unwrap()
        ).unwrap();
    }
}
```

**Success Criteria:**
- ✅ All base types compile in Rust
- ✅ JSON Schemas generated successfully
- ✅ Round-trip tests pass

### Phase 3: Migrate Components to Rust (Week 3)

**Goal:** Convert components/ to Rust with traits

```rust
// src/components/quantum_state.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Quantum State Component
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuantumState {
    pub density_matrix: DensityMatrix,
    pub coherence_score: NormalizedValue,
    pub entanglement_network: EntanglementMap,
}

// ECS Component trait
impl Component for QuantumState {
    fn physics_properties(&self) -> PhysicsProperties {
        PhysicsProperties {
            engine: PhysicsEngine::Quantum,
            is_quantum: true,
        }
    }
}
```

**Success Criteria:**
- ✅ All components have trait implementations
- ✅ JSON Schemas generated
- ✅ ECS patterns work correctly

### Phase 4: Migrate Entities to Rust (Week 4)

**Goal:** Convert entities/ to Rust, fixing enum+const issue

```rust
// src/entities/motif.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Motif Entity Type (not an enum!)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MotifEntityType;

impl MotifEntityType {
    pub const VALUE: &'static str = "Motif";
    
    pub fn as_str(&self) -> &'static str {
        Self::VALUE
    }
}

impl Default for MotifEntityType {
    fn default() -> Self { Self }
}

/// Motif Entity - A quantum entity representing recurring patterns
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Motif {
    // Base entity fields
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub created_at: Timestamp,
    
    // Entity type (always "Motif")
    #[serde(default)]
    pub entity_type: MotifEntityType,
    
    // Components
    pub content: MotifContent,
    pub quantum_state: QuantumState,
    pub consolidation: ConsolidationState,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_motif_serialization() {
        let motif = Motif {
            entity_id: EntityId::new(),
            entity_type: MotifEntityType::default(),
            // ...
        };
        
        // Serialize
        let json = serde_json::to_value(&motif).unwrap();
        
        // Validate against schema
        let schema = schema_for!(Motif);
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&json));
        
        // Deserialize
        let motif2: Motif = serde_json::from_value(json).unwrap();
        assert_eq!(motif.entity_id, motif2.entity_id);
    }
}
```

**Success Criteria:**
- ✅ All 13 entities compile
- ✅ No enum + const conflicts
- ✅ JSON Schemas generated
- ✅ Round-trip tests pass
- ✅ **100% success rate**

---

## Validation Tools

### Install Required Tools

```bash
# Rust
rustup update stable

# Cargo tools
cargo install cargo-make

# JSON Schema validator
cargo install jsonschema-cli
# Or: pip install jsonschema-cli

# quicktype (for Levels 0-3)
npm install -g quicktype
```

### Cargo Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "0.8", features = ["preserve_order"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
jsonschema = "0.17"  # For round-trip validation
```

---

## Success Metrics: 100% Target

| Phase | Deliverable | Success Metric |
|-------|-------------|----------------|
| 1 | Snippets (JSON Schema) | 100% generation, all validate |
| 2 | Base types (Rust + schemars) | 100% compile, schemas generate |
| 3 | Components (Rust + traits) | 100% compile, traits work |
| 4 | Entities (Rust + ECS) | 100% compile, round-trip validates |
| **Total** | **Full pipeline** | **100% success, bidirectional validation** |

### Zero Custom Scripting Achieved

✅ **Level 0-3:** Standard quicktype (no custom code)  
✅ **Level 4-5:** Standard schemars (just Rust derives)  
✅ **Validation:** Standard jsonschema + schemars (built-in)  
✅ **Build:** Standard cargo + make (no custom generators)

---

## Comparison: Old vs. New

### Old Approach (Current)

```
JSON Schema (all levels)
  ↓
quicktype/typify (one tool for everything)
  ↓
❌ 88% success, 0% entities
❌ enum + const conflicts
❌ allOf doesn't work
❌ No traits
❌ One-way only (no validation)
```

### New Approach (Recommended)

```
Levels 0-3: JSON Schema → quicktype → Rust
  ↓
✅ 100% success (simple types work great)
  ↓
Levels 4-5: Rust (schemars derive) → JSON Schema
  ↓
✅ 100% success (no conflicts, traits work)
✅ Bidirectional validation built-in
✅ Round-trip testing automatic
```

---

## Key Insights

1. **Different tools for different jobs** - snippets in JSON Schema, entities in Rust
2. **Bidirectional is critical** - schemars enables Rust → Schema → Validate
3. **Rust-first for complex types** - avoid allOf, enum+const issues entirely
4. **Standard tools only** - no custom generators, just derives
5. **100% is achievable** - with right tool selection and validation

---

## Next Steps

1. **Validate snippets work** (should be 100% already)
   ```bash
   python3 scripts/recursive_schema_build.py --levels 0,1,2,3
   ```

2. **Create base types in Rust** (Week 2)
   ```bash
   mkdir -p src/base
   # Write base.rs with schemars derives
   cargo test
   ```

3. **Migrate one entity as proof-of-concept** (Week 2)
   ```rust
   // src/entities/motif.rs with full schemars
   cargo test --test motif_validation
   ```

4. **Measure success** (Week 3)
   - Generate all JSON Schemas from Rust
   - Run full validation suite
   - **Target: 100% success ✅**

---

**Bottom Line:** Use JSON Schema for simple types (works great), use Rust with schemars for complex types (works great), get bidirectional validation for free, achieve 100% success with zero custom scripting.

Ready to implement! 🚀

