#!/usr/bin/env python3
"""
Familiar v3: Multi-Crate Rust Generation Pipeline

Uses the pre-assembled schemas (which have resolved references) and generates
a proper multi-crate Rust workspace with correct dependency management.

🎯 Purpose: Schema-first Rust generation with proper multi-crate workspace
📁 Input: docs/v3/schemas/assembled/*.schema.json
📁 Output: Multi-crate Rust workspace with proper dependencies
🔧 Tool: cargo typify
⚡ Feature: Eliminates duplication through proper Rust architecture
"""

import subprocess
import os
import sys
from pathlib import Path
import json
import tempfile
import shutil
import re
from typing import Dict, Any, List

def clean_schema_for_typify(schema_data: Dict[str, Any]) -> Dict[str, Any]:
    """Remove extensions that cause typify to crash."""
    cleaned = json.loads(json.dumps(schema_data))
    
    extensions_to_remove = [
        'x-infrastructure', 'category', 'source_file', 'schema_version',
        'physics_properties', 'x-python-type', 'x-typescript-type',
        'x-java-type', 'x-go-type', 'ui_label', 'format'
    ]

    def clean_nested(obj: Any) -> Any:
        if isinstance(obj, dict):
            # Remove typify-incompatible extensions
            for ext in extensions_to_remove:
                if ext in obj:
                    del obj[ext]
            
            # Process x-rust-type hints
            if 'x-rust-type' in obj:
                rust_type = obj['x-rust-type']
                del obj['x-rust-type']
                
                # Transform schema based on rust type hints
                if rust_type == 'f64':
                    obj['type'] = 'number'
                elif rust_type.startswith('[f64;'):
                    obj['type'] = 'array'
                    obj['items'] = {'type': 'number'}
                    if '; 6]' in rust_type:
                        obj['minItems'] = 6
                        obj['maxItems'] = 6

            # Recursively clean nested objects
            for key, value in list(obj.items()):
                obj[key] = clean_nested(value)

            return obj
        elif isinstance(obj, list):
            return [clean_nested(item) for item in obj]
        return obj

    return clean_nested(cleaned)

def categorize_schema(schema_name: str) -> str:
    """Determine which crate a schema belongs to."""
    
    # Core shared types - foundational types used everywhere
    if schema_name in ['EntityType', 'RelationshipType', 'ThreadType', 'BondState', 
                       'ThreadState', 'Status', 'AccessType', 'UUID', 'Timestamp']:
        return 'types'
    
    # More types - anything ending in Type, State, or Status
    if any(schema_name.endswith(suffix) for suffix in ['Type', 'State', 'Status']):
        return 'types'
    
    # Physics types - physics-related shared types
    if any(pattern in schema_name for pattern in [
        'Vec3', 'Vec6', 'ComplexNumber', 'NormalizedValue', 'SignedNormalizedValue',
        'ExplorationBias', 'BondDampingFactor', 'SocialEnergyFactor', 
        'ConsolidationRateModifier', 'EmotionalVolatility', 'PhysicsProfile',
        'EmotionalValence', 'Physics', 'Quantum', 'BasePhysics'
    ]):
        return 'physics'
    
    # Component schemas - ECS components and content types
    if any(schema_name.endswith(suffix) for suffix in [
        'Content', 'Permissions', 'Config', 'Details', 'Members', 'Identity',
        'Tension', 'Reason'
    ]) or 'Component' in schema_name:
        return 'components'
    
    # Table schemas - database tables and logs
    if any(pattern in schema_name for pattern in [
        '_log', 'Table', '_state_log'
    ]) or schema_name.startswith('Base') and 'Table' in schema_name:
        return 'tables'
    
    # Laws - physics laws and dynamics
    if any(pattern in schema_name for pattern in [
        'Law', 'Dynamics', 'Collapse', 'Consolidation', 'Decay'
    ]) or schema_name.startswith('Base') and 'Law' in schema_name:
        return 'laws'
    
    # Workflows
    if any(pattern in schema_name for pattern in [
        'Workflow', 'Ingestion'
    ]):
        return 'workflows'
    
    # Taxonomy
    if any(pattern in schema_name for pattern in [
        'Taxonomy', 'Level', 'Node'
    ]) and 'Profile' in schema_name:
        return 'taxonomy'
    
    # Entity schemas - EXACT matches only for main business objects
    if schema_name in [
        'Bond', 'Thread', 'Moment', 'Intent', 'Focus', 'Filament', 
        'Stitch', 'Motif', 'Course', 'Shuttle', 'Tenant', 'PersonThread',
        'GenericThread'
    ]:
        return 'entities'
    
    # Infrastructure schemas
    if any(pattern in schema_name for pattern in [
        'Service', 'Stack', 'Infrastructure', 'Database', 'Api', 'Workflow',
        'Endpoint'
    ]) or schema_name.startswith('Base') and any(infra in schema_name for infra in ['Service', 'Api', 'Infrastructure']):
        return 'infrastructure'
    
    # Default to common types
    return 'common'

def generate_rust_code(schema_name: str, cleaned_schema: Dict[str, Any], output_file: Path) -> bool:
    """Generate Rust code from a cleaned schema."""
    print(f"  ⚡ Generating {output_file.name}...")
    
    # Ensure parent directory exists
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    # Create temporary file with cleaned schema
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(cleaned_schema, f, indent=2)
        temp_schema_path = f.name
    
    try:
        subprocess.run(
            ["cargo", "typify", "-o", str(output_file), temp_schema_path],
            capture_output=True, text=True, check=True
        )
        return True
    except subprocess.CalledProcessError as e:
        print(f"    ❌ Typify failed for {schema_name}: {e.stderr}")
        return False
    except FileNotFoundError:
        print("    ❌ cargo typify not found. Install with: cargo install cargo-typify")
        sys.exit(1)
    finally:
        os.unlink(temp_schema_path)

