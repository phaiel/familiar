#!/usr/bin/env python3
"""
JSON Schema to Rust Code Generation Tool Bakeoff (HARD MODE)

Tests tools on the HARDEST schemas - complex entities and physics components:
1. typify (Rust Library) - Using same approach as working script
2. entype (TypeScript) - JSON data → Rust types  
3. quicktype (Multi-lang) - JSON Schema → Rust types
4. schemafy (Rust) - JSON Schema → Rust types [OPTIONAL]

Focus: Complex schemas with polymorphism, physics variables, nested structures
"""

import json
import os
import subprocess
import time
import tempfile
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Optional, Any
import shutil

@dataclass
class ToolResult:
    name: str
    success: bool
    error_message: Optional[str]
    output_size: int
    execution_time: float
    output_file: Optional[str] = None

@dataclass
class SchemaResult:
    schema_name: str
    typify: ToolResult
    entype: ToolResult
    quicktype: ToolResult
    schemafy: Optional[ToolResult] = None

def select_hard_schemas(all_schemas: List[Path]) -> List[Path]:
    """Select only the hardest, most complex schemas for testing."""
    
    # Entity schemas - complex business objects with polymorphism
    entity_schemas = [
        'Bond.schema.json', 'Thread.schema.json', 'Moment.schema.json', 
        'Intent.schema.json', 'Focus.schema.json', 'Filament.schema.json',
        'Stitch.schema.json', 'Motif.schema.json', 'Course.schema.json'
    ]
    
    # Physics component schemas - complex nested structures with constraints
    physics_schemas = [
        'BondPhysicsConfig.schema.json', 'BondTensionDynamics.schema.json',
        'PhysicsProfile.schema.json', 'EmotionalValence.schema.json',
        'MemoryConsolidation.schema.json', 'MotifCollapse.schema.json'
    ]
    
    # Complex component schemas with nested structures
    component_schemas = [
        'BondContent.schema.json', 'BondPermissions.schema.json',
        'ThreadContent.schema.json', 'MomentDetails.schema.json'
    ]
    
    hard_schema_names = entity_schemas + physics_schemas + component_schemas
    
    # Filter to only include schemas that exist
    hard_schemas = []
    for schema_file in all_schemas:
        if schema_file.name in hard_schema_names:
            hard_schemas.append(schema_file)
    
    print(f"🎯 Selected {len(hard_schemas)} HARD schemas for testing:")
    for schema in hard_schemas:
        print(f"   💪 {schema.name}")
    
    return hard_schemas

def clean_schema_for_typify(schema_data: Dict[str, Any]) -> Dict[str, Any]:
    """Clean schema using same approach as working generate_rust_from_assembled.py script."""
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

def run_command(cmd: List[str], input_data: str = None, timeout: int = 60, cwd: str = None) -> tuple[bool, str, float]:
    """Run a command and return (success, output/error, execution_time)"""
    start_time = time.time()
    try:
        if input_data:
            result = subprocess.run(
                cmd, 
                input=input_data, 
                text=True, 
                capture_output=True, 
                timeout=timeout,
                cwd=cwd
            )
        else:
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                timeout=timeout,
                cwd=cwd
            )
        
        execution_time = time.time() - start_time
        
        if result.returncode == 0:
            return True, result.stdout, execution_time
        else:
            return False, result.stderr, execution_time
            
    except subprocess.TimeoutExpired:
        execution_time = time.time() - start_time
        return False, f"Timeout after {timeout}s", execution_time
    except Exception as e:
        execution_time = time.time() - start_time
        return False, str(e), execution_time

def test_typify(schema_file: str, output_dir: str) -> ToolResult:
    """Test typify using EXACT same approach as working generate_rust_from_assembled.py script."""
    output_file = os.path.join(output_dir, "typify_output.rs")
    
    try:
        # Load and clean schema using same method as working script
        schema_data = json.loads(Path(schema_file).read_text())
        cleaned_schema = clean_schema_for_typify(schema_data)
        
        # Create temporary file with cleaned schema
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(cleaned_schema, f, indent=2)
            temp_schema_path = f.name
        
        try:
            # Use cargo typify directly like the working script
            success, output, exec_time = run_command([
                "cargo", "typify", "-o", output_file, temp_schema_path
            ], timeout=60)
            
            output_size = 0
            if success and os.path.exists(output_file):
                output_size = os.path.getsize(output_file)
            
            return ToolResult(
                name="typify",
                success=success,
                error_message=None if success else output,
                output_size=output_size,
                execution_time=exec_time,
                output_file=output_file if success else None
            )
            
        finally:
            os.unlink(temp_schema_path)
            
    except Exception as e:
        return ToolResult(
            name="typify",
            success=False,
            error_message=str(e),
            output_size=0,
            execution_time=0
        )

