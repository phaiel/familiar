#!/usr/bin/env python3
"""
⚠️  DEPRECATED: Extract ALL Pydantic models for Rust generation.

🚨 DEPRECATION NOTICE: This script violates the Schema-First Principle (Recommendation 1)
    
    This script represents REVERSE FLOW (Pydantic → JSON), which is prohibited.
    
    ✅ CORRECT APPROACH: Use datamodel-code-generator for unidirectional flow:
       JSON Schema → Pydantic models (via datamodel-code-generator)
       JSON Schema → Rust types (via typify)
    
    🎯 MIGRATION PATH:
       - Use: pants run docs/v3:generate-python-types  # Schema → Pydantic
       - Use: pants run docs/v3:generate-rust-types    # Schema → Rust
    
    ⚖️  AUTOMATED GOVERNANCE: This script triggers governance check failure
       Run: pants run docs/v3:check-schema-first

🚨 This script will be removed in the next cleanup cycle.

LEGACY FUNCTIONALITY (DO NOT USE):
This script discovers and extracts information from ALL Pydantic models:
- entities, components, snippets, events, laws, payloads, tables, taxonomy, workflows

The extracted data is used to generate Rust structs via Copier + Jinja2 templates,
with validation against the assembled JSON schemas in /json folder.
"""

import sys
import os
import json
import importlib
import inspect
from pathlib import Path
from typing import get_type_hints, get_origin, get_args, Any, Dict, List
from pydantic import BaseModel
from pydantic.fields import FieldInfo

# Add the familiar_schemas package to the path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src" / "familiar_schemas"))
# Also add current working directory for generated modules
sys.path.insert(0, ".")

def get_rust_type(python_type, field_info=None):
    """Convert Python type annotations to Rust types."""
    if python_type is str:
        return "String"
    elif python_type is int:
        return "i64"
    elif python_type is float:
        return "f64"
    elif python_type is bool:
        return "bool"
    elif get_origin(python_type) is list:
        inner_type = get_args(python_type)[0]
        return f"Vec<{get_rust_type(inner_type)}>"
    elif get_origin(python_type) is dict:
        key_type, value_type = get_args(python_type)
        return f"std::collections::HashMap<{get_rust_type(key_type)}, {get_rust_type(value_type)}>"
    elif hasattr(python_type, '__name__'):
        return python_type.__name__
    else:
        return "serde_json::Value"

def extract_field_info(model_class):
    """Extract field information from a Pydantic model."""
    fields = []
    type_hints = get_type_hints(model_class)
    
    for field_name, field_info in model_class.model_fields.items():
        python_type = type_hints.get(field_name, Any)
        rust_type = get_rust_type(python_type, field_info)
        
        field_data = {
            "name": field_name,
            "rust_type": rust_type,
            "python_type": str(python_type),
            "required": field_info.is_required(),
            "default": field_info.default if field_info.default is not ... else None,
            "description": field_info.description
        }
        
        # Add validation constraints if present
        constraints = {}
        if hasattr(field_info, 'constraints'):
            for constraint_name in ['ge', 'le', 'gt', 'lt', 'min_length', 'max_length']:
                if hasattr(field_info.constraints, constraint_name):
                    constraints[constraint_name] = getattr(field_info.constraints, constraint_name)
        
        if constraints:
            field_data["constraints"] = constraints
            
        fields.append(field_data)
    
    return fields

def extract_models_from_category(category_name):
    """Extract all models from a specific category (entities, components, etc.)."""
    models = {}
    
    # Discover files from the category directory - correct path to generated pydantic models
    category_dir = Path("generated/pydantic") / category_name
    
    if not category_dir.exists():
        print(f"Category directory not found: {category_dir}")
        return models
    
    py_files = list(category_dir.glob("*.py"))
    print(f"Found {len(py_files)} files in {category_name}")
    
    for py_file in py_files:
        if py_file.name.startswith("__") or py_file.name.startswith("."):
            continue
            
        module_name = py_file.stem  # Remove .py extension
        
        try:
            # Import the specific module from generated.pydantic
            module = importlib.import_module(f'generated.pydantic.{category_name}.{module_name}')
            
            # Find classes in the module that are Pydantic models
            for name in dir(module):
                attr = getattr(module, name)
                if (inspect.isclass(attr) and 
                    issubclass(attr, BaseModel) and 
                    attr is not BaseModel):
                    
                    print(f"Analyzing {category_name}: {name}")
                    try:
                        fields = extract_field_info(attr)
                        
                        # Create unique model name to avoid duplicates
                        unique_name = name
                        if unique_name in models:
                            # Add module name to make it unique
                            unique_name = f"{name}_{module_name}"
                            print(f"    ⚠️  Duplicate name resolved: {name} → {unique_name}")
                        
                        # Determine correct category based on whether this is a top-level class
                        module_path = f"generated.pydantic.{category_name}.{module_name}"
                        
                        # If this is a nested class (not the main entity), recategorize it
                        actual_category = category_name
                        if category_name == "entities" and not module_path.endswith(f".{name}"):
                            # This is a nested class in an entity file, determine its actual category
                            if name.startswith("Components") or name.endswith("Content") or name.endswith("State"):
                                actual_category = "components"
                            elif name.startswith("Physics") and not name.endswith("Config"):
                                actual_category = "components" 
                            elif name in ["Fields", "Universal", "Infrastructure", "History", "Permissions"]:
                                actual_category = "components"
                            else:
                                actual_category = "snippets"  # Default for other nested classes
                        
                        models[unique_name] = {
                            "class_name": name,
                            "category": actual_category,
                            "module": module_path,
                            "fields": fields,
                            "docstring": attr.__doc__
                        }
                    except Exception as e:
                        print(f"Error analyzing {category_name} {name}: {e}")
        except ImportError as e:
            print(f"Could not import {category_name} module {module_name}: {e}")
    
    return models

