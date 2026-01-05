# Familiar Schemas MCP

A Model Context Protocol (MCP) server for querying and analyzing the `familiar-schemas` registry. Designed for AI-assisted coding workflows (Cursor, Claude, etc.).

## Why Use This MCP?

When working with a schema-driven codebase, AI assistants need **fast, accurate context** about:
- What types exist and how they relate
- Where types are generated across languages
- What would break if a schema changes
- How to properly import and use types

**Without MCP**: AI guesses based on file contents, often missing relationships.
**With MCP**: AI queries the schema graph for precise, up-to-date answers.

---

## Quick Start

```bash
# The MCP is automatically available when working in familiar-core
# Just ask questions that trigger tool calls
```

---

## When to Use Each Tool

### 🔍 "What is this type?"

**Use Case**: You encounter `TenantId` in code and want to understand it.

```
Tool: get_type
Query: "What is TenantId?"

Returns: schema path, kind (primitive), fields, who references it
```

**Use Case**: Need the full JSON schema for documentation.

```
Tool: schema_raw
Query: "Show me the raw schema for AuthSession"
```

---

### 🔗 "What does this type depend on?"

**Use Case**: Understanding what types a schema needs.

```
Tool: get_refs
Query: "What does CourseResponse reference?"

Returns: list of $ref targets (MessageIntent, QueryType, etc.)
```

**Use Case**: Need ALL dependencies (transitive).

```
Tool: closure
Query: "Get all dependencies of FatesGate"
Args: direction="out"

Returns: full dependency tree with depth levels
```

---

### 📥 "What uses this type?"

**Use Case**: Finding all consumers before making a change.

```
Tool: get_dependents
Query: "What schemas depend on TenantId?"

Returns: 76 schemas that reference TenantId
```

**Use Case**: Full blast radius analysis.

```
Tool: impact
Query: "What would be affected if I change Email schema?"

Returns: direct dependents, transitive dependents, grouped by kind
```

---

### 🦀 "Where is this type in Rust?"

**Use Case**: Jump to generated Rust code.

```
Tool: get_artifacts
Query: "Where is TenantId generated?"
Args: lang="rust"

Returns: 
  file: familiar-core/src/contracts/generated.rs
  line: 1216
  type_kind: newtype
```

**Use Case**: See all types in a generated file.

```
Tool: file_artifacts
Query: "What types are in contracts/generated.rs?"

Returns: 418 artifacts with line numbers
```

---

### 🔄 "What needs regeneration?"

**Use Case**: Schema changed, what Rust files are affected?

```
Tool: affected_artifacts
Query: "What artifacts would change if I modify Moment schema?"

Returns: all artifacts for Moment + all dependent schemas' artifacts
```

**Use Case**: Find schemas without generated code.

```
Tool: schemas_without_artifacts
Query: "What schemas don't have TypeScript types?"
Args: lang="typescript"

Returns: list of schemas missing TS codegen
```

---

### 🔧 "How do I import this?"

**Use Case**: Generate correct import statements.

```
Tool: imports_for
Query: "What imports do I need for CourseResponse?"
Args: lang="rust"

Returns:
  use crate::contracts::CourseResponse;
  use crate::contracts::MessageIntent;
  use crate::contracts::QueryType;
  // ... all transitive deps
```

---

### 🔬 "Why is this type boxed?"

**Use Case**: Understanding recursive type handling.

```
Tool: explain_boxing
Query: "Why does Block need Box<T>?"

Returns:
  should_box: true
  reason: "Self-referential type"
  detail: "Block references itself in AccordionBlock variant"
```

---

### 📊 "What derives should this have?"

**Use Case**: Understanding trait implementations.

```
Tool: explain_derives
Query: "What derives does TenantId get and why?"

Returns:
  final_derives: [Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, Hash]
  reason: "High in-degree (76) and all fields support Eq/Hash"
```

---

### 🔄 "Are there any cycles?"

**Use Case**: Finding problematic circular references.

```
Tool: scc_report
Query: "Show all cycles in the schema graph"

Returns:
  cycles: [
    { members: [Block, AccordionBlock], suggested_box_edge: "Block -> AccordionBlock" }
  ]
```

---

### 🎯 "What are the most important types?"