def create_crate(crate_dir: Path, crate_name: str, dependencies: List[str], modules: List[str]):
    """Create a Rust crate with proper Cargo.toml and lib.rs."""
    src_dir = crate_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    # Create Cargo.toml
    deps_section = "\n".join([
        'serde = { version = "1.0", features = ["derive"] }',
        'serde_json = "1.0"',
        'chrono = { version = "0.4", features = ["serde"] }',
        'uuid = { version = "1.0", features = ["serde", "v4"] }',
        'indexmap = { version = "2.0", features = ["serde"] }'
    ])
    
    # Add dependencies on other familiar crates
    for dep in dependencies:
        deps_section += f'\nfamiliar_{dep} = {{ path = "../{dep}" }}'
    
    cargo_toml = f"""[package]
name = "familiar_{crate_name}"
version = "0.1.0"
edition = "2021"
description = "Generated Rust types for the '{crate_name}' category."

[dependencies]
{deps_section}
"""
    
    (crate_dir / "Cargo.toml").write_text(cargo_toml)

    # Create lib.rs with module declarations
    lib_content = [f"//! Generated Rust types for the '{crate_name}' category.\n"]
    for module in sorted(modules):
        lib_content.append(f"pub mod {module};")
    
    (src_dir / "lib.rs").write_text("\n".join(lib_content))
    print(f"    📦 Created crate 'familiar_{crate_name}' with {len(modules)} modules")

def main():
    """Main execution function."""
    print("🦀 Multi-Crate Rust Generation Pipeline")
    print("=" * 50)

    script_dir = Path(__file__).parent
    assembled_dir = script_dir.parent / "schemas" / "assembled"
    output_dir = script_dir.parent.parent.parent / "src" / "familiar_schemas" / "generated" / "rust"

    print(f"📁 Assembled schemas: {assembled_dir}")
    print(f"📁 Output workspace: {output_dir}")

    # Clean setup
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True)

    # Find all assembled schemas
    all_schemas = list(assembled_dir.glob("*.schema.json"))
    print(f"🔍 Found {len(all_schemas)} assembled schema files")

    # Categorize schemas by crate
    crate_schemas = {}
    for schema_file in all_schemas:
        schema_name = schema_file.stem.replace('.schema', '')
        if schema_name == "assembly_manifest":
            continue
            
        crate_name = categorize_schema(schema_name)
        
        if crate_name not in crate_schemas:
            crate_schemas[crate_name] = []
        crate_schemas[crate_name].append((schema_name, schema_file))

    print(f"📊 Schema distribution across crates:")
    for crate_name, schemas in crate_schemas.items():
        print(f"   📦 {crate_name}: {len(schemas)} schemas")

    # Define dependency order (dependencies first)
    dependency_order = ['types', 'physics', 'common', 'components', 'tables', 'laws', 'workflows', 'taxonomy', 'entities', 'infrastructure']
    
    # Define crate dependencies
    crate_deps = {
        'types': [],
        'physics': ['types'],
        'common': ['types'],
        'components': ['types', 'physics'],
        'tables': ['types', 'common'],
        'laws': ['types', 'physics'],
        'workflows': ['types', 'common'],
        'taxonomy': ['types', 'physics'],
        'entities': ['types', 'physics', 'components'],
        'infrastructure': ['types', 'common']
    }

    successful_files = 0
    failed_files = 0

    # Process crates in dependency order
    for crate_name in dependency_order:
        if crate_name not in crate_schemas:
            continue
            
        print(f"\n🏗️  Processing crate: {crate_name}")
        crate_dir = output_dir / crate_name
        src_dir = crate_dir / "src"
        
        modules = []
        
        for schema_name, schema_file in crate_schemas[crate_name]:
            print(f"🔧 Processing {schema_name}")
            
            try:
                # Load and clean schema
                schema_data = json.loads(schema_file.read_text())
                cleaned_schema = clean_schema_for_typify(schema_data)
                
                # Generate Rust code
                rust_file = src_dir / f"{schema_name.lower()}.rs"
                if generate_rust_code(schema_name, cleaned_schema, rust_file):
                    modules.append(schema_name.lower())
                    successful_files += 1
                else:
                    failed_files += 1
                    
            except Exception as e:
                print(f"⚠️  Error processing {schema_name}: {e}")
                failed_files += 1

        # Create crate
        deps = crate_deps.get(crate_name, [])
        create_crate(crate_dir, crate_name, deps, modules)

    # Create workspace Cargo.toml
    workspace_members = [f'"{crate}"' for crate in dependency_order if crate in crate_schemas]
    workspace_toml = f"""[workspace]
members = [
    {",\n    ".join(workspace_members)}
]
resolver = "2"

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["serde", "v4"] }}
indexmap = {{ version = "2.0", features = ["serde"] }}
"""
    
    (output_dir / "Cargo.toml").write_text(workspace_toml)

    # Summary
    total_schemas = successful_files + failed_files
    success_rate = (successful_files / total_schemas * 100) if total_schemas > 0 else 0
    
    print(f"\n📊 Generation Summary:")
    print(f"   ⚡ Successful: {successful_files}")
    print(f"   ❌ Failed: {failed_files}")
    print(f"   🎯 Success rate: {success_rate:.1f}%")
    print(f"\n🎉 Rust workspace created at: {output_dir}")
    print(f"\n🚀 To build the entire workspace:")
    print(f"   cd {output_dir}")
    print(f"   cargo build --workspace")

if __name__ == "__main__":
    sys.exit(main()) 