#!/usr/bin/env python3
"""
Schema Constraint Fixer for Typify Compatibility

Fixes problematic schema patterns that cause typify to fail:
1. enum + const conflicts (ALL entities fail due to this)
2. Constrained numeric types that cause panics
3. Other typify-incompatible patterns

Usage:
    python fix_schema_constraints.py input_schema.json output_schema.json
    python fix_schema_constraints.py --batch input_dir/ output_dir/
"""

import json
import sys
import argparse
from pathlib import Path
import copy

def fix_enum_const_conflict(schema_obj):
    """
    Fix schemas that have both 'enum' and 'const' fields.
    
    Problem pattern:
    {
        "type": "string",
        "enum": ["Focus", "Filament", "Motif", "Intent", "Moment", "Bond", "Thread"],
        "const": "Bond"
    }
    
    Solution: Replace with just const since it's more specific.
    """
    def fix_recursive(obj):
        if isinstance(obj, dict):
            # Fix enum+const conflicts
            if 'enum' in obj and 'const' in obj:
                print(f"🔧 Fixed enum+const conflict: keeping const='{obj['const']}', removing enum")
                # Keep const, remove enum since const is more specific
                del obj['enum']
            
            # Recursively fix nested objects
            for key, value in obj.items():
                fix_recursive(value)
        elif isinstance(obj, list):
            for item in obj:
                fix_recursive(item)
    
    schema_copy = copy.deepcopy(schema_obj)
    fix_recursive(schema_copy)
    return schema_copy

def fix_constrained_numerics(schema_obj):
    """
    Fix numeric types with constraints that cause typify to panic.
    
    Problem patterns:
    - minimum/maximum on numbers
    - exclusiveMinimum/exclusiveMaximum
    
    Solution: Keep type but remove problematic constraints.
    """
    def fix_recursive(obj):
        if isinstance(obj, dict):
            # Fix constrained numeric types
            if obj.get('type') in ['number', 'integer']:
                constraints_removed = []
                for constraint in ['minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum']:
                    if constraint in obj:
                        del obj[constraint]
                        constraints_removed.append(constraint)
                
                if constraints_removed:
                    print(f"🔧 Fixed numeric constraints: removed {constraints_removed}")
            
            # Recursively fix nested objects
            for key, value in obj.items():
                fix_recursive(value)
        elif isinstance(obj, list):
            for item in obj:
                fix_recursive(item)
    
    schema_copy = copy.deepcopy(schema_obj)
    fix_recursive(schema_copy)
    return schema_copy

def fix_complex_patterns(schema_obj):
    """
    Fix other complex patterns that may cause typify issues.
    """
    def fix_recursive(obj):
        if isinstance(obj, dict):
            # Fix pattern properties that might be too complex
            if 'patternProperties' in obj:
                print("🔧 Found patternProperties - typify may struggle with this")
            
            # Fix conditional schemas (if/then/else)
            if any(key in obj for key in ['if', 'then', 'else']):
                print("🔧 Found conditional schema - typify may struggle with this")
            
            # Recursively fix nested objects
            for key, value in obj.items():
                fix_recursive(value)
        elif isinstance(obj, list):
            for item in obj:
                fix_recursive(item)
    
    schema_copy = copy.deepcopy(schema_obj)
    fix_recursive(schema_copy)
    return schema_copy

def fix_schema(schema_obj):
    """Apply all fixes to make schema typify-compatible."""
    print("🔍 Analyzing schema for typify compatibility issues...")
    
    # Apply fixes in order
    fixed = fix_enum_const_conflict(schema_obj)
    fixed = fix_constrained_numerics(fixed)
    fixed = fix_complex_patterns(fixed)
    
    return fixed

def process_single_schema(input_path, output_path):
    """Process a single schema file."""
    print(f"📝 Processing: {input_path}")
    
    with open(input_path, 'r') as f:
        schema = json.load(f)
    
    fixed_schema = fix_schema(schema)
    
    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, 'w') as f:
        json.dump(fixed_schema, f, indent=2)
    
    print(f"✅ Fixed schema saved to: {output_path}")

def process_batch(input_dir, output_dir):
    """Process all JSON schemas in a directory."""
    input_path = Path(input_dir)
    output_path = Path(output_dir)
    
    schema_files = list(input_path.glob("**/*.json"))
    print(f"📁 Found {len(schema_files)} schema files to process")
    
    for schema_file in schema_files:
        # Preserve directory structure
        relative_path = schema_file.relative_to(input_path)
        output_file = output_path / relative_path
        
        try:
            process_single_schema(schema_file, output_file)
        except Exception as e:
            print(f"❌ Error processing {schema_file}: {e}")

def main():
    parser = argparse.ArgumentParser(description="Fix JSON schemas for typify compatibility")
    parser.add_argument("input", help="Input schema file or directory")
    parser.add_argument("output", help="Output schema file or directory")
    parser.add_argument("--batch", action="store_true", help="Process directory batch")
    
    args = parser.parse_args()
    
    if args.batch:
        process_batch(args.input, args.output)
    else:
        process_single_schema(Path(args.input), Path(args.output))
    
    print("🎉 Schema fixing complete!")

if __name__ == "__main__":
    main() 