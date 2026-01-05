#!/usr/bin/env python3
"""
Familiar v3 Complete Schema Assembly Pipeline

This script processes ALL schemas in the directory structure (except snippets),
fully dereferencing each one and outputting them as individual files.
This replaces the FamiliarTypes.schema.json approach with a comprehensive scan.
"""
import json
import os
import shutil
import argparse
from pathlib import Path
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012
from referencing.exceptions import Unresolvable

def remove_problematic_string_patterns(node: any):
    """
    Recursively removes all problematic constraints from string type definitions.
    This is a comprehensive workaround for typify assertion failures.
    """
    if isinstance(node, dict):
        is_string_type = (
            node.get("type") == "string" or
            (isinstance(node.get("type"), list) and "string" in node.get("type"))
        )

        if is_string_type:
            # Remove all string constraints that can cause typify assertion failures
            # NOTE: We preserve "enum" for Pydantic generation - only remove for typify if needed
            constraints_to_remove = [
                "pattern", "minLength", "maxLength", "format",
                "contentEncoding", "contentMediaType", "contentSchema"
                # "enum" removed from this list - we need enums for proper Pydantic generation
            ]
            for constraint in constraints_to_remove:
                if constraint in node:
                    del node[constraint]

        for value in node.values():
            remove_problematic_string_patterns(value)
            
    elif isinstance(node, list):
        for item in node:
            remove_problematic_string_patterns(item)

def deep_merge(d1, d2):
    """Recursively merges dictionary d2 into d1, handling lists."""
    for k, v in d2.items():
        if k in d1 and isinstance(d1[k], dict) and isinstance(v, dict):
            d1[k] = deep_merge(d1[k], v)
        elif k in d1 and isinstance(d1[k], list) and isinstance(v, list):
            d1[k].extend(x for x in v if x not in d1[k])
        else:
            d1[k] = v
    return d1

def create_default_infrastructure():
    """Create a default infrastructure block for entities with missing infrastructure references."""
    return {
        "type": "object",
        "description": "Default infrastructure configuration",
        "properties": {
            "service_template": {
                "type": "string",
                "default": "familiar-poem-api-service"
            },
            "database_module": {
                "type": "string",
                "default": "timescaledb"
            },
            "required_extensions": {
                "type": "array",
                "items": {"type": "string"},
                "default": ["pgvector", "vectorscale"]
            },
            "streaming_topics": {
                "type": "array",
                "items": {"type": "string"},
                "default": []
            },
            "observability_dashboard": {
                "type": "string",
                "default": "grafana/dashboards/default.json"
            }
        },
        "required": ["service_template", "database_module", "streaming_topics", "observability_dashboard"]
    }

def should_preserve_ref(ref_value: str) -> bool:
    """
    Determine if a $ref should be preserved (not dereferenced) to avoid type duplication.
    Preserve refs to shared types/enums but dereference structural components.
    """
    # Preserve refs to shared types that should not be duplicated
    preserve_patterns = [
        "/types/",           # All type definitions
        "/snippets/types/",  # Snippet type definitions  
        "/snippets/fields/", # Reusable field definitions
        "Type.json",         # Files ending in Type.json (enums)
        "Type.v1.json",      # Versioned type files
    ]
    
    return any(pattern in ref_value for pattern in preserve_patterns)

def _dereference_recursive(node, resolver):
    """
    Recursively walk a JSON structure, selectively resolve $ref pointers,
    and merge allOf directives. Preserve refs to shared types to avoid duplication.
    """
    if isinstance(node, dict):
        if "$ref" in node:
            ref_value = node["$ref"]
            if not isinstance(ref_value, str):
                return node
                
            # Preserve certain refs to avoid type duplication in code generation
            if should_preserve_ref(ref_value):
                print(f"  🔗 Preserving ref: {ref_value}")
                return node  # Return the original $ref node
                
            try:
                resolved = resolver.lookup(ref_value)
                return _dereference_recursive(resolved.contents, resolved.resolver)
            except Unresolvable:
                # Handle missing infrastructure references
                if "infrastructure" in ref_value.lower():
                    print(f"⚠️  Missing infrastructure reference: {ref_value}, using default")
                    default_infra = create_default_infrastructure()
                    # Merge any additional properties from the original node
                    if "properties" in node:
                        if "properties" not in default_infra:
                            default_infra["properties"] = {}
                        default_infra["properties"].update(node["properties"])
                    return default_infra
                else:
                    print(f"⚠️  Unresolvable reference: {ref_value}")
                    return node
        if "allOf" in node:
            merged = {}
            for sub_schema in node.pop("allOf"):
                dereferenced_sub = _dereference_recursive(sub_schema, resolver)
                if dereferenced_sub:
                    merged = deep_merge(merged, dereferenced_sub)
            # Merge the rest of the node
            for key, value in node.items():
                if key != "allOf":
                    if key in merged:
                        if isinstance(merged[key], dict) and isinstance(value, dict):
                            merged[key] = deep_merge(merged[key], value)
                        else:
                            merged[key] = value
                    else:
                        merged[key] = value
            return _dereference_recursive(merged, resolver)
        else:
            result = {}
            for key, value in node.items():
                result[key] = _dereference_recursive(value, resolver)
            return result
    elif isinstance(node, list):
        return [_dereference_recursive(item, resolver) for item in node]
    else:
        return node