def test_entype(schema_file: str, output_dir: str) -> ToolResult:
    """Test entype tool (typegen-json)"""
    output_file = os.path.join(output_dir, "entype_output.rs")
    
    try:
        with open(schema_file, 'r') as f:
            schema = json.load(f)
        
        # Create a complex JSON sample from schema
        sample_data = create_complex_sample_from_schema(schema)
        sample_json = json.dumps(sample_data)
        
        cmd = ["typegen-json", "--lang", "rust"]
        
        success, output, exec_time = run_command(cmd, input_data=sample_json)
        
        if success:
            with open(output_file, 'w') as f:
                f.write(output)
            output_size = len(output)
        else:
            output_size = 0
            
    except Exception as e:
        return ToolResult(
            name="entype",
            success=False,
            error_message=str(e),
            output_size=0,
            execution_time=0
        )
    
    return ToolResult(
        name="entype",
        success=success,
        error_message=None if success else output,
        output_size=output_size,
        execution_time=exec_time,
        output_file=output_file if success else None
    )

def test_quicktype(schema_file: str, output_dir: str) -> ToolResult:
    """Test quicktype tool"""
    output_file = os.path.join(output_dir, "quicktype_output.rs")
    
    cmd = ["quicktype", "--src-lang", "schema", "--lang", "rust", 
           "--src", schema_file, "--out", output_file]
    
    success, output, exec_time = run_command(cmd, timeout=60)
    
    output_size = 0
    if success and os.path.exists(output_file):
        output_size = os.path.getsize(output_file)
    
    return ToolResult(
        name="quicktype",
        success=success,
        error_message=None if success else output,
        output_size=output_size,
        execution_time=exec_time,
        output_file=output_file if success else None
    )

def test_schemafy(schema_file: str, output_dir: str) -> ToolResult:
    """Test schemafy tool"""
    output_file = os.path.join(output_dir, "schemafy_output.rs")
    
    cmd = ["schemafy-cli", schema_file]
    
    success, output, exec_time = run_command(cmd)
    
    if success:
        with open(output_file, 'w') as f:
            f.write(output)
        output_size = len(output)
    else:
        output_size = 0
    
    return ToolResult(
        name="schemafy",
        success=success,
        error_message=None if success else output,
        output_size=output_size,
        execution_time=exec_time,
        output_file=output_file if success else None
    )

def create_complex_sample_from_schema(schema: dict) -> dict:
    """Create a complex JSON sample that exercises nested structures and polymorphism."""
    if schema.get("type") == "object":
        result = {}
        properties = schema.get("properties", {})
        
        for prop_name, prop_schema in properties.items():
            prop_type = prop_schema.get("type", "string")
            
            if prop_type == "string":
                # Handle special string types
                if prop_schema.get("format") == "uuid":
                    result[prop_name] = "550e8400-e29b-41d4-a716-446655440000"
                elif prop_schema.get("format") == "date-time":
                    result[prop_name] = "2023-01-01T00:00:00Z"
                elif "enum" in prop_schema:
                    result[prop_name] = prop_schema["enum"][0]
                else:
                    result[prop_name] = "example_string"
            elif prop_type == "number":
                # Use constraints if available
                minimum = prop_schema.get("minimum", 0.0)
                maximum = prop_schema.get("maximum", 100.0)
                result[prop_name] = (minimum + maximum) / 2
            elif prop_type == "integer":
                minimum = prop_schema.get("minimum", 0)
                maximum = prop_schema.get("maximum", 100)
                result[prop_name] = (minimum + maximum) // 2
            elif prop_type == "boolean":
                result[prop_name] = True
            elif prop_type == "array":
                items_schema = prop_schema.get("items", {"type": "string"})
                if items_schema.get("type") == "number":
                    result[prop_name] = [1.0, 2.0, 3.0]
                elif items_schema.get("type") == "object":
                    result[prop_name] = [create_complex_sample_from_schema(items_schema)]
                else:
                    result[prop_name] = ["item1", "item2"]
            elif prop_type == "object":
                result[prop_name] = create_complex_sample_from_schema(prop_schema)
            else:
                result[prop_name] = None
                
        return result
    else:
        return {"complex_example": "data"}

