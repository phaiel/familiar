# Schema Library Strategy: Immutable, Versioned, Template-Driven

**Question:** How to create an uneditable schema library that triggers builds through a schema-to-code pipeline with templates?

**Answer:** Publish schemas as a Rust crate with semantic versioning, generate JSON Schemas as artifacts, and use Copier templates that consume both.

---

## TL;DR

```
┌─────────────────────────────────────────────────────────────┐
│ familiar-schemas v1.2.3 (Rust crate)                        │
│ - Published to crates.io or internal registry               │
│ - Immutable once published                                  │
│ - Contains: Rust types with JsonSchema derives             │
│ - Generates: JSON Schemas as build artifacts               │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ├─→ JSON Schemas (for non-Rust templates)
                   │
                   ├─→ Copier Template: microservice
                   │   - Reads schemas (Rust or JSON)
                   │   - Generates: service scaffold
                   │
                   ├─→ Copier Template: entity-service
                   │   - Reads specific entities
                   │   - Generates: CRUD API
                   │
                   └─→ Copier Template: graphql-gateway
                       - Reads all entities
                       - Generates: GraphQL schema + resolvers
```

**One command builds everything:**
```bash
./build-solution.sh --schemas familiar-schemas@1.2.3
```

---

## Architecture

### 1. Schemas as a Rust Crate (Immutable Library)

#### Structure

```
familiar-schemas/          ← Git repository
├── Cargo.toml             ← Crate metadata, version
├── README.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs             ← Re-exports all schemas
│   ├── primitives/
│   │   ├── mod.rs
│   │   ├── uuid.rs
│   │   ├── timestamp.rs
│   │   └── normalized_value.rs
│   ├── types/
│   │   ├── mod.rs
│   │   ├── complex_number.rs
│   │   └── vec3.rs
│   ├── components/
│   │   ├── mod.rs
│   │   ├── quantum_state.rs
│   │   └── bond_content.rs
│   └── entities/
│       ├── mod.rs
│       ├── motif.rs
│       ├── thread.rs
│       └── bond.rs
├── generated/             ← JSON Schemas (built by CI)
│   ├── UUID.schema.json
│   ├── Motif.schema.json
│   └── ...
├── build.rs               ← Generates JSON Schemas
└── tests/
    └── schema_validation_tests.rs
```

#### Cargo.toml

```toml
[package]
name = "familiar-schemas"
version = "1.2.3"
edition = "2021"
description = "Familiar schema library - immutable, versioned schemas"
license = "MIT"
repository = "https://github.com/your-org/familiar-schemas"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "0.8", features = ["preserve_order"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
jsonschema = "0.17"

[features]
default = []
generate-json-schemas = []  # Enable to generate JSON Schemas

[[example]]
name = "generate-schemas"
required-features = ["generate-json-schemas"]
```

#### lib.rs

```rust
//! Familiar Schema Library
//! 
//! This crate contains all canonical schema definitions for the Familiar system.
//! 
//! # Usage
//! 
//! ```rust
//! use familiar_schemas::entities::Motif;
//! use familiar_schemas::primitives::UUID;
//! 
//! let motif = Motif {
//!     entity_id: UUID::new(),
//!     // ...
//! };
//! ```
//! 
//! # Versioning
//! 
//! This crate follows semantic versioning:
//! - Major: Breaking schema changes
//! - Minor: New schemas (backwards compatible)
//! - Patch: Bug fixes, documentation

pub mod primitives;
pub mod types;
pub mod components;
pub mod entities;

// Re-export common types
pub use primitives::{UUID, Timestamp, NormalizedValue};
pub use entities::{Motif, Thread, Bond};

/// Schema library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Generate all JSON Schemas
#[cfg(feature = "generate-json-schemas")]
pub fn generate_all_json_schemas(output_dir: &std::path::Path) -> std::io::Result<()> {
    use schemars::schema_for;
    use std::fs;
    
    fs::create_dir_all(output_dir)?;
    
    // Generate for all types
    generate_schema::<primitives::UUID>("UUID", output_dir)?;
    generate_schema::<entities::Motif>("Motif", output_dir)?;
    // ... etc
    
    Ok(())
}

#[cfg(feature = "generate-json-schemas")]
fn generate_schema<T: schemars::JsonSchema>(name: &str, dir: &std::path::Path) -> std::io::Result<()> {
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::write(dir.join(format!("{}.schema.json", name)), json)?;
    Ok(())
}
```

#### build.rs (Optional - Generate JSON Schemas on Build)

```rust
fn main() {
    // Only generate if feature is enabled
    if cfg!(feature = "generate-json-schemas") {
        // Generate JSON Schemas during build
        let output_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("generated");
        
        std::fs::create_dir_all(&output_dir).unwrap();
        
        // This would call generate_all_json_schemas
        println!("cargo:rerun-if-changed=src/");
    }
}
```

---

### 2. Publishing & Versioning Strategy

#### Option A: crates.io (Public)

```bash
# Publish to crates.io
cargo publish