def clean_schema_for_pydantic(schema):
    """
    Remove or rename JSON Schema keywords that cause issues with Pydantic generation.
    """
    if isinstance(schema, dict):
        # Remove problematic JSON Schema keywords
        cleaned = {}
        for key, value in schema.items():
            if key.startswith('$'):
                # Skip $schema, $id, $ref etc. as they cause Pydantic generation issues
                continue
            else:
                cleaned[key] = clean_schema_for_pydantic(value)
        return cleaned
    elif isinstance(schema, list):
        return [clean_schema_for_pydantic(item) for item in schema]
    else:
        return schema

def dereference_schema(schema_path: Path, registry: Registry) -> dict:
    """
    Loads a schema and fully dereferences it, including all $ref and allOf.
    """
    schema_content = json.loads(schema_path.read_text(encoding="utf-8"))
    resolver = registry.resolver(schema_path.as_uri())
    return _dereference_recursive(schema_content, resolver)

def get_schema_name(schema_path: Path) -> str:
    """Extract a clean schema name from the file path."""
    # Remove various extensions in order of specificity
    name = schema_path.name
    
    # Handle specific multi-part extensions first
    if name.endswith('.event.schema.json'):
        name = name.replace('.event.schema.json', '')
    elif name.endswith('.table.schema.json'):
        name = name.replace('.table.schema.json', '')
    elif name.endswith('.schema.json'):
        name = name.replace('.schema.json', '')
    elif name.endswith('.json'):
        # For snippets and other .json files, just remove .json
        name = name.replace('.json', '')
    
    return name

def categorize_schema(schema_path: Path, source_dir: Path) -> str:
    """Determine the category of a schema based on its path."""
    relative_path = schema_path.relative_to(source_dir)
    parts = relative_path.parts
    
    if len(parts) > 1:
        return parts[0]  # entities, components, events, etc.
    else:
        return "root"

def main(args):
    source_dir = args.source_dir.resolve()
    output_dir = args.output_dir.resolve()

    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Find ALL schema files (including snippets, excluding FamiliarTypes)
    all_schema_files = []
    total_found = 0
    for schema_path in source_dir.rglob('*.json'):
        total_found += 1
        # Skip FamiliarTypes.schema.json (we're replacing it)
        if schema_path.name == 'FamiliarTypes.schema.json':
            continue
        # Skip .DS_Store and other non-schema files
        if not schema_path.name.endswith('.json'):
            continue
        # Skip if it's clearly not a schema
        if '.vscode' in schema_path.parts:
            continue
            
        all_schema_files.append(schema_path)
    
    print(f"🔍 Found {len(all_schema_files)} schema files to process (out of {total_found} total JSON files)")
    
    # Categorize schemas
    categories = {}
    for schema_path in all_schema_files:
        category = categorize_schema(schema_path, source_dir)
        if category not in categories:
            categories[category] = []
        categories[category].append(schema_path)
    
    print(f"📊 Schema categories: {dict((k, len(v)) for k, v in categories.items())}")

    # Build registry with ALL schemas for reference resolution
    resources = []
    for schema_path in all_schema_files:
        try:
            content = json.loads(schema_path.read_text(encoding="utf-8"))
            resource = Resource.from_contents(content, default_specification=DRAFT202012)
            resources.append((schema_path.as_uri(), resource))
        except json.JSONDecodeError as e:
            print(f"  ⚠️  Skipping invalid JSON: {schema_path} - {e}")
            continue
    
    registry = Registry().with_resources(resources)
    print(f"🗂️  Built registry with {len(resources)} schema resources")

    # Process each schema individually
    processed_count = 0
    failed_count = 0
    
    for schema_path in all_schema_files:
        schema_name = get_schema_name(schema_path)
        category = categorize_schema(schema_path, source_dir)
        
        print(f"  📝 Processing {category}/{schema_name}...")
        
        try:
            # Fully dereference this schema
            dereferenced = dereference_schema(schema_path, registry)
            
            # Apply the typify workaround
            remove_problematic_string_patterns(dereferenced)
            
            # Create a clean schema wrapper
            individual_schema = {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": f"https://familiar.dev/schemas/{schema_name}.v1.schema.json",
                "title": schema_name,
                "description": dereferenced.get("description", f"Schema for {schema_name}"),
                "category": category,
                "source_file": str(schema_path.relative_to(source_dir)),
                **dereferenced
            }
            
            # Write individual schema file
            output_path = output_dir / f"{schema_name}.schema.json"
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(individual_schema, f, indent=2, ensure_ascii=False)
            
            processed_count += 1
            
        except Exception as e:
            print(f"    ❌ Failed to process {schema_name}: {e}")
            failed_count += 1
            continue

    print(f"\n✅ Successfully assembled {processed_count} schemas")
    if failed_count > 0:
        print(f"⚠️  Failed to process {failed_count} schemas")
    
    # Generate a manifest of all processed schemas
    manifest = {
        "generated_at": "2024-01-01T00:00:00Z",  # Will be updated with real timestamp
        "total_schemas": processed_count,
        "categories": {k: len(v) for k, v in categories.items()},
        "schemas": []
    }
    
    for schema_path in sorted(output_dir.glob("*.schema.json")):
        schema_name = get_schema_name(schema_path)
        manifest["schemas"].append({
            "name": schema_name,
            "file": schema_path.name,
            "category": "unknown"  # Could be enhanced to track category
        })
    
    manifest_path = output_dir / "assembly_manifest.json"
    with open(manifest_path, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, indent=2, ensure_ascii=False)
    
    print(f"📋 Assembly manifest written to {manifest_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Familiar v3 Complete Schema Assembler")
    parser.add_argument("--source-dir", type=Path, default=Path("schemas"))
    parser.add_argument("--output-dir", type=Path, default=Path("schemas/assembled"))
    args = parser.parse_args()
    main(args) 