def test_schema_with_all_tools(schema_file: str, include_schemafy: bool) -> SchemaResult:
    """Test a single complex schema with all available tools"""
    schema_name = os.path.basename(schema_file)
    
    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"  💪 Testing HARD schema: {schema_name}...")
        
        # Test each tool
        typify_result = test_typify(schema_file, temp_dir)
        entype_result = test_entype(schema_file, temp_dir)
        quicktype_result = test_quicktype(schema_file, temp_dir)
        schemafy_result = test_schemafy(schema_file, temp_dir) if include_schemafy else None
        
        return SchemaResult(
            schema_name=schema_name,
            typify=typify_result,
            entype=entype_result,
            quicktype=quicktype_result,
            schemafy=schemafy_result
        )

def print_summary_report(results: List[SchemaResult], include_schemafy: bool):
    """Print comprehensive summary report"""
    total_schemas = len(results)
    
    tool_stats = {
        "typify": {"success": 0, "total_time": 0, "total_size": 0},
        "entype": {"success": 0, "total_time": 0, "total_size": 0},
        "quicktype": {"success": 0, "total_time": 0, "total_size": 0}
    }
    
    if include_schemafy:
        tool_stats["schemafy"] = {"success": 0, "total_time": 0, "total_size": 0}
    
    for result in results:
        for tool_name in tool_stats.keys():
            tool_result = getattr(result, tool_name)
            if tool_result and tool_result.success:
                tool_stats[tool_name]["success"] += 1
                tool_stats[tool_name]["total_size"] += tool_result.output_size
            if tool_result:
                tool_stats[tool_name]["total_time"] += tool_result.execution_time
    
    print("\n" + "="*80)
    print("JSON SCHEMA TO RUST BAKEOFF - HARD MODE (COMPLEX SCHEMAS ONLY)")
    print("="*80)
    print(f"Total HARD schemas tested: {total_schemas}")
    print(f"Tools tested: {', '.join(tool_stats.keys())}")
    print("🎯 Focus: Entities, Physics, Polymorphism, Nested Structures")
    print()
    
    # Success rate table
    print("SUCCESS RATES ON HARD SCHEMAS:")
    print("-" * 60)
    print(f"{'Tool':<12} {'Success':<10} {'Rate':<8} {'Avg Time':<12} {'Avg Size':<12}")
    print("-" * 60)
    
    for tool_name, stats in tool_stats.items():
        success_rate = (stats["success"] / total_schemas) * 100
        avg_time = stats["total_time"] / total_schemas
        avg_size = stats["total_size"] / max(stats["success"], 1)
        
        print(f"{tool_name:<12} {stats['success']:<10} {success_rate:>6.1f}% {avg_time:>10.2f}s {avg_size:>10.0f}B")
    
    print("-" * 60)
    
    # Detailed failure analysis
    print("\nFAILURE ANALYSIS (HARD SCHEMAS):")
    print("-" * 40)
    
    for tool_name in tool_stats.keys():
        failures = []
        for result in results:
            tool_result = getattr(result, tool_name)
            if tool_result and not tool_result.success:
                failures.append((result.schema_name, tool_result.error_message))
        
        if failures:
            print(f"\n{tool_name.upper()} FAILURES ({len(failures)}):")
            for schema_name, error in failures[:3]:  # Show first 3 failures
                error_preview = error[:150] + "..." if len(error) > 150 else error
                print(f"  💪 {schema_name}: {error_preview}")
            if len(failures) > 3:
                print(f"  ... and {len(failures) - 3} more")
    
    # Winner determination
    print("\n" + "="*80)
    print("HARD MODE RESULTS RANKING:")
    print("="*80)
    
    tool_ranking = []
    for tool_name, stats in tool_stats.items():
        success_rate = (stats["success"] / total_schemas) * 100
        avg_time = stats["total_time"] / total_schemas
        tool_ranking.append((tool_name, success_rate, avg_time, stats["success"]))
    
    tool_ranking.sort(key=lambda x: (-x[1], x[2]))
    
    print(f"{'Rank':<6} {'Tool':<12} {'Success Rate':<14} {'Schemas':<10} {'Avg Time':<12}")
    print("-" * 60)
    
    for i, (tool_name, success_rate, avg_time, success_count) in enumerate(tool_ranking, 1):
        print(f"{i:<6} {tool_name:<12} {success_rate:>6.1f}%{'':<8} {success_count:<10} {avg_time:>10.2f}s")
    
    winner = tool_ranking[0][0]
    winner_rate = tool_ranking[0][1]
    
    print("-" * 60)
    print(f"🏆 HARD MODE WINNER: {winner.upper()} with {winner_rate:.1f}% success rate!")
    
    # Prediction verification
    print(f"\n🎯 PREDICTION CHECK: User predicted 'typify will still win'")
    if winner == "typify":
        print("✅ PREDICTION CORRECT! Typify wins on hard schemas.")
    else:
        print(f"❌ PREDICTION INCORRECT! {winner.capitalize()} wins on hard schemas.")
    
    print(f"\n💪 HARD MODE: This test used only the most complex schemas with:")
    print(f"   🧬 Entity polymorphism")
    print(f"   ⚡ Physics variables with constraints") 
    print(f"   🏗️  Deeply nested structures")
    print(f"   🔧 x-rust-type hints and transformations")