# Other projects depend on it
[dependencies]
familiar-schemas = "1.2.3"
```

**Pros:**
- ✅ Standard Rust ecosystem
- ✅ Automatic versioning
- ✅ Built-in dependency management

**Cons:**
- ⚠️ Public (unless using crates.io for private crates)

#### Option B: Private Registry (Cloudsmith, JFrog, etc.)

```toml
# In .cargo/config.toml
[registries.familiar]
index = "https://registry.familiar.dev/git/index"

# Publish
cargo publish --registry familiar

# Use
[dependencies]
familiar-schemas = { version = "1.2.3", registry = "familiar" }
```

**Pros:**
- ✅ Private
- ✅ Full control
- ✅ Same workflow as crates.io

#### Option C: Git Dependencies (Simplest)

```toml
[dependencies]
familiar-schemas = { git = "https://github.com/your-org/familiar-schemas", tag = "v1.2.3" }
```

**Pros:**
- ✅ Simple, no registry needed
- ✅ Private (if private repo)
- ✅ Tag-based versioning

**Cons:**
- ⚠️ Slower builds (git clone)

#### Recommended: Start with Git, Move to Registry

```toml
# Development
familiar-schemas = { git = "https://github.com/your-org/familiar-schemas", tag = "v1.2.3" }

# Production (when mature)
familiar-schemas = { version = "1.2.3", registry = "familiar" }
```

---

### 3. Template Strategy (Copier Templates)

#### Copier Template Structure

```
templates/microservice-template/
├── copier.yml             ← Template configuration
├── template/              ← Template files
│   ├── Cargo.toml.jinja
│   ├── src/
│   │   ├── main.rs.jinja
│   │   ├── handlers/
│   │   │   └── {{entity_name}}.rs.jinja
│   │   └── models/
│   │       └── mod.rs.jinja
│   ├── Dockerfile.jinja
│   └── README.md.jinja
├── schemas/               ← Schema extraction scripts
│   └── extract.py
└── hooks/
    ├── post_gen_project.py
    └── pre_gen_project.py
```

#### copier.yml

```yaml
# Template metadata
_metadata:
  name: "Familiar Microservice Template"
  description: "Generate a microservice from Familiar schemas"

# Questions
project_name:
  type: str
  help: "Name of your microservice"
  default: "my-service"

schema_version:
  type: str
  help: "Version of familiar-schemas to use"
  default: "1.2.3"

entities:
  type: yaml
  help: "Which entities to include? (e.g., ['Motif', 'Thread'])"
  default: ["Motif"]

include_graphql:
  type: bool
  help: "Include GraphQL API?"
  default: true

include_rest:
  type: bool
  help: "Include REST API?"
  default: true

# Tasks (run after generation)
_tasks:
  - "cargo build"
  - "cargo test"
```

#### Template File: Cargo.toml.jinja

```toml
[package]
name = "{{ project_name }}"
version = "0.1.0"
edition = "2021"

[dependencies]
# Schema library - IMMUTABLE dependency
familiar-schemas = { git = "https://github.com/your-org/familiar-schemas", tag = "v{{ schema_version }}" }

# Other dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

{% if include_graphql %}
async-graphql = "5.0"
{% endif %}

{% if include_rest %}
axum = "0.6"
{% endif %}
```

#### Template File: handlers/entity.rs.jinja

```rust
//! Handler for {{ entity_name }} entity
//! 
//! This file is generated from the familiar-schemas crate.
//! Schema version: {{ schema_version }}

use familiar_schemas::entities::{{ entity_name }};
use axum::{Json, extract::Path};
use uuid::Uuid;

/// Get {{ entity_name }} by ID
pub async fn get_{{ entity_name | lower }}(
    Path(id): Path<Uuid>
) -> Result<Json<{{ entity_name }}>, StatusCode> {
    // Implementation...
    todo!()
}

/// Create new {{ entity_name }}
pub async fn create_{{ entity_name | lower }}(
    Json(payload): Json<{{ entity_name }}>
) -> Result<Json<{{ entity_name }}>, StatusCode> {
    // Validation happens automatically via serde
    // Type safety from schema crate
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use familiar_schemas::testing::create_test_{{ entity_name | lower }};
    
    #[tokio::test]
    async fn test_create_{{ entity_name | lower }}() {
        let {{ entity_name | lower }} = create_test_{{ entity_name | lower }}();
        // Test implementation
    }
}
```

---

### 4. Click-to-Build Solution

#### build-solution.sh

```bash
#!/bin/bash
# Build entire solution from schemas + templates

