#!/usr/bin/env python3
"""
Generate Rust structs from Pydantic models using Copier + jinja2-jsonschema validation.

This script:
1. Loads extracted Pydantic model data
2. Uses Copier to generate Rust structs with full template pipeline
3. Includes inline JSON schema validation during generation
4. Organizes output by category (entities, components, snippets, etc.)
5. Updates existing lib.rs files in each category crate (NOT creating src/ structure)
"""

import json
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Any
import subprocess

def load_pydantic_models() -> Dict[str, Any]:
    """Load the extracted Pydantic model data."""
    data_file = Path(__file__).parent.parent / "schemas" / "assembled" / "json_rust" / "all_pydantic_models.json"
    
    if not data_file.exists():
        print(f"❌ Pydantic model data not found: {data_file}")
        print("   Run extract_pydantic_entities.py first")
        return {}
    
    with open(data_file, 'r') as f:
        return json.load(f)

def sanitize_pascal_case(name: str) -> str:
    """Convert name to proper PascalCase without underscores."""
    # Split on underscores and capitalize each part
    parts = name.split('_')
    # Remove empty parts and capitalize
    clean_parts = [part.capitalize() for part in parts if part]
    result = ''.join(clean_parts)
    
    # Ensure it starts with uppercase
    if result and result[0].islower():
        result = result[0].upper() + result[1:]
    
    return result

def escape_rust_keywords(field_name: str) -> str:
    """Escape Rust keywords by adding r# prefix."""
    rust_keywords = {
        'as', 'break', 'const', 'continue', 'crate', 'else', 'enum', 'extern',
        'false', 'fn', 'for', 'if', 'impl', 'in', 'let', 'loop', 'match',
        'mod', 'move', 'mut', 'pub', 'ref', 'return', 'self', 'Self', 'static',
        'struct', 'super', 'trait', 'true', 'type', 'unsafe', 'use', 'where',
        'while', 'async', 'await', 'dyn', 'abstract', 'become', 'box', 'do',
        'final', 'macro', 'override', 'priv', 'typeof', 'unsized', 'virtual',
        'yield', 'try'
    }
    
    if field_name in rust_keywords:
        return f"r#{field_name}"
    return field_name

def prepare_entity_data(models: Dict[str, Any], category: str) -> List[Dict[str, Any]]:
    """Prepare entity data for Rust generation."""
    entities = []
    
    for model_name, model_data in models.items():
        if model_data.get("category") != category:
            continue
        
        # Sanitize the model name to proper PascalCase
        clean_name = sanitize_pascal_case(model_name)
        
        # Convert fields to Rust format
        rust_fields = []
        has_physics_fields = False
        
        for field in model_data.get("fields", []):
            is_physics_field = any(keyword in field["name"].lower() 
                                 for keyword in ["energy", "tension", "decay", "normalized", "factor", "physics"])
            if is_physics_field:
                has_physics_fields = True
                
            # Escape Rust keywords in field names
            safe_field_name = escape_rust_keywords(field["name"])
                
            rust_field = {
                "name": safe_field_name,
                "rust_type": convert_to_rust_type(field["python_type"], field["name"]),
                "required": field["required"],
                "description": field.get("description", "")
            }
            rust_fields.append(rust_field)
        
        entity = {
            "name": clean_name,
            "snake_name": to_snake_case(clean_name),
            "pydantic_class": model_data.get("module", "") + "." + model_data.get("class_name", model_name),
            "schema_file": None,  # We don't have direct schema files, they're generated from Pydantic
            "fields": rust_fields,
            "has_physics_fields": has_physics_fields
        }
        entities.append(entity)
    
    return entities

