# Project Cleanup Plan

## Doctrine

```
familiar-schemas = Schema files + Schema validation/linting (no business logic)
familiar-core    = Graph + Codegen + Analysis + Business logic
xtask            = Essential build automation only
```

---

## Current State (Problems)

### familiar-schemas (TOO MUCH)
```
src/
├── mcp/              ← MOVE: graph + MCP server → familiar-core
│   ├── graph.rs      ← DUPLICATE: graph logic
│   ├── tools.rs      ← MOVE: MCP tools
│   └── mod.rs
├── codegen/          ← MOVE: codegen → familiar-core
│   ├── mod.rs
│   └── graph_passes.rs
├── lint/             ← KEEP: schema guardrails
│   └── mod.rs
├── bin/              ← DELETE MOST: migration tools done
│   ├── codegen_diff.rs      ← DELETE: migration tool
│   ├── codegen_test.rs      ← DELETE: migration tool
│   ├── extract_impls.rs     ← DELETE: migration tool
│   ├── inject_rust_meta.rs  ← DELETE: migration tool
│   ├── schema_fix.rs        ← DELETE: migration tool
│   ├── analyze_diffs.rs     ← DELETE: migration tool
│   ├── check_order.rs       ← DELETE: migration tool
│   ├── check.rs             ← DELETE: migration tool
│   ├── detailed.rs          ← DELETE: migration tool
│   ├── equiv_check.rs       ← DELETE: migration tool
│   ├── order.rs             ← DELETE: migration tool
│   ├── precise.rs           ← DELETE: migration tool
│   ├── semantic_test.rs     ← DELETE: migration tool
│   ├── show_examples.rs     ← DELETE: migration tool
│   ├── show_tuples.rs       ← DELETE: migration tool
│   ├── show_type.rs         ← DELETE: migration tool
│   ├── test_codegen.rs      ← DELETE: migration tool
│   ├── mcp.rs               ← MOVE: MCP server → familiar-core
│   ├── registry.rs          ← KEEP: schema registry CLI
│   ├── validator.rs         ← KEEP: schema validation
│   ├── export.rs            ← KEEP: schema export
│   ├── config.rs            ← KEEP: config utility
│   └── drift.rs             ← MOVE: drift → familiar-core
```

### familiar-core (ALSO HAS GRAPH)
```
src/schemas/
├── graph.rs          ← KEEP: canonical graph location
├── mod.rs
└── generated_version.rs

src/bin/
├── analyze_schema.rs      ← KEEP: analysis
├── drift_report.rs        ← KEEP: drift
├── extract_codegen_meta.rs ← DELETE: migration tool
├── protobuf_codegen.rs    ← KEEP: protobuf codegen
├── rust_type_fix.rs       ← DELETE: migration tool
└── schema_export.rs       ← KEEP: export
```

### xtask (TOO MANY COMMANDS)
```
Current commands:
  - Export        ← KEEP
  - Validate      ← KEEP
  - Drift         ← KEEP
  - Sync          ← DELETE: Windmill-specific
  - Update        ← KEEP
  - Graph         ← KEEP
  - ValidateGraph ← KEEP
  - AutoLink      ← DELETE: migration tool
  - Refactor      ← DELETE: migration tool
  - InjectCodegenMeta ← DELETE: migration tool
  - Migrate       ← DELETE: migration tool
  - LintFacets    ← KEEP
```

---

## Target State

### familiar-schemas (MINIMAL)
```
src/
├── lib.rs           ← Schema registry library
├── config.rs        ← Configuration
├── registry.rs      ← Registry operations
├── schema.rs        ← Schema types
├── version.rs       ← Versioning
├── checksum.rs      ← Integrity
├── compatibility.rs ← Compatibility checking
├── error.rs         ← Errors
├── lint/            ← Schema guardrails (KEPT)
│   └── mod.rs
└── bin/
    ├── registry.rs  ← Schema registry CLI
    ├── validator.rs ← Schema validation CLI
    └── export.rs    ← Schema export CLI

versions/            ← Schema files (unchanged)
```

### familiar-core (GRAPH + CODEGEN + ANALYSIS)
```
src/
├── schemas/
│   ├── graph.rs     ← CANONICAL graph (merge from familiar-schemas)
│   └── mod.rs
├── codegen/         ← MOVED from familiar-schemas
│   ├── mod.rs
│   └── graph_passes.rs
├── mcp/             ← MOVED from familiar-schemas
│   ├── mod.rs
│   └── tools.rs
├── analysis/        ← Analysis (existing)
└── bin/
    ├── analyze_schema.rs
    ├── drift_report.rs
    ├── schema_export.rs
    ├── protobuf_codegen.rs
    └── familiar-mcp.rs   ← MOVED from familiar-schemas
```