set -e

SCHEMA_VERSION=${1:-"1.2.3"}
OUTPUT_DIR=${2:-"./generated"}

echo "🚀 Building Familiar solution from schemas v${SCHEMA_VERSION}"

# 1. Clone/update schema library
echo "📦 Fetching schemas..."
git clone --branch v${SCHEMA_VERSION} \
  https://github.com/your-org/familiar-schemas \
  /tmp/familiar-schemas || true

# 2. Generate JSON Schemas (for non-Rust templates)
echo "🔧 Generating JSON Schemas..."
cd /tmp/familiar-schemas
cargo build --features generate-json-schemas
cargo run --example generate-schemas --features generate-json-schemas

# 3. Apply templates
echo "🎨 Applying templates..."

# Template 1: Motif Service
copier copy \
  templates/microservice-template \
  ${OUTPUT_DIR}/motif-service \
  --data schema_version=${SCHEMA_VERSION} \
  --data entities='["Motif"]' \
  --data project_name=motif-service \
  --force

# Template 2: Thread Service
copier copy \
  templates/microservice-template \
  ${OUTPUT_DIR}/thread-service \
  --data schema_version=${SCHEMA_VERSION} \
  --data entities='["Thread"]' \
  --data project_name=thread-service \
  --force

# Template 3: GraphQL Gateway
copier copy \
  templates/graphql-gateway-template \
  ${OUTPUT_DIR}/graphql-gateway \
  --data schema_version=${SCHEMA_VERSION} \
  --force

