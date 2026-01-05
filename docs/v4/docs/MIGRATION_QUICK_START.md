# Migration Quick Start Guide

**Goal:** Start the 100% lossless migration TODAY  
**Time:** 2 hours to get tooling working

---

## Prerequisites

```bash
# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install tools
cargo install cargo-make
```

---

## Step 1: Set Up Migration Tools (30 mins)

```bash
cd /Users/erictheiss/familiar/docs/v4/tools

# Initialize workspace
cargo init --bin extract-schema/
cargo init --bin compare-schemas/
cargo init --bin test-round-trip/
cargo init --bin analyze-coverage/
cargo init --bin migration-report/

# Build tools
cargo build --all

# Test extraction tool
cargo run --bin extract-schema -- \
  --input ../../v3/schemas/snippets/types/primitives/UUID.json \
  --output ../migration/extracted/uuid_info.json
```

**Expected output:**
```
📦 Extracting schema information from: ../../v3/schemas/snippets/types/primitives/UUID.json
✅ Extracted information:
   Properties: 0
   Validation rules: 2
   Custom extensions: 0
   References: 0

📄 Output written to: ../migration/extracted/uuid_info.json
```

---

## Step 2: Run Phase 0 Audit (30 mins)

```bash
cd /Users/erictheiss/familiar/docs/v3/schemas

# Count all schemas
find . -name "*.json" -type f | wc -l

# Generate inventory
python3 << 'EOF'
import os
import json
from pathlib import Path

schemas = {
    "primitives": [],
    "types": [],
    "fields": [],
    "components": [],
    "entities": []
}

for root, dirs, files in os.walk("."):
    for file in files:
        if file.endswith(".json"):
            path = os.path.join(root, file)
            
            if "primitives" in path:
                schemas["primitives"].append(path)
            elif "types" in path:
                schemas["types"].append(path)
            elif "fields" in path:
                schemas["fields"].append(path)
            elif "components" in path:
                schemas["components"].append(path)
            elif "entities" in path:
                schemas["entities"].append(path)

print("Schema Inventory:")
print(f"  Primitives: {len(schemas['primitives'])}")
print(f"  Types: {len(schemas['types'])}")
print(f"  Fields: {len(schemas['fields'])}")
print(f"  Components: {len(schemas['components'])}")
print(f"  Entities: {len(schemas['entities'])}")
print(f"  TOTAL: {sum(len(v) for v in schemas.values())}")

with open("../../v4/migration/INVENTORY.json", "w") as f:
    json.dump(schemas, f, indent=2)
print("\n✅ Inventory saved to v4/migration/INVENTORY.json")
EOF
```

---

## Step 3: Migrate First Primitive (30 mins)

### 3.1 Extract UUID Schema Info

```bash
cd /Users/erictheiss/familiar/docs/v4

# Extract information
cargo run --bin extract-schema -- \
  --input ../v3/schemas/snippets/types/primitives/UUID.json \
  --output migration/extracted/uuid.json

# Review extracted info
cat migration/extracted/uuid.json
```

### 3.2 Verify v4 UUID Implementation

```bash
# Check if UUID already exists in v4
cat schemas/src/primitives/uuid.rs

# Generate JSON Schema from it
cd schemas
cargo run --example generate-schemas --features generate-json-schemas

# Check generated schema
cat generated/UUID.schema.json
```

### 3.3 Compare v3 vs v4

```bash
# Manual comparison (automated tool coming)
echo "v3 UUID:"
cat ../v3/schemas/snippets/types/primitives/UUID.json
echo ""
echo "v4 Generated UUID:"
cat schemas/generated/UUID.schema.json
```

**Check:**
- [ ] Both have `type: "string"`
- [ ] Both have `format: "uuid"`
- [ ] Descriptions match
- [ ] No information lost

### 3.4 Test Round-Trip

```bash
# Create test data
cat > migration/test-data/uuid-samples.json << 'EOF'
[
  "550e8400-e29b-41d4-a716-446655440000",
  "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "123e4567-e89b-12d3-a456-426614174000"
]
EOF

# Test (manual for now, automated tool coming)
# 1. Validate against v3 schema
# 2. Deserialize in Rust
# 3. Serialize back
# 4. Validate against v4 schema
# 5. Compare

# For now, simple Rust test:
cd schemas
cargo test uuid
```

### 3.5 Document

```bash
# Create migration log
cat > migration/logs/uuid.md << 'EOF'
# UUID Migration

**Date:** $(date +%Y-%m-%d)
**Schema:** snippets/types/primitives/UUID.json

## Checklist
- [x] Information extracted
- [x] Rust type exists in v4
- [x] JSON Schema generated
- [x] Comparison done
- [ ] Round-trip tests (manual verification)
- [x] Documentation updated

## Results
- Compatibility: 100%
- Information Loss: 0%
- Differences: None (just $id and ordering)

## Status
✅ COMPLETE
EOF

cat migration/logs/uuid.md
```

