# Familiar v4: Pure Rust Schema-First Architecture

**Philosophy:** Schemas as an immutable, versioned library that drives all code generation

## Key Concepts

1. **Schemas = Rust Library Crate** - Published, versioned, immutable
2. **Schema-First** - All code generated from schemas
3. **Template-Driven** - Copier templates consume schemas to generate projects
4. **Click-to-Build** - Single command builds entire solution from schemas + templates
5. **Bidirectional Validation** - Schemas validate both Rust and JSON

## Architecture

```
familiar-schemas (crate)          ← Immutable schema library
  ↓ (depends on)
  ├─→ Generated JSON Schemas      ← For non-Rust consumers
  ├─→ Template 1 (copier)         ← Generates microservice A
  ├─→ Template 2 (copier)         ← Generates microservice B
  └─→ Template N (copier)         ← Generates component N
```

## Directory Structure

```
docs/v4/
├── schemas/              ← Rust schema crate (source of truth)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── primitives/
│   │   ├── types/
│   │   ├── components/
│   │   └── entities/
│   └── generated/        ← JSON Schemas (auto-generated)
│
├── templates/            ← Copier templates
│   ├── microservice/
│   ├── entity-service/
│   ├── graphql-api/
│   └── rest-api/
│
├── examples/             ← Example usage
└── scripts/              ← Build scripts
```

## Workflow

1. **Develop schemas** (in schemas/ crate)
2. **Publish schema crate** (to registry or git tag)
3. **Templates depend on schema crate** (via Cargo.toml)
4. **Click to build** (script reads schemas + applies templates)
5. **Generated code uses schema crate** (imports types directly)

## Benefits

- ✅ Schemas are versioned, immutable
- ✅ Schema changes trigger rebuilds
- ✅ Multiple projects use same schemas
- ✅ Type safety across entire system
- ✅ Single source of truth
- ✅ Click-to-build entire solution

See `SCHEMA_LIBRARY_STRATEGY.md` for detailed architecture.