# 4. Build all services
echo "🔨 Building all services..."
for service in ${OUTPUT_DIR}/*; do
  if [ -f "$service/Cargo.toml" ]; then
    echo "Building $(basename $service)..."
    cd $service && cargo build && cd -
  fi
done

echo "✅ Solution built successfully!"
echo ""
echo "Generated services:"
ls -la ${OUTPUT_DIR}/
```

#### Usage

```bash
# Build with specific schema version
./build-solution.sh 1.2.3

# Build with latest
./build-solution.sh latest

# Custom output directory
./build-solution.sh 1.2.3 ./my-solution
```

---

### 5. Template Discovery & Introspection

Templates can introspect the schema crate to discover what to generate:

#### hooks/pre_gen_project.py

```python
#!/usr/bin/env python3
"""
Pre-generation hook: Discover entities and types from schema crate
"""
import subprocess
import json
from pathlib import Path

def get_schema_metadata(version: str) -> dict:
    """Get metadata about available schemas"""
    
    # Option 1: Read from generated JSON Schemas
    schemas_dir = Path("/tmp/familiar-schemas/generated")
    entities = []
    
    for schema_file in schemas_dir.glob("*.schema.json"):
        with open(schema_file) as f:
            schema = json.load(f)
            if "Entity" in schema.get("title", ""):
                entities.append(schema_file.stem)
    
    # Option 2: Query the Rust crate directly
    # (Would require a CLI tool in the schema crate)
    
    return {
        "version": version,
        "entities": entities,
        "components": [...],
        "types": [...]
    }

def main():
    schema_version = "{{ schema_version }}"
    metadata = get_schema_metadata(schema_version)
    
    print(f"📊 Schema metadata for v{schema_version}:")
    print(f"  Entities: {', '.join(metadata['entities'])}")
    
    # Could prompt user to select entities
    # Or auto-generate for all entities

if __name__ == "__main__":
    main()
```

---

### 6. Schema Change Triggers

#### CI/CD Integration

```yaml
# .github/workflows/schema-publish.yml
name: Publish Schemas

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Publish to registry
        run: cargo publish --registry familiar
      
      - name: Generate JSON Schemas
        run: |
          cargo build --features generate-json-schemas
          cargo run --example generate-schemas
      
      - name: Upload JSON Schema artifacts
        uses: actions/upload-artifact@v2
        with:
          name: json-schemas
          path: generated/*.schema.json
      
      - name: Trigger dependent builds
        run: |
          # Trigger rebuilds of all services using this schema version
          curl -X POST https://api.github.com/repos/org/service1/dispatches \
            -H "Authorization: token ${{ secrets.GH_TOKEN }}" \
            -d '{"event_type":"schema_update","client_payload":{"version":"${{ github.ref_name }}"}}'
```

---

### 7. Consuming Schemas in Generated Code

#### In Generated Service

```rust
// src/main.rs (generated by template)
use familiar_schemas::entities::{Motif, Thread};
use familiar_schemas::primitives::{UUID, Timestamp};

// Types are guaranteed to match schema
// No code generation needed - direct dependency!

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/motif/:id", get(handlers::get_motif))
        .route("/motif", post(handlers::create_motif));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Handlers use schema types directly
mod handlers {
    use super::*;
    
    pub async fn get_motif(
        Path(id): Path<UUID>
    ) -> Result<Json<Motif>, StatusCode> {
        // Schema types guarantee correctness
        let motif = db::fetch_motif(id).await?;
        Ok(Json(motif))
    }
}
```

---

## Benefits of This Approach

### 1. Immutable Schemas ✅

```
Published schema crate v1.2.3 NEVER changes
  ↓
Services depend on specific version
  ↓
Schema changes = new version (v1.2.4)
  ↓
Services choose when to upgrade
```

### 2. Single Source of Truth ✅

```
Rust types in familiar-schemas crate
  ↓
Everything else derives from this
  ↓
No sync issues, no drift
```

### 3. Type Safety Across System ✅

```
All services import same schema crate
  ↓
Types are identical across boundaries
  ↓
Compile-time verification of compatibility
```

### 4. Template-Driven Generation ✅

```
Templates read schemas (Rust or JSON)
  ↓
Generate: services, APIs, clients, docs
  ↓
One click builds entire solution
```

### 5. Bidirectional Validation ✅

```
Rust types → JSON Schemas → Validate data → Back to Rust
  ↓
Every layer validated
  ↓
100% correctness
```

---

## Comparison: Schema Library Approaches

| Approach | Source Format | Distribution | Generation | Type Safety |
|----------|--------------|--------------|------------|-------------|
| **JSON Schema Files** | JSON | Git/npm | Code from JSON | Runtime only |
| **Protobuf** | .proto | Git/buf | Code from proto | Compile-time |
| **OpenAPI** | YAML | Git/npm | Code from spec | Runtime mostly |
| **Rust Crate** | Rust code | Git/registry | JSON from Rust | **Compile-time** ✅ |

**Rust crate wins on type safety and correctness.**

---

## Migration Path

### Week 1: Create Schema Crate

```bash
cd docs/v4/schemas
cargo init --lib
# Move types from v3 to Rust
# Add JsonSchema derives
cargo test
```

### Week 2: Publish Schema Crate

```bash
# Tag first version
git tag v0.1.0
git push --tags

# Publish
cargo publish --registry familiar
```

### Week 3: Create First Template

```bash
# Create microservice template
copier copy gh:copier-org/copier-templates-extensions \
  templates/microservice-template

# Configure to use schema crate
# Test generation
```

### Week 4: Build Solution

```bash
# Create build script
./build-solution.sh 0.1.0

# Verify all services build
# Test integration
```

---

## Recommended Tools

### Schema Crate
- **Language:** Rust
- **Validation:** schemars + jsonschema
- **Publishing:** Git tags or internal registry

### Templates
- **Tool:** Copier (Python-based, Jinja2 templates)
- **Alternative:** Cookiecutter, cargo-generate
- **Why Copier:** Updates existing projects, good for schema evolution

### Build Orchestration
- **Simple:** Bash script
- **Advanced:** Earthly, Bazel, or cargo-make
- **CI/CD:** GitHub Actions, GitLab CI

---

## Answer to Your Question

> "I need a schema library that is uneditable and can be used to trigger builds through some type of schema-to-code pipeline."

### Solution:

1. **Uneditable Schema Library:**
   - Publish `familiar-schemas` as a versioned Rust crate
   - Once published, that version is immutable
   - Changes = new version

2. **Trigger Builds:**
   - Services depend on specific schema version
   - Schema updates trigger CI/CD
   - Or: manual "click to build" script

3. **Schema-to-Code Pipeline:**
   - Templates (Copier) read schemas
   - Generate: services, APIs, docs
   - Services import schema crate directly (no codegen for Rust)

4. **Template-Driven:**
   - Multiple Copier templates
   - Each template consumes schemas
   - One command applies all templates

5. **Click to Build:**
   ```bash
   ./build-solution.sh v1.2.3
   ```
   This:
   - Fetches schema crate
   - Generates JSON Schemas
   - Applies all templates
   - Builds all services

**Result:** Immutable, versioned schemas that drive template-based code generation with a single command.

---

## Next Steps

1. **Create schema crate structure** (in v4/schemas/)
2. **Migrate types from v3 to Rust** (with JsonSchema derives)
3. **Create first Copier template** (microservice template)
4. **Create build-solution.sh script**
5. **Test end-to-end workflow**

Want me to create the actual schema crate structure in v4?

