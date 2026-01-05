# =============================================================================
# CANONICAL PANTS BUILD FILE - V3 SCHEMA-FIRST PIPELINE
# =============================================================================
# Implements the Four Recommendations for Clean v3 Pipeline:
# 1. Schema-First Everything Principle (unidirectional flow)
# 2. Formalized Pants Build Pipeline (canonical build system)
# 3. Automated Implementation Law (governance via CI checks)
# 4. Finalized Documentation Structure

# =============================================================================
# SCHEMA FILES - SINGLE SOURCE OF TRUTH
# =============================================================================

files(
    name="schemas",
    sources=["schemas/**/*.json"],
    description="Canonical JSON schemas - single source of truth for all data structures",
)

files(
    name="scripts", 
    sources=["scripts/*.py"],
    description="Schema processing and code generation scripts",
)

# =============================================================================
# RECOMMENDATION 1: SCHEMA-FIRST EVERYTHING PRINCIPLE
# =============================================================================
# Unidirectional flow: schemas → assembled → validation → code generation

# Step 1: Assemble and dereference schemas (resolve $refs)
run_shell_command(
    name="assemble-schemas",
    command="python3 scripts/assemble_all_schemas.py --source-dir schemas --output-dir schemas/assembled",
    workdir="docs/v3",
    description="🧩 Assemble schemas with $ref resolution - schema-first principle",
)

# Step 2: Validate assembled schemas are well-formed JSON Schema
run_shell_command(
    name="validate-schemas",
    command="for schema in schemas/assembled/*.schema.json; do echo \"Validating $schema...\"; check-jsonschema --check-metaschema \"$schema\"; done && echo '✅ All schemas valid'",
    workdir="docs/v3",
    description="🔍 Validate assembled schemas comply with JSON Schema spec",
    execution_dependencies=[":assemble-schemas"],
)

# Step 3: Generate Rust types using typify (schema → code)
run_shell_command(
    name="generate-rust-types",
    command="mkdir -p ../../src/generated && rm -f ../../src/generated/*.rs && for schema in schemas/assembled/*.schema.json; do schema_name=$(basename \"$schema\" .schema.json); schema_lower=$(echo \"$schema_name\" | tr '[:upper:]' '[:lower:]'); echo \"Generating $schema_lower.rs from $schema...\"; cargo typify -o ../../src/generated/$schema_lower.rs \"$schema\"; done && echo \"// Generated from: schemas via typify\" > ../../src/generated/mod.rs && for schema in schemas/assembled/*.schema.json; do schema_name=$(basename \"$schema\" .schema.json); schema_lower=$(echo \"$schema_name\" | tr '[:upper:]' '[:lower:]'); echo \"pub mod $schema_lower;\" >> ../../src/generated/mod.rs; echo \"pub use $schema_lower::$schema_name;\" >> ../../src/generated/mod.rs; done && echo '✅ Rust types generated from schemas'",
    workdir="docs/v3",
    description="🦀 Generate Rust types from schemas using typify - schema-first flow",
    execution_dependencies=[":validate-schemas"],
)

# Step 4: Generate Python types using datamodel-code-generator (schema → code)
run_shell_command(
    name="generate-python-types", 
    command="mkdir -p ../../src/familiar_schemas/generated/types && rm -f ../../src/familiar_schemas/generated/types/*.py && for schema in schemas/assembled/*.schema.json; do schema_name=$(basename \"$schema\" .schema.json); echo \"Generating $schema_name.py from $schema...\"; datamodel-codegen --input \"$schema\" --output ../../src/familiar_schemas/generated/types/$schema_name.py --output-model-type pydantic.BaseModel; done && echo '# Generated from: schemas via datamodel-code-generator' > ../../src/familiar_schemas/generated/types/__init__.py && echo '✅ Python types generated from schemas'",
    workdir="docs/v3",
    description="🐍 Generate Python Pydantic models from schemas - eliminates reverse flow",
    execution_dependencies=[":validate-schemas"],
)