**Use Case**: Understanding the schema architecture.

```
Tool: hub_report
Query: "What are the most referenced types?"
Args: top_n=10

Returns:
  1. familiar-worker (195 refs) - service node
  2. familiar-daemon (147 refs) - service node  
  3. TenantId (76 refs) - primitive
  4. NormalizedFloat (17 refs) - primitive
  ...
```

---

### 📋 "Show me the graph stats"

**Use Case**: Quick health check of schema registry.

```
Tool: status
Query: "What's the schema registry status?"

Returns:
  schema_count: 488
  edge_count: 892
  scc_count: 1
  artifact_count: 418
  bundle_hash: "75e557ae..."
```

**Use Case**: Detailed stats with artifact coverage.

```
Tool: graph_stats
Query: "Show full graph statistics"

Returns:
  schemas: { total: 488, edges: 892, sccs: 1 }
  artifacts: { 
    total: 418,
    by_language: [{ lang: "rust", count: 418, coverage: "85.7%" }]
  }
  indexes: { all O(1) lookups }
```

---

## Common AI Coding Workflows

### 1. Adding a New Field to a Schema

```
1. get_dependents("MySchema") - See what would be affected
2. impact("MySchema") - Full blast radius
3. affected_artifacts("MySchema") - What files need regeneration
4. After edit: cargo xtask codegen generate
```

### 2. Creating a New Schema

```
1. search("similar concept") - Find related schemas
2. get_type("RelatedSchema") - Understand the pattern
3. imports_for("RelatedSchema") - See required imports
4. list_kinds() - Choose appropriate x-familiar-kind
```

### 3. Debugging a Type Error

```
1. resolve("TypeName") - Find the canonical schema
2. get_artifacts("TypeName") - Where is it generated?
3. get_refs("TypeName") - What does it depend on?
4. explain_derives("TypeName") - What traits does it have?
```

### 4. Understanding Recursive Types

```
1. scc_report() - Find all cycles
2. explain_boxing("TypeName") - Why is it boxed?
3. closure("TypeName", direction="out") - Full dependency tree
```

### 5. Refactoring a Core Type

```
1. hub_report(top_n=20) - Is this a hub type?
2. impact("TypeName") - What's the blast radius?
3. services_for_schema("TypeName") - What services use it?
4. affected_artifacts("TypeName") - What files change?
```

---

## Performance Characteristics

All lookups are **O(1)** via HashMap indexes:
- Schema by ID: `O(1)`
- Schema by path: `O(1)`
- Schema by name: `O(1)`
- Artifacts by schema: `O(1)`
- Artifacts by file: `O(1)`
- Artifacts by language: `O(1)`

Graph traversals use petgraph:
- Transitive closure: `O(V+E)` BFS
- SCC detection: `O(V+E)` Kosaraju's algorithm
- Topological sort: `O(V+E)`

---

## Tool Reference

| Category | Tools |
|----------|-------|
| **Meta** | `status`, `resolve`, `graph_stats` |
| **Query** | `get_type`, `schema_raw`, `get_refs`, `get_dependents`, `search`, `list_kinds` |
| **Graph** | `closure`, `scc_report`, `hub_report`, `impact` |
| **Codegen** | `imports_for`, `explain_boxing`, `explain_derives`, `policy_for` |
| **Artifacts** | `get_artifacts`, `file_artifacts`, `affected_artifacts`, `artifact_dependencies`, `colocated_artifacts`, `schemas_without_artifacts` |
| **Lint** | `lint_unions`, `hoist_candidates`, `find_embedded_definitions` |
| **Services** | `services_for_schema` |

---

## Tips for AI Assistants

1. **Start with `resolve`** if the user mentions a type by name - it handles fuzzy matching.

2. **Use `get_type` before `schema_raw`** - it's faster and usually sufficient.

3. **Use `closure` sparingly** - it can return large results for hub types.

4. **Check `artifact_count` in `status`** - if 0, artifacts haven't been loaded yet.

5. **For refactoring, always check `impact`** - it shows the full blast radius.

6. **`hub_report` identifies architectural types** - changes to these need extra care.

7. **`explain_boxing` tells you WHY** - don't guess about recursive types.