### xtask (ESSENTIAL ONLY)
```
Commands:
  - Export       ← Schema export to registry
  - Validate     ← Schema validation
  - Drift        ← Drift detection
  - Update       ← Update schema lock
  - Graph        ← Generate graph visualization
  - ValidateGraph← Validate graph connectivity
  - LintFacets   ← Lint schema facets
```

---

## Deletion List

### familiar-schemas/src/bin/ (18 files to DELETE)
1. `analyze_diffs.rs`
2. `check_order.rs`
3. `check.rs`
4. `codegen_diff.rs`
5. `codegen_test.rs`
6. `detailed.rs`
7. `equiv_check.rs`
8. `extract_impls.rs`
9. `inject_rust_meta.rs`
10. `order.rs`
11. `precise.rs`
12. `schema_fix.rs`
13. `semantic_test.rs`
14. `show_examples.rs`
15. `show_tuples.rs`
16. `show_type.rs`
17. `test_codegen.rs`
18. `drift.rs` (move to familiar-core)

### familiar-schemas/src/ (directories to MOVE)
1. `mcp/` → familiar-core
2. `codegen/` → familiar-core

### familiar-core/src/bin/ (2 files to DELETE)
1. `extract_codegen_meta.rs`
2. `rust_type_fix.rs`

### xtask commands (5 to DELETE)
1. `Sync` (Windmill-specific)
2. `AutoLink` (migration tool)
3. `Refactor` (migration tool)
4. `InjectCodegenMeta` (migration tool)
5. `Migrate` (migration tool)

---

## Implementation Order

### Phase 1: Delete Migration Binaries from familiar-schemas
```bash
# Delete 17 migration binaries
rm familiar-schemas/src/bin/{analyze_diffs,check_order,check,codegen_diff,codegen_test,detailed,equiv_check,extract_impls,inject_rust_meta,order,precise,schema_fix,semantic_test,show_examples,show_tuples,show_type,test_codegen}.rs
```

### Phase 2: Delete Migration Binaries from familiar-core
```bash
rm familiar-core/src/bin/{extract_codegen_meta,rust_type_fix}.rs
```

### Phase 3: Move codegen/ and mcp/ to familiar-core
1. Copy `familiar-schemas/src/codegen/` → `familiar-core/src/codegen/`
2. Copy `familiar-schemas/src/mcp/` → `familiar-core/src/mcp/`
3. Update imports to use familiar-core's graph
4. Delete from familiar-schemas

### Phase 4: Merge graph implementations
1. familiar-core's `src/schemas/graph.rs` is canonical
2. Migrate any missing features from familiar-schemas's graph
3. Update all references

### Phase 5: Delete xtask migration commands
1. Remove `Sync`, `AutoLink`, `Refactor`, `InjectCodegenMeta`, `Migrate` from enum
2. Remove handler functions
3. Remove supporting code

### Phase 6: Update Cargo.toml files
1. familiar-schemas: remove codegen/mcp dependencies
2. familiar-core: add MCP dependencies

### Phase 7: Final verification
```bash
cargo check --workspace
cargo xtask schemas validate-graph
cargo xtask schemas lint-facets
```

---

## Responsibility Matrix (Final)

| Component | Responsibility | Business Logic? |
|-----------|---------------|-----------------|
| familiar-schemas | Schema files, version history | NO |
| familiar-schemas | Schema validation/linting | NO (structural only) |
| familiar-schemas | Registry operations | NO (data access only) |
| familiar-core | Graph building/traversal | YES |
| familiar-core | Codegen from schemas | YES |
| familiar-core | MCP server | YES |
| familiar-core | Analysis tools | YES |
| familiar-core | Drift detection | YES |
| xtask | Build automation | NO (orchestration only) |

---

## Success Criteria

1. `familiar-schemas` has NO modules named `codegen` or `mcp`
2. `familiar-schemas` has ≤5 binaries (registry, validator, export, config, maybe drift)
3. `familiar-core` is the ONLY place with graph implementation
4. `xtask` has ≤7 schema commands
5. `cargo check --workspace` passes
6. `cargo xtask schemas validate-graph` passes
7. MCP server runs from familiar-core