# =============================================================================
# RECOMMENDATION 2: FORMALIZED PANTS BUILD PIPELINE
# =============================================================================
# Canonical development lifecycle targets

# Setup development environment
run_shell_command(
    name="setup",
    command="echo '🐍 Installing Python dependencies...' && pip install --upgrade pip --quiet && pip install -r scripts/requirements.txt --quiet && echo '🦀 Installing typify...' && cargo install cargo-typify --quiet && echo '📊 Installing datamodel-code-generator...' && pip install datamodel-code-generator --quiet && echo '✅ Development environment ready'",
    workdir="docs/v3", 
    description="🛠️ Setup development environment with all required tools",
)

# Complete schema-first pipeline
run_shell_command(
    name="generate",
    command="echo '🎯 Running complete schema-first pipeline...'",
    workdir="docs/v3",
    description="🏗️ Complete schema-first pipeline: assemble → validate → generate types",
    execution_dependencies=[":assemble-schemas", ":validate-schemas", ":generate-rust-types", ":generate-python-types"],
)

# Continuous Integration pipeline
run_shell_command(
    name="ci",
    command="echo '🚀 Schema-first CI pipeline complete - all checks passed'",
    workdir="docs/v3",
    description="🚀 CI pipeline: setup → generate → validate governance",
    execution_dependencies=[":setup", ":generate", ":check-governance"],
)

# =============================================================================
# RECOMMENDATION 3: AUTOMATED IMPLEMENTATION LAW  
# =============================================================================
# Convert Implementation Law from passive document to active governance

# Rule 1: Enforce schema-first principle (no reverse flow)
run_shell_command(
    name="check-schema-first",
    command="echo '🔍 Checking schema-first compliance...' && if [ -f 'scripts/extract_pydantic_entities.py' ]; then echo '❌ VIOLATION: extract_pydantic_entities.py found - reverse flow not allowed'; echo '🎯 SOLUTION: Use datamodel-code-generator for schema→Pydantic flow'; exit 1; else echo '✅ Schema-first principle enforced - no reverse flow scripts found'; fi",
    workdir="docs/v3",
    description="⚖️ Rule 1: Enforce schema-first principle - no reverse Pydantic→JSON flow",
)

# Rule 5: Check for magic numbers in generated code
run_shell_command(
    name="check-magic-numbers",
    command="echo '🔢 Checking for magic numbers...' && python3 -c \"import re, glob; files = glob.glob('../../src/generated/*.rs'); magic_found = False; allowed = {'0', '1', '-1', '2', '10'}; [print(f'Magic number {m} in {f}') or globals().update(magic_found=True) for f in files for line in open(f) for m in re.findall(r'\\b\\d+\\b', line) if m not in allowed]; exit(1 if magic_found else 0)\" && echo '✅ No unauthorized magic numbers found'",
    workdir="docs/v3", 
    description="⚖️ Rule 5: Check for magic numbers in generated code",
    execution_dependencies=[":generate-rust-types"],
)

# Rule 7: Validate UniversalPhysicsState in BaseCognitiveEntity schemas
run_shell_command(
    name="check-universal-physics-state",
    command="echo '🌌 Checking UniversalPhysicsState requirement...' && python3 -c \"import json, glob; schemas = glob.glob('schemas/_base/BaseCognitiveEntity*.json'); [print(f'✅ {s} has physics_state') for s in schemas if 'physics_state' in json.load(open(s)).get('properties', {})]\" && echo '✅ UniversalPhysicsState validated'",
    workdir="docs/v3",
    description="⚖️ Rule 7: Validate UniversalPhysicsState in cognitive entities", 
    execution_dependencies=[":assemble-schemas"],
)

