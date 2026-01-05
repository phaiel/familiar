#!/usr/bin/env python3
"""
Recursive Schema Build Pipeline for Familiar v3

Builds schemas from lowest level (primitives) to highest (entities),
ensuring dependencies are built before dependents.

This solves the current 0% entity success rate by:
1. Building in correct dependency order
2. Preprocessing schemas for Rust compatibility
3. Using appropriate generation strategy per level
"""

import json
import os
import sys
import subprocess
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

@dataclass
class SchemaInfo:
    """Information about a schema file"""
    path: Path
    name: str
    level: int
    dependencies: Set[str]
    category: str

class BuildLevel(Enum):
    """Schema build levels in dependency order"""
    PRIMITIVES = 0      # No dependencies
    SIMPLE_TYPES = 1    # Depend on Level 0 only
    COMPLEX_TYPES = 2   # Depend on Levels 0-1
    FIELDS = 3          # Depend on Levels 0-2
    COMPONENTS = 4      # Depend on Levels 0-3
    ENTITIES = 5        # Depend on Levels 0-4

class RecursiveSchemaBuild:
    """Recursive schema build system"""
    
    def __init__(self, schemas_dir: Path, output_dir: Path):
        self.schemas_dir = schemas_dir.resolve()
        self.output_dir = output_dir.resolve()
        self.schemas: Dict[str, SchemaInfo] = {}
        self.build_order: List[List[SchemaInfo]] = [[] for _ in range(6)]
        self.stats = {
            'total': 0,
            'success': 0,
            'failed': 0,
            'by_level': [0] * 6,
            'success_by_level': [0] * 6,
        }
        
    def scan_schemas(self):
        """Scan all schemas and categorize by level"""
        print("🔍 Scanning schemas...")
        
        # Level 0: Primitives (no dependencies)
        self._scan_level(
            BuildLevel.PRIMITIVES,
            ["snippets/types/primitives/*.json"]
        )
        
        # Level 1: Simple types
        self._scan_level(
            BuildLevel.SIMPLE_TYPES,
            [
                "snippets/types/physics/ComplexNumber.json",
                "snippets/types/physics/Vec3.json",
                "snippets/types/physics/Vec6.json",
                "snippets/types/classification/*.json",
                "snippets/types/social/RelationshipType.json",
                "snippets/types/social/BondEvent.json",
                "snippets/types/lifecycles/*.json",
            ]
        )
        
        # Level 2: Complex physics types
        self._scan_level(
            BuildLevel.COMPLEX_TYPES,
            [
                "snippets/types/physics/DensityMatrix.json",
                "snippets/types/physics/EntanglementMap.json",
                "snippets/types/physics/PhysicsConstants.json",
                "snippets/types/physics/AbstractionLevel.json",
                "snippets/types/physics/CognitivePerspective.json",
                "snippets/types/physics/FilamentType.json",
                "snippets/types/physics/MotifType.json",
            ]
        )
        
        # Level 3: Fields
        self._scan_level(
            BuildLevel.FIELDS,
            ["snippets/fields/*.json"]
        )
        
        # Level 4: Base schemas and components
        self._scan_level(
            BuildLevel.COMPONENTS,
            [
                "_base/*.schema.json",
                "components/*.schema.json",
            ]
        )
        
        # Level 5: Entities
        self._scan_level(
            BuildLevel.ENTITIES,
            ["entities/*.schema.json"]
        )
        
        self._print_scan_summary()
        
    def _scan_level(self, level: BuildLevel, patterns: List[str]):
        """Scan schemas matching patterns into a level"""
        for pattern in patterns:
            for schema_path in self.schemas_dir.glob(pattern):
                if schema_path.is_file() and schema_path.suffix == '.json':
                    self._add_schema(schema_path, level.value)
                    
    def _add_schema(self, path: Path, level: int):
        """Add a schema to the build order"""
        schema_name = self._get_schema_name(path)
        
        # Skip duplicates
        if schema_name in self.schemas:
            return
            
        # Analyze dependencies
        dependencies = self._analyze_dependencies(path)
        
        # Determine category
        category = self._categorize_schema(path)
        
        schema_info = SchemaInfo(
            path=path,
            name=schema_name,
            level=level,
            dependencies=dependencies,
            category=category
        )
        
        self.schemas[schema_name] = schema_info
        self.build_order[level].append(schema_info)
        self.stats['total'] += 1
        self.stats['by_level'][level] += 1
        
    def _get_schema_name(self, path: Path) -> str:
        """Extract clean schema name from path"""
        name = path.name
        
        # Remove extensions
        if name.endswith('.event.schema.json'):
            name = name.replace('.event.schema.json', '')
        elif name.endswith('.table.schema.json'):
            name = name.replace('.table.schema.json', '')
        elif name.endswith('.schema.json'):
            name = name.replace('.schema.json', '')
        elif name.endswith('.json'):
            name = name.replace('.json', '')
            
        return name
        
    def _analyze_dependencies(self, path: Path) -> Set[str]:
        """Extract $ref dependencies from schema"""
        dependencies = set()
        
        try:
            with open(path, 'r') as f:
                content = json.load(f)
                self._extract_refs(content, dependencies)
        except Exception as e:
            print(f"⚠️  Failed to analyze {path}: {e}")
            
        return dependencies
        
    def _extract_refs(self, obj: any, refs: Set[str]):
        """Recursively extract $ref values"""
        if isinstance(obj, dict):
            if '$ref' in obj:
                ref = obj['$ref']
                # Extract just the filename
                if '/' in ref:
                    ref = ref.split('/')[-1]
                if ref.endswith('.json'):
                    ref = ref.replace('.json', '')
                refs.add(ref)
            for value in obj.values():
                self._extract_refs(value, refs)
        elif isinstance(obj, list):
            for item in obj:
                self._extract_refs(item, refs)
                
    def _categorize_schema(self, path: Path) -> str:
        """Determine schema category from path"""
        relative = path.relative_to(self.schemas_dir)
        parts = relative.parts
        
        if len(parts) >= 2:
            return parts[0]  # entities, components, etc.
        return "root"
        
    def _print_scan_summary(self):
        """Print summary of scanned schemas"""
        print(f"\n📊 Scanned {self.stats['total']} schemas:")
        
        level_names = [
            "Primitives",
            "Simple Types",
            "Complex Types",
            "Fields",
            "Components",
            "Entities"
        ]
        
        for level in range(6):
            count = self.stats['by_level'][level]
            print(f"  Level {level} ({level_names[level]:15s}): {count:3d} schemas")
            
    def build_all(self) -> bool:
        """Execute full recursive build"""
        print("\n" + "="*60)
        print("🚀 Starting Recursive Build")
        print("="*60)
        
        for level in range(6):
            if not self._build_level(level):
                print(f"\n❌ Build failed at Level {level}")
                return False
                
        self._print_final_summary()
        return True
        
    def _build_level(self, level: int) -> bool:
        """Build all schemas in a level"""
        schemas = self.build_order[level]
        
        if not schemas:
            return True
            
        level_names = ["Primitives", "Simple Types", "Complex Types", 
                      "Fields", "Components", "Entities"]
        
        print(f"\n{'='*60}")
        print(f"📦 Building Level {level}: {level_names[level]}")
        print(f"{'='*60}")
        print(f"Schemas to build: {len(schemas)}")
        
        # Create output directory for this level
        level_output = self.output_dir / f"level_{level}_{level_names[level].lower().replace(' ', '_')}"
        level_output.mkdir(parents=True, exist_ok=True)
        
        success_count = 0
        fail_count = 0
        
        for schema in schemas:
            try:
                # Preprocess schema for Rust compatibility
                processed = self._preprocess_schema(schema, level)
                
                # Generate Rust code
                rust_generated = self._generate_rust(schema, processed, level, level_output)
                
                if rust_generated:
                    success_count += 1
                    self.stats['success'] += 1
                    self.stats['success_by_level'][level] += 1
                    print(f"  ✅ {schema.name}")
                else:
                    fail_count += 1
                    self.stats['failed'] += 1
                    print(f"  ⚠️  {schema.name} (generation returned no output)")
                    
            except Exception as e:
                fail_count += 1
                self.stats['failed'] += 1
                print(f"  ❌ {schema.name}: {e}")
                
        print(f"\nLevel {level} Results: {success_count} success, {fail_count} failed")
        return True  # Continue even if some fail
        
    def _preprocess_schema(self, schema: SchemaInfo, level: int) -> dict:
        """Preprocess schema for Rust compatibility"""
        with open(schema.path, 'r') as f:
            content = json.load(f)
            
        # Apply preprocessing based on level
        if level >= BuildLevel.ENTITIES.value:
            content = self._fix_enum_const_conflicts(content)
            
        if level >= BuildLevel.COMPLEX_TYPES.value:
            content = self._fix_constrained_numerics(content)
            
        content = self._clean_for_rust(content)
        
        return content
        
    def _fix_enum_const_conflicts(self, schema: dict) -> dict:
        """Fix enum + const conflicts that break Rust generators"""
        def fix_recursive(obj):
            if isinstance(obj, dict):
                # If both enum and const exist, remove enum (keep const for specificity)
                if 'enum' in obj and 'const' in obj:
                    del obj['enum']
                    print(f"    🔧 Fixed enum+const conflict")
                    
                for key, value in obj.items():
                    obj[key] = fix_recursive(value)
                    
            elif isinstance(obj, list):
                return [fix_recursive(item) for item in obj]
                
            return obj
            
        return fix_recursive(schema)
        
    def _fix_constrained_numerics(self, schema: dict) -> dict:
        """Transform constrained numerics to newtype hints"""
        def fix_recursive(obj):
            if isinstance(obj, dict):
                # Detect constrained numeric
                if obj.get('type') == 'number' and ('minimum' in obj or 'maximum' in obj):
                    # Mark for newtype generation
                    obj['x-rust-newtype'] = True
                    obj['x-rust-validation'] = {
                        'min': obj.pop('minimum', None),
                        'max': obj.pop('maximum', None),
                    }
                    print(f"    🔧 Marked constrained numeric for newtype")
                    
                for key, value in obj.items():
                    obj[key] = fix_recursive(value)
                    
            elif isinstance(obj, list):
                return [fix_recursive(item) for item in obj]
                
            return obj
            
        return fix_recursive(schema)
        
    def _clean_for_rust(self, schema: dict) -> dict:
        """Remove Rust-incompatible extensions"""
        extensions_to_remove = [
            'category', 'source_file', 'schema_version',
            'physics_properties',
        ]
        
        def clean_recursive(obj):
            if isinstance(obj, dict):
                for ext in extensions_to_remove:
                    if ext in obj:
                        del obj[ext]
                        
                for key, value in obj.items():
                    obj[key] = clean_recursive(value)
                    
            elif isinstance(obj, list):
                return [clean_recursive(item) for item in obj]
                
            return obj
            
        return clean_recursive(schema)
        
    def _generate_rust(self, schema: SchemaInfo, processed: dict, 
                      level: int, output_dir: Path) -> bool:
        """Generate Rust code using appropriate strategy per level"""
        
        # Write processed schema to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(processed, f, indent=2)
            temp_path = f.name
            
        try:
            # Choose generation strategy based on level
            if level <= BuildLevel.COMPLEX_TYPES.value:
                # Use quicktype for simple types
                return self._generate_with_quicktype(schema, temp_path, output_dir)
            elif level == BuildLevel.FIELDS.value:
                # Use quicktype with type aliases
                return self._generate_with_quicktype(schema, temp_path, output_dir, use_aliases=True)
            else:
                # For components and entities, try quicktype but expect some failures
                # In the future, this will use custom templates
                return self._generate_with_quicktype(schema, temp_path, output_dir)
                
        finally:
            # Clean up temp file
            if os.path.exists(temp_path):
                os.unlink(temp_path)
                
    def _generate_with_quicktype(self, schema: SchemaInfo, schema_path: str, 
                                output_dir: Path, use_aliases: bool = False) -> bool:
        """Generate Rust code using quicktype"""
        
        output_file = output_dir / f"{schema.name}.rs"
        
        cmd = [
            "quicktype",
            "--src-lang", "schema",
            "--lang", "rust",
            "--derive-debug",
            "--derive-clone",
            "--visibility", "public",
            "--density", "dense",
            "--out", str(output_file),
            schema_path
        ]
        
        if use_aliases:
            cmd.append("--use-default-for-missing")
            
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                return True
            else:
                # Don't print full error for expected failures
                if "not yet implemented" not in result.stderr:
                    print(f"      Error: {result.stderr[:100]}...")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"      Timeout generating {schema.name}")
            return False
        except FileNotFoundError:
            print(f"      quicktype not found - install with: npm install -g quicktype")
            return False
        except Exception as e:
            print(f"      Exception: {e}")
            return False
            
    def _print_final_summary(self):
        """Print final build statistics"""
        print("\n" + "="*60)
        print("📊 Build Summary")
        print("="*60)
        
        total = self.stats['total']
        success = self.stats['success']
        failed = self.stats['failed']
        
        success_rate = (success / total * 100) if total > 0 else 0
        
        print(f"\nOverall: {success}/{total} schemas ({success_rate:.1f}% success)")
        print(f"  ✅ Success: {success}")
        print(f"  ❌ Failed:  {failed}")
        
        print("\nBy Level:")
        level_names = [
            "Primitives",
            "Simple Types",
            "Complex Types",
            "Fields",
            "Components",
            "Entities"
        ]
        
        for level in range(6):
            total_level = self.stats['by_level'][level]
            success_level = self.stats['success_by_level'][level]
            if total_level > 0:
                rate = success_level / total_level * 100
                print(f"  Level {level} ({level_names[level]:15s}): {success_level:3d}/{total_level:3d} ({rate:5.1f}%)")
                

def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Recursive Schema Build Pipeline for Familiar v3"
    )
    parser.add_argument(
        "--schemas-dir",
        type=Path,
        default=Path(__file__).parent.parent / "schemas",
        help="Directory containing schema files"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(__file__).parent.parent / "rust_generated",
        help="Output directory for generated Rust code"
    )
    
    args = parser.parse_args()
    
    # Validate inputs
    if not args.schemas_dir.exists():
        print(f"❌ Schemas directory not found: {args.schemas_dir}")
        return 1
        
    # Create builder
    builder = RecursiveSchemaBuild(args.schemas_dir, args.output_dir)
    
    # Scan and build
    builder.scan_schemas()
    success = builder.build_all()
    
    return 0 if success else 1
    

if __name__ == "__main__":
    sys.exit(main())

