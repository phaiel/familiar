# Schema-First Architecture

## Single Source of Truth

All data types in the Familiar system originate from Rust structs in `familiar-core`. This ensures consistency across all services regardless of language.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RUST (familiar-core)                              │
│                                                                      │
│  src/types/*.rs        - Domain types with derives                   │
│    #[derive(Serialize, Deserialize, TS, JsonSchema)]                │
│    #[ts(export)]                                                     │
└───────────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
          ┌─────────────────┐         ┌─────────────────┐
          │   ts-rs         │         │   schemars      │
          │   (bindings/)   │         │   (schemas/)    │
          └────────┬────────┘         └────────┬────────┘
                   │                           │
                   ▼                           ▼
┌──────────────────────────────┐   ┌──────────────────────────────┐
│     TYPESCRIPT CONSUMERS      │   │      PYTHON CONSUMERS         │
│                              │   │                              │
│  familiar-ui/types/          │   │  generated_pydantic/         │
│    Re-exports from bindings  │   │    datamodel-codegen output  │
│                              │   │                              │
│  windmill/scripts/types.ts   │   │  windmill/scripts/agentic/   │
│    Generated from bindings   │   │    Imports from generated    │
└──────────────────────────────┘   └──────────────────────────────┘
```

## Why Schema-First?

1. **Type Safety**: Changes in Rust automatically propagate to consumers
2. **Consistency**: No manual synchronization of type definitions
3. **Documentation**: Rust doc comments become TypeScript/Python docstrings
4. **Validation**: Pydantic models validate at runtime

## Regeneration Steps

### TypeScript Bindings (ts-rs)

```bash
cd familiar-core

# Run tests which triggers ts-rs export
cargo test --lib

# Bindings are written to familiar-core/bindings/
ls bindings/*.ts
```

### Windmill TypeScript Types

```bash
cd services/windmill/scripts

# Regenerate types.ts from bindings
./generate_types.sh

# Verify the output
head -50 types.ts
```

### Python Pydantic Models

```bash
cd familiar-core

# Generate JSON schemas via schemars
cargo run --example generate_json_schemas

# Generate Pydantic from JSON schemas
pip install datamodel-code-generator
datamodel-codegen \
  --input schemas/agentic/ \
  --output generated_pydantic/agentic/ \
  --input-file-type jsonschema
```

### familiar-ui Types

The UI imports directly from `familiar-core/bindings` via path alias:

```typescript
// tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@familiar-core/*": ["../../familiar-core/bindings/*"]
    }
  }
}
```

```typescript
// types/index.ts - re-exports for convenience
export type { AgenticFlowResponse } from "@familiar-core/AgenticFlowResponse"
export type { UIToolCall } from "@familiar-core/UIToolCall"
```

## Anti-Drift Checklist

Before merging PRs that modify types:

### 1. Verify Rust Types

```bash
cd familiar-core
cargo check
cargo test --lib  # Regenerates bindings
```

### 2. Verify TypeScript

```bash
# Check familiar-ui compiles
cd services/familiar-ui
npm run build

# Check windmill types
cd ../windmill/scripts
./generate_types.sh
```

### 3. Verify Python

```bash
# Run pydantic validation tests
cd familiar-core
python -m pytest tests/test_contracts.py
```

### 4. Schema Checksum (CI)

Add to CI pipeline:

```yaml
# .github/workflows/schema-check.yml
- name: Check schema drift
  run: |
    # Generate fresh bindings
    cargo test --lib
    
    # Check for uncommitted changes
    git diff --exit-code bindings/
```

## What NOT to Do

### TypeScript

```typescript
// BAD: Custom type definition
interface MyMessage {
  id: string;
  content: string;
}

// GOOD: Import from bindings
import type { UIChannelMessage } from "@familiar-core/UIChannelMessage"
```

### Python

```python
# BAD: Custom Pydantic model
class AgentState(BaseModel):
    tenant_id: str
    # ... manual definition

# GOOD: Import from generated
from generated_pydantic.agentic import AgentState
```

### Windmill Scripts

```typescript
// BAD: Inline type in script
type Purpose = "LOG" | "QUERY";

// GOOD: Import from types.ts
import type { MessageIntent } from "./types.ts";
```

## Adding New Types

1. **Define in Rust** (`familiar-core/src/types/`)
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
   #[ts(export)]
   pub struct NewType {
       /// Field description becomes doc comment
       pub field: String,
   }
   ```

2. **Add to module** (`familiar-core/src/types/mod.rs`)
   ```rust
   pub mod new_type;
   pub use new_type::NewType;
   ```

3. **Regenerate bindings**
   ```bash
   cargo test --lib
   ```

4. **Update consumers**
   - Windmill: Run `./generate_types.sh`
   - Python: Regenerate Pydantic models
   - UI: Import from `@familiar-core`

5. **Test all consumers**
   ```bash
   # UI
   cd services/familiar-ui && npm run build
   
   # Python
   cd familiar-core && python -m pytest
   ```

## Fallback Pattern

When generated imports aren't available (e.g., in Windmill sandbox):

```python
try:
    from generated_pydantic.agentic import AgentState
except ImportError:
    # Fallback MUST match generated schema exactly
    class AgentState(BaseModel):
        """MUST match generated schema"""
        tenant_id: str
        current_speaker: Optional[str] = None
        # ... exact same fields
```

## Troubleshooting

### "Type not found in bindings"

1. Check Rust type has `#[ts(export)]` derive
2. Ensure type is `pub use`d in module
3. Run `cargo test --lib` to regenerate

### "Pydantic validation error"

1. Ensure fallback model matches generated schema
2. Check field order and optionality
3. Run schema generator to update

### "TypeScript import error"

1. Check tsconfig paths are correct
2. Verify bindings file exists
3. Restart TypeScript language server