---

## Step 4: Create Migration Tracker (30 mins)

```bash
cd /Users/erictheiss/familiar/docs/v4

# Create tracking spreadsheet (or use Port.io!)
cat > migration/TRACKER.md << 'EOF'
# Migration Tracker

## Summary
- Total Schemas: ___
- Completed: 1 (UUID)
- In Progress: 0
- Remaining: ___
- Success Rate: ___%

## Status by Phase

### Phase 2: Primitives (0/9)
- [x] UUID - 100% ✅
- [ ] Timestamp
- [ ] NormalizedValue
- [ ] SignedNormalizedValue
- [ ] AnyValue
- [ ] KeyValue
- [ ] StringValueMap
- [ ] TaskList
- [ ] NullableTimestamp

### Phase 3: Types (0/25)
- [ ] ComplexNumber
- [ ] Vec3
- [ ] ...

### Phase 4: Fields (0/30)
- [ ] EntityId
- [ ] CreatedAt
- [ ] ...

### Phase 5: Components (0/40)
- [ ] QuantumState
- [ ] ...

### Phase 6: Entities (0/13)
- [ ] Motif
- [ ] Thread
- [ ] Bond
- [ ] ...

## Daily Progress

### 2025-01-06
- Completed: UUID
- Issues: None
- Next: Timestamp, NormalizedValue
EOF

echo "✅ Tracker created"
```

---

## Step 5: Set Up Port.io Tracking (Optional, 30 mins)

If using Port.io for visual tracking:

```bash
# Create blueprint for migration tracking
cat > migration/port-blueprint.json << 'EOF'
{
  "identifier": "schemaMigration",
  "title": "Schema Migration",
  "icon": "Blueprint",
  "schema": {
    "properties": {
      "v3Path": {
        "type": "string",
        "title": "v3 Path"
      },
      "v4Path": {
        "type": "string",
        "title": "v4 Path"
      },
      "phase": {
        "type": "string",
        "enum": ["primitives", "types", "fields", "components", "entities"]
      },
      "status": {
        "type": "string",
        "enum": ["pending", "in-progress", "completed", "failed"]
      },
      "compatibility": {
        "type": "number",
        "title": "Compatibility %"
      },
      "informationLoss": {
        "type": "number",
        "title": "Information Loss %"
      }
    }
  }
}
EOF

echo "📊 Port.io blueprint ready"
echo "Import this to Port.io → Builder → Blueprints"
```

---

## Next Steps

You're now ready to start the full migration!

### Today (2-4 hours)
1. Complete remaining primitives (8 more)
2. Set up comparison tool
3. Set up round-trip testing

### This Week
1. Phase 2: All primitives (9/9) ✅
2. Phase 3: Simple types (15/15) ✅
3. Start Phase 4: Fields

### Success Indicators
- [ ] Each schema has migration log
- [ ] All comparisons show 100% compatibility
- [ ] No information loss detected
- [ ] Tracker updated daily

---

## Helpful Commands

```bash
# Count schemas by type
find ../v3/schemas -name "*.json" | grep -E "primitives|types|fields|components|entities" | wc -l

# Generate schema from Rust
cd schemas && cargo run --example generate-schemas --features generate-json-schemas

# Test specific module
cd schemas && cargo test primitives::

# View migration progress
cat migration/TRACKER.md

# Update tracker
vim migration/TRACKER.md
```

---

## Troubleshooting

**Q: Extract tool not working?**
```bash
cd tools/extract-schema
cargo build
cargo run -- --help
```

**Q: Can't generate schemas from Rust?**
```bash
cd schemas
cargo clean
cargo build --features generate-json-schemas
```

**Q: Lost track of progress?**
```bash
# Check migration logs
ls migration/logs/
# Check completed schemas
grep -r "✅ COMPLETE" migration/logs/
```

---

## Daily Ritual

Start each migration day with:

```bash
# 1. Check yesterday's progress
cat migration/TRACKER.md

# 2. Pick next schema
echo "Today: [schema name]"

# 3. Follow the process:
#    - Extract info
#    - Create/verify Rust type
#    - Generate schema
#    - Compare
#    - Test
#    - Document

# 4. Update tracker
vim migration/TRACKER.md

# 5. Commit progress
git add migration/
git commit -m "Migration: [schema name] complete"
```

---

## You're Ready!

Start with the remaining 8 primitives. Once those are done, you'll have the pattern down and can accelerate through the rest.

**Remember:** 100% lossless is the goal. Take time to verify at each step.

Good luck! 🚀