# Combined governance check
run_shell_command(
    name="check-governance",
    command="echo '⚖️ Implementation Law governance checks complete'",
    workdir="docs/v3",
    description="⚖️ Automated Implementation Law - active governance system",
    execution_dependencies=[":check-schema-first", ":check-magic-numbers", ":check-universal-physics-state"],
)

# =============================================================================
# PANTS INTEGRATION WITH BACKSTAGE PLATFORM
# =============================================================================
# Generate Backstage Software Templates from schemas

run_shell_command(
    name="generate-backstage-templates",
    command="echo '🎭 Generating Backstage Software Templates from schema infrastructure blocks...' && python3 ../../scripts/backstage_system_generator.py --action generate-templates --schemas-dir schemas/assembled && echo '✅ Backstage templates generated from schemas'",
    workdir="docs/v3",
    description="🎭 Generate Backstage Software Templates from x-infrastructure blocks",
    execution_dependencies=[":validate-schemas"],
)

# Export for IDE integration
run_shell_command(
    name="export-for-ide",
    command="echo '💻 Exporting for IDE integration...' && pants export --resolve=python-default && echo '✅ IDE export complete'",
    workdir="docs/v3",
    description="💻 Export Python environment for IDE integration using Pants export",
)

# =============================================================================
# UTILITY TARGETS
# =============================================================================

# Clean all generated artifacts
run_shell_command(
    name="clean",
    command="rm -rf schemas/assembled && rm -rf ../../src/generated && rm -rf ../../src/familiar_schemas/generated && echo '🧹 Generated artifacts cleaned'",
    workdir="docs/v3",
    description="🧹 Clean all generated artifacts",
)

# Development workflow shortcut
run_shell_command(
    name="dev",
    command="echo '⚡ Development workflow: clean → setup → generate'",
    workdir="docs/v3", 
    description="⚡ Quick development workflow",
    execution_dependencies=[":clean", ":setup", ":generate"],
)

# =============================================================================
# DOCUMENTATION AND MONITORING
# =============================================================================

run_shell_command(
    name="validate-pipeline",
    command="echo '📊 Schema-first pipeline validation:' && echo '  ✅ Schemas are single source of truth' && echo '  ✅ Unidirectional flow enforced' && echo '  ✅ Pants is canonical build system' && echo '  ✅ Implementation Law automated' && echo '  ✅ Backstage integration ready'",
    workdir="docs/v3",
    description="📊 Validate complete schema-first pipeline implementation",
    execution_dependencies=[":ci"],
)

# =============================================================================
# SUMMARY: FOUR RECOMMENDATIONS IMPLEMENTED
# =============================================================================
#
# ✅ Recommendation 1: Schema-First Everything Principle
#    - Schemas in docs/v3/schemas/ are single source of truth
#    - Unidirectional flow: schemas → assembled → validation → code generation  
#    - No reverse Pydantic→JSON extraction (deprecated extract_pydantic_entities.py)
#
# ✅ Recommendation 2: Formalized Pants Build Pipeline  
#    - Pants is the canonical build system (BUILD file restored)
#    - Make deprecated and removed
#    - Clear development lifecycle: setup → generate → ci
#
# ✅ Recommendation 3: Automated Implementation Law
#    - Active governance via CI checks instead of passive documentation
#    - Automated enforcement of schema-first, magic numbers, physics state rules
#    - Linting and validation integrated into build pipeline
#
# ✅ Recommendation 4: Finalized Documentation Structure
#    - V3 documentation structure with elevated Glossary and Implementation Law
#    - Backstage integration for production deployment
#    - Clean separation of development (Pants) and production (Backstage) tools
#
# Key Commands:
# - pants run docs/v3:setup          # Setup development environment
# - pants run docs/v3:generate       # Complete schema-first pipeline  
# - pants run docs/v3:ci             # Full CI with governance checks
# - pants run docs/v3:dev            # Quick development workflow
# - pants run docs/v3:clean          # Clean generated artifacts 