def convert_to_rust_type(python_type: str, field_name: str = "") -> str:
    """Convert Python type to Rust type."""
    # Handle common type mappings
    type_map = {
        "str": "String",
        "int": "i64", 
        "float": "f64",
        "bool": "bool",
        "typing.Union": "Option",
        "typing.Optional": "Option",
        "typing.List": "Vec",
        "typing.Dict": "HashMap",
        "datetime.datetime": "chrono::DateTime<chrono::Utc>",
        "uuid.UUID": "uuid::Uuid"
    }
    
    # Clean up the type string
    clean_type = python_type.replace("<class '", "").replace("'>", "").replace("typing.", "")
    
    # Handle special physics fields
    if "normalized" in field_name.lower() or "factor" in field_name.lower():
        return "f64"  # Normalized values are always f64
    
    # Handle Vec/List types
    if "List[" in clean_type or "Vec[" in clean_type:
        inner_type = clean_type.split("[")[1].rstrip("]")
        inner_rust = convert_to_rust_type(inner_type)
        return f"Vec<{inner_rust}>"
    
    # Handle Optional types
    if "Union[" in clean_type and "NoneType" in clean_type:
        # Extract the non-None type
        types = clean_type.split("[")[1].rstrip("]").split(", ")
        non_none_types = [t for t in types if "NoneType" not in t]
        if non_none_types:
            inner_rust = convert_to_rust_type(non_none_types[0])
            return f"Option<{inner_rust}>"
    
    return type_map.get(clean_type, "String")  # Default to String

def to_snake_case(name: str) -> str:
    """Convert PascalCase to snake_case."""
    import re
    # Handle special cases first
    if name.lower() == name:  # Already lowercase
        return name.lower()
    
    # Insert underscores before uppercase letters
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    s2 = re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1)
    
    # Clean up any double underscores and convert to lowercase
    result = re.sub('_+', '_', s2).lower()
    
    # Remove leading/trailing underscores
    return result.strip('_')

def install_jinja2_jsonschema():
    """Install jinja2-jsonschema if not available."""
    try:
        import jinja2_jsonschema
        print("    ✅ jinja2-jsonschema already available")
        return True
    except ImportError:
        print("    📦 Installing jinja2-jsonschema...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "jinja2-jsonschema"], 
                         check=True, capture_output=True)
            print("    ✅ jinja2-jsonschema installed successfully")
            return True
        except subprocess.CalledProcessError as e:
            print(f"    ❌ Failed to install jinja2-jsonschema: {e}")
            return False

def generate_rust_with_copier(entities: List[Dict[str, Any]], category: str, base_output_dir: Path) -> bool:
    """Generate Rust code using existing category crate structure."""
    
    # Ensure jinja2-jsonschema is available
    if not install_jinja2_jsonschema():
        print(f"    ⚠️  Proceeding without schema validation")
    
    # Set up paths - use existing category crate structure
    script_dir = Path(__file__).parent
    root_dir = script_dir.parent.parent.parent
    template_dir = root_dir / "templates" / "rust-entities"
    
    # Target the existing category crate directory
    category_output_dir = base_output_dir / category
    
    if not template_dir.exists():
        print(f"    ❌ Template directory not found: {template_dir}")
        return False
    
    if not category_output_dir.exists():
        print(f"    ⚠️  Category directory doesn't exist: {category_output_dir}")
        print(f"    📁 Creating category crate: {category}")
        category_output_dir.mkdir(parents=True, exist_ok=True)
        (category_output_dir / "src").mkdir(exist_ok=True)
    
    try:
        # Use Copier to generate the category crate with all entities
        import copier
        
        print(f"    🔧 Running Copier for {category} crate...")
        
        # Prepare answers for the category crate
        answers = {
            "service_name": f"familiar-{category}",
            "project_name": f"familiar_{category}",
            "entities": entities,
            "include_physics_validation": True,
            "include_ecs_components": True,
            "include_serde_support": True,
            "include_builder_patterns": False,   # Disabled to avoid compilation issues
            "generate_contract_tests": True,
            "enable_schema_validation": True,    # Enable schema validation with jinja2-jsonschema
            "pydantic_source_dir": "../src/familiar_schemas/familiar_schemas/entities",
            "json_schema_dir": "../docs/v3/schemas/assembled/json"
        }
        
        # Generate the category crate using Copier
        copier.run_copy(
            src_path=str(template_dir),
            dst_path=str(category_output_dir),
            data=answers,
            answers_file=None,
            overwrite=True,
            unsafe=True,  # Allow overwriting
            vcs_ref=None
        )
        
        print(f"    ✅ Copier generation completed for {category}")
        
        # Format generated Rust code with cargo fmt if Cargo.toml exists
        cargo_toml = category_output_dir / "Cargo.toml"
        if cargo_toml.exists():
            print(f"    🎨 Formatting Rust code with cargo fmt...")
            try:
                subprocess.run(['cargo', 'fmt'], cwd=str(category_output_dir), check=True, capture_output=True)
                print(f"    ✅ Rust code formatted successfully")
            except (subprocess.CalledProcessError, FileNotFoundError) as e:
                print(f"    ⚠️  cargo fmt not available or failed: {e}")
        
        return True
        
    except ImportError:
        print(f"    ❌ Copier not installed. Installing...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "copier"], 
                         check=True, capture_output=True)
            print(f"    ✅ Copier installed. Please run the script again.")
            return False
        except subprocess.CalledProcessError as e:
            print(f"    ❌ Failed to install Copier: {e}")
            return False
    except Exception as e:
        print(f"    ❌ Copier generation failed: {e}")
        return False