def main():
    """Main bakeoff execution - HARD MODE"""
    print("💪 JSON Schema to Rust HARD MODE Bakeoff")
    print("Testing only the most complex schemas!")
    print()
    
    # Find all schemas in assembled directory
    assembled_dir = Path("docs/v3/schemas/assembled")
    if not assembled_dir.exists():
        print(f"Error: {assembled_dir} not found!")
        return
    
    all_schemas = list(assembled_dir.glob("*.json"))
    if not all_schemas:
        print(f"Error: No JSON files found in {assembled_dir}")
        return
    
    # Select only the hardest schemas
    hard_schemas = select_hard_schemas(all_schemas)
    if not hard_schemas:
        print("Error: No hard schemas found!")
        return
    
    print()
    
    # Check tool availability
    tools_available = True
    required_tools = ["cargo", "typegen-json", "quicktype"]
    optional_tools = ["schemafy-cli"]
    
    # Check if cargo typify is available
    success, _, _ = run_command(["cargo", "typify", "--help"], timeout=5)
    if not success:
        print("❌ cargo typify not found. Install with: cargo install cargo-typify")
        tools_available = False
    else:
        print("✅ Tool available: cargo typify")
    
    for tool in required_tools[1:]:  # Skip cargo, already checked typify
        success, _, _ = run_command([tool, "--help"], timeout=5)
        if not success:
            print(f"❌ Tool not available: {tool}")
            tools_available = False
        else:
            print(f"✅ Tool available: {tool}")
    
    # Check optional tools
    include_schemafy = False
    for tool in optional_tools:
        success, _, _ = run_command([tool, "--help"], timeout=5)
        if not success:
            print(f"⚠️  Optional tool not available: {tool}")
        else:
            print(f"✅ Optional tool available: {tool}")
            if tool == "schemafy-cli":
                include_schemafy = True
    
    if not tools_available:
        print("\nSome required tools are missing. Please install them first.")
        return
    
    tool_count = 3 + (1 if include_schemafy else 0)
    print(f"\n🚀 Starting HARD MODE bakeoff on {len(hard_schemas)} complex schemas with {tool_count} tools...")
    print("⚡ Testing: Entity polymorphism, physics variables, nested structures...")
    print()
    
    # Run tests
    results = []
    start_time = time.time()
    
    for i, schema_file in enumerate(hard_schemas, 1):
        print(f"[{i}/{len(hard_schemas)}] Testing {schema_file.name}")
        result = test_schema_with_all_tools(str(schema_file), include_schemafy)
        results.append(result)
        
        elapsed = time.time() - start_time
        if i < len(hard_schemas):
            estimated_total = (elapsed / i) * len(hard_schemas)
            remaining = estimated_total - elapsed
            print(f"  Progress: {i}/{len(hard_schemas)} ({i/len(hard_schemas)*100:.1f}%) - Est. {remaining/60:.1f}m remaining")
    
    total_time = time.time() - start_time
    print(f"\n💪 HARD MODE bakeoff completed in {total_time:.1f} seconds")
    
    # Generate report
    print_summary_report(results, include_schemafy)

if __name__ == "__main__":
    main() 