def load_json_schemas():
    """Load all JSON schemas for validation."""
    schemas = {}
    # Updated path to the correct assembled schemas location
    json_dir = Path(__file__).parent.parent / "schemas" / "assembled"
    
    if not json_dir.exists():
        print(f"JSON schemas directory not found: {json_dir}")
        return schemas
    
    for schema_file in json_dir.glob("*.schema.json"):
        schema_name = schema_file.stem.replace('.schema', '')
        try:
            with open(schema_file, 'r') as f:
                schemas[schema_name] = json.load(f)
            print(f"Loaded schema: {schema_name}")
        except Exception as e:
            print(f"Error loading schema {schema_name}: {e}")
    
    return schemas

def validate_against_schemas(models, schemas):
    """Validate extracted models against JSON schemas."""
    validation_results = {}
    
    for model_name, model_data in models.items():
        if model_name in schemas:
            validation_results[model_name] = {
                "has_schema": True,
                "schema_file": f"{model_name}.schema.json"
            }
        else:
            validation_results[model_name] = {
                "has_schema": False,
                "schema_file": None
            }
    
    return validation_results

def main():
    print("Extracting ALL Pydantic models for Rust generation...")
    
    # Categories to extract
    categories = [
        "entities", "components", "snippets", "events", 
        "laws", "payloads", "tables", "taxonomy", "workflows"
    ]
    
    all_models = {}
    category_counts = {}
    
    # Extract models from each category
    for category in categories:
        print(f"\n=== EXTRACTING {category.upper()} ===")
        category_models = extract_models_from_category(category)
        all_models.update(category_models)
        category_counts[category] = len(category_models)
        print(f"Found {len(category_models)} {category} models")
    
    # Load JSON schemas for validation
    print(f"\n=== LOADING JSON SCHEMAS ===")
    schemas = load_json_schemas()
    print(f"Loaded {len(schemas)} JSON schemas")
    
    # Validate models against schemas
    print(f"\n=== VALIDATING AGAINST SCHEMAS ===")
    validation_results = validate_against_schemas(all_models, schemas)
    
    # Prepare final result
    result = {
        "models": all_models,
        "schemas": schemas,
        "validation": validation_results,
        "extraction_summary": {
            "total_models": len(all_models),
            "category_counts": category_counts,
            "total_schemas": len(schemas),
            "models_with_schemas": sum(1 for v in validation_results.values() if v["has_schema"]),
            "models_without_schemas": sum(1 for v in validation_results.values() if not v["has_schema"])
        }
    }
    
    # Save to file in schemas/assembled/json_rust
    output_file = Path(__file__).parent.parent / "schemas" / "assembled" / "json_rust" / "all_pydantic_models.json"
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, 'w') as f:
        json.dump(result, f, indent=2, default=str)
    
    # Print summary
    print(f"\n=== FINAL SUMMARY ===")
    print(f"Total models extracted: {len(all_models)}")
    for category, count in category_counts.items():
        print(f"  {category}: {count}")
    print(f"JSON schemas loaded: {len(schemas)}")
    print(f"Models with schemas: {result['extraction_summary']['models_with_schemas']}")
    print(f"Models without schemas: {result['extraction_summary']['models_without_schemas']}")
    print(f"Results saved to: {output_file}")
    
    # Show validation summary
    print(f"\n=== SCHEMA VALIDATION SUMMARY ===")
    with_schemas = [name for name, val in validation_results.items() if val["has_schema"]]
    without_schemas = [name for name, val in validation_results.items() if not val["has_schema"]]
    
    if with_schemas:
        print(f"✅ Models WITH schemas ({len(with_schemas)}):")
        for name in sorted(with_schemas)[:10]:  # Show first 10
            print(f"  {name}")
        if len(with_schemas) > 10:
            print(f"  ... and {len(with_schemas) - 10} more")
    
    if without_schemas:
        print(f"⚠️  Models WITHOUT schemas ({len(without_schemas)}):")
        for name in sorted(without_schemas)[:10]:  # Show first 10
            print(f"  {name}")
        if len(without_schemas) > 10:
            print(f"  ... and {len(without_schemas) - 10} more")

if __name__ == "__main__":
    main()