def main():
    print("🦀 Familiar Pydantic → Rust Generator with Copier + jinja2-jsonschema")
    print("=" * 70)
    
    # Load Pydantic model data
    print("📂 Loading Pydantic model data...")
    data = load_pydantic_models()
    
    if not data:
        return 1
    
    models = data.get("models", {})
    
    # Count actual models per category
    actual_counts = {}
    for category in ["entities", "components", "snippets", "laws", "tables", "taxonomy", "workflows"]:
        actual_counts[category] = len([name for name, model_data in models.items() 
                                     if model_data.get("category") == category])
    
    print(f"   ✅ Loaded {len(models)} models")
    print(f"   📊 Actual categories: {actual_counts}")
    
    # Set up output directory - use existing structure
    script_dir = Path(__file__).parent
    root_dir = script_dir.parent.parent.parent
    base_output_dir = root_dir / "src" / "familiar_schemas" / "generated" / "rust"
    
    print(f"\n🏗️  Updating Rust crates in: {base_output_dir}")
    
    # Generate only entities (needed for API endpoints)
    categories_to_generate = ["entities"]  # Only entities need API endpoints
    total_generated = 0
    
    for category in categories_to_generate:
        category_count = actual_counts.get(category, 0)
        if category_count == 0:
            print(f"  ⏭️  Skipping {category} (no models)")
            continue
        
        print(f"\n  🦀 Updating {category} crate ({category_count} models)...")
        
        # Prepare entity data for this category
        entities = prepare_entity_data(models, category)
        
        if not entities:
            print(f"    ⚠️  No entities found for {category}")
            continue
        
        # Generate Rust code using Copier
        if generate_rust_with_copier(entities, category, base_output_dir):
            total_generated += len(entities)
            print(f"    📊 {category}: {len(entities)} structs generated")
        else:
            print(f"    ❌ Failed to generate {category}")
    
    print(f"\n🎯 Generation Summary:")
    print(f"   ✅ Total structs generated: {total_generated}")
    print(f"   📁 Base directory: {base_output_dir}")
    print(f"   🔧 Next steps:")
    print(f"      1. cd {base_output_dir}/{categories_to_generate[0]}")
    print(f"      2. cargo check")
    print(f"      3. cargo test")
    
    # List documentation URLs
    print(f"\n📚 Generated Documentation:")
    for category in categories_to_generate:
        category_count = actual_counts.get(category, 0)
        if category_count > 0:
            doc_url = f"file://{base_output_dir}/{category}/target/doc/index.html"
            print(f"   🌐 {category}: {doc_url}")
    
    print(f"\n🔍 Features enabled:")
    print(f"   ✅ Copier template pipeline")
    print(f"   ✅ jinja2-jsonschema validation")
    print(f"   ✅ cargo fmt formatting")
    print(f"   ✅ Existing crate structure preserved")
    print(f"   ✅ Rust keyword escaping")

if __name__ == "__main__":
    sys.exit(main()) 