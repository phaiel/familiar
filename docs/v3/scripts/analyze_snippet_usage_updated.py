#!/usr/bin/env python3
"""
Updated Snippet Usage Analysis Script
Analyzes snippet usage patterns in source schemas (excluding assembled)
FIXED: Transitive closure only for types, flags transitive fields as optimization opportunities
"""

import json
import os
import re
from collections import defaultdict, Counter
from pathlib import Path
from datetime import datetime

def load_json_file(filepath):
    """Load and parse a JSON file safely"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            return json.load(f)
    except (json.JSONDecodeError, FileNotFoundError, UnicodeDecodeError) as e:
        print(f"⚠️  Error loading {filepath}: {e}")
        return None

def find_snippet_references(schema_content, filepath):
    """Find all $ref references to snippets in a schema"""
    refs = []
    
    # Check if this file is in a snippets directory
    is_snippet_file = "snippets" in str(filepath)
    
    def extract_refs(obj, path=""):
        if isinstance(obj, dict):
            for key, value in obj.items():
                current_path = f"{path}.{key}" if path else key
                if key == "$ref" and isinstance(value, str):
                    # Check if this is a snippet reference
                    is_snippet_ref = False
                    snippet_category = "other"
                    
                    if "snippets" in value:
                        # Absolute or relative path containing "snippets"
                        is_snippet_ref = True
                        snippet_category = "field" if "/fields/" in value else "type" if "/types/" in value else "other"
                    elif is_snippet_file and (value.startswith("./") or value.startswith("../")):
                        # Relative reference from within snippets directory
                        is_snippet_ref = True
                        # Determine category from current file's path since relative refs don't show structure
                        if "/fields/" in str(filepath):
                            snippet_category = "field"  # Field snippet referencing another field
                        elif "/types/" in str(filepath):
                            snippet_category = "type"   # Type snippet referencing another type
                    
                    if is_snippet_ref:
                        # Extract snippet name from path
                        snippet_name = Path(value).stem
                        refs.append({
                            'snippet': snippet_name,
                            'path': value,
                            'location': current_path,
                            'category': snippet_category
                        })
                
                extract_refs(value, current_path)
        elif isinstance(obj, list):
            for i, item in enumerate(obj):
                extract_refs(item, f"{path}[{i}]")
    
    if schema_content:
        extract_refs(schema_content)
    
    return refs

def build_snippet_dependency_graph(snippets_dir):
    """Build a graph of snippet-to-snippet dependencies with categories"""
    snippet_deps = defaultdict(set)  # snippet -> set of snippets it references
    snippet_files = {}  # snippet_name -> file_path
    snippet_categories = {}  # snippet_name -> category (field/type/other)
    
    if not snippets_dir.exists():
        return snippet_deps, snippet_files, snippet_categories
    
    # Load all snippet files
    for snippet_file in snippets_dir.rglob("*.json"):
        snippet_name = snippet_file.stem
        snippet_files[snippet_name] = snippet_file
        
        # Determine category from path
        if "/fields/" in str(snippet_file):
            snippet_categories[snippet_name] = "field"
        elif "/types/" in str(snippet_file):
            snippet_categories[snippet_name] = "type"
        else:
            snippet_categories[snippet_name] = "other"
        
        content = load_json_file(snippet_file)
        if content:
            # Find references to other snippets
            refs = find_snippet_references(content, snippet_file)
            for ref in refs:
                snippet_deps[snippet_name].add(ref['snippet'])
    
    return snippet_deps, snippet_files, snippet_categories

def find_transitive_type_usage(direct_used_snippets, snippet_deps, snippet_categories):
    """Find all TYPE snippets that are transitively used. Fields should be directly referenced."""
    used_snippets = set(direct_used_snippets)
    transitive_types = set()
    changed = True
    
    # Keep expanding until no new snippets are found
    while changed:
        changed = False
        new_snippets = set()
        
        for used_snippet in used_snippets:
            for referenced_snippet in snippet_deps.get(used_snippet, set()):
                if referenced_snippet not in used_snippets:
                    # Only add types transitively - fields should be direct
                    if snippet_categories.get(referenced_snippet) == "type":
                        new_snippets.add(referenced_snippet)
                        transitive_types.add(referenced_snippet)
                        changed = True
        
        used_snippets.update(new_snippets)
    
    return used_snippets, transitive_types

def find_transitive_field_issues(direct_used_snippets, snippet_deps, snippet_categories):
    """Find fields that are only used transitively (optimization opportunities)"""
    transitive_field_issues = set()
    
    # Find all snippets reachable from direct usage
    all_reachable = set(direct_used_snippets)
    changed = True
    
    while changed:
        changed = False
        for snippet in list(all_reachable):
            for referenced in snippet_deps.get(snippet, set()):
                if referenced not in all_reachable:
                    all_reachable.add(referenced)
                    changed = True
                    # If it's a field reached transitively, flag it
                    if snippet_categories.get(referenced) == "field" and referenced not in direct_used_snippets:
                        transitive_field_issues.add(referenced)
    
    return transitive_field_issues

def count_inline_definitions(schema_content):
    """Count inline type definitions that could potentially be snippets"""
    inline_count = 0
    patterns = {
        'type_definitions': 0,
        'enum_definitions': 0,
        'constraint_patterns': 0,
        'property_patterns': 0
    }
    
    def count_patterns(obj):
        nonlocal inline_count
        if isinstance(obj, dict):
            # Count type definitions
            if 'type' in obj and 'properties' in obj:
                patterns['type_definitions'] += 1
                inline_count += 1
            
            # Count enum definitions
            if 'enum' in obj:
                patterns['enum_definitions'] += 1
                inline_count += 1
            
            # Count constraint patterns
            if 'minimum' in obj or 'maximum' in obj:
                patterns['constraint_patterns'] += 1
                inline_count += 1
            
            # Count common property patterns
            for key in ['entity_type', 'status', 'description', 'entity_id']:
                if key in obj:
                    patterns['property_patterns'] += 1
            
            for value in obj.values():
                count_patterns(value)
        elif isinstance(obj, list):
            for item in obj:
                count_patterns(item)
    
    if schema_content:
        count_patterns(schema_content)
    
    return inline_count, patterns

def analyze_schemas():
    """Main analysis function"""
    print("🔍 Starting Updated Snippet Usage Analysis (transitive types only)...")
    
    # Paths
    schemas_dir = Path("schemas")
    snippets_dir = schemas_dir / "snippets"
    
    # Build snippet dependency graph
    snippet_deps, snippet_files, snippet_categories = build_snippet_dependency_graph(snippets_dir)
    all_snippets = set(snippet_files.keys())
    
    print(f"📁 Found {len(all_snippets)} total snippets")
    print(f"🔗 Found {sum(len(deps) for deps in snippet_deps.values())} snippet-to-snippet references")
    
    # Categorize snippets
    field_snippets = {s for s, cat in snippet_categories.items() if cat == "field"}
    type_snippets = {s for s, cat in snippet_categories.items() if cat == "type"}
    other_snippets = {s for s, cat in snippet_categories.items() if cat == "other"}
    
    print(f"📊 Snippet breakdown: {len(field_snippets)} fields, {len(type_snippets)} types, {len(other_snippets)} other")
    
    # Find all source schemas (excluding assembled and snippets)
    source_schemas = []
    for schema_file in schemas_dir.rglob("*.json"):
        if "assembled" not in str(schema_file) and "snippets" not in str(schema_file):
            source_schemas.append(schema_file)
    
    print(f"📄 Found {len(source_schemas)} source schemas")
    
    # Analyze direct snippet usage from schemas
    direct_used_snippets = set()
    snippet_usage = defaultdict(list)
    total_snippet_references = 0
    total_inline_definitions = 0
    schema_inline_counts = {}
    
    constraint_patterns = Counter()
    property_patterns = Counter()
    
    for schema_file in source_schemas:
        schema_content = load_json_file(schema_file)
        if not schema_content:
            continue
        
        # Find snippet references
        refs = find_snippet_references(schema_content, schema_file)
        total_snippet_references += len(refs)
        
        for ref in refs:
            direct_used_snippets.add(ref['snippet'])
            snippet_usage[ref['snippet']].append({
                'file': str(schema_file),
                'location': ref['location'],
                'path': ref['path'],
                'category': ref['category']
            })
        
        # Count inline definitions
        inline_count, patterns = count_inline_definitions(schema_content)
        total_inline_definitions += inline_count
        schema_inline_counts[str(schema_file)] = {
            'count': inline_count,
            'patterns': patterns
        }
        
        # Aggregate pattern counts
        for pattern_type, count in patterns.items():
            constraint_patterns[pattern_type] += count
    
    # Find transitive type usage (only for types)
    all_used_snippets, transitive_types = find_transitive_type_usage(direct_used_snippets, snippet_deps, snippet_categories)
    
    # Find transitive field issues (optimization opportunities)
    transitive_field_issues = find_transitive_field_issues(direct_used_snippets, snippet_deps, snippet_categories)
    
    print(f"📊 Direct snippet usage: {len(direct_used_snippets)} snippets")
    print(f"📊 Total snippet usage (including transitive types): {len(all_used_snippets)} snippets")
    print(f"⚠️  Transitive field issues: {len(transitive_field_issues)} fields")
    
    # Calculate metrics
    orphaned_snippets = all_snippets - all_used_snippets
    usage_rate = (len(all_used_snippets) / len(all_snippets)) * 100 if all_snippets else 0
    
    # Generate report
    report = generate_report(
        total_snippets=len(all_snippets),
        direct_used_snippets=direct_used_snippets,
        all_used_snippets=all_used_snippets,
        transitive_types=transitive_types,
        transitive_field_issues=transitive_field_issues,
        orphaned_snippets=orphaned_snippets,
        snippet_usage=snippet_usage,
        snippet_deps=snippet_deps,
        snippet_categories=snippet_categories,
        field_snippets=field_snippets,
        type_snippets=type_snippets,
        total_schemas=len(source_schemas),
        total_references=total_snippet_references,
        total_inline=total_inline_definitions,
        schema_inline_counts=schema_inline_counts,
        constraint_patterns=constraint_patterns,
        usage_rate=usage_rate
    )
    
    return report

def generate_report(total_snippets, direct_used_snippets, all_used_snippets, transitive_types,
                   transitive_field_issues, orphaned_snippets, snippet_usage, snippet_deps, 
                   snippet_categories, field_snippets, type_snippets, total_schemas, 
                   total_references, total_inline, schema_inline_counts, constraint_patterns, usage_rate):
    """Generate the analysis report"""
    
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    report = f"""# Updated Source Schema Snippet Usage Analysis Report

## Executive Summary

**UPDATED ANALYSIS** (excluding assembled directory, **TRANSITIVE TYPES ONLY**):
- **{total_schemas} source schemas** analyzed
- **{total_snippets} snippets exist** ({len(field_snippets)} fields, {len(type_snippets)} types)
- **{len(all_used_snippets)} snippets actually used** (including transitive types)
- **{len(direct_used_snippets)} directly used**, **{len(transitive_types)} types transitively used**
- **{len(transitive_field_issues)} fields used transitively** (medium priority optimization)
- **{len(orphaned_snippets)} truly orphaned snippets** 
- **{total_inline} inline definitions** in source schemas
- **{total_references} total snippet references**
- **Usage rate: {usage_rate:.1f}%**

## Key Findings

### 1. Snippet Usage Status ({len(all_used_snippets)}/{total_snippets} snippets used)

**✅ DIRECTLY USED SNIPPETS ({len(direct_used_snippets)} total):**
"""
    
    # List directly used snippets by category
    direct_fields = {s for s in direct_used_snippets if snippet_categories.get(s) == "field"}
    direct_types = {s for s in direct_used_snippets if snippet_categories.get(s) == "type"}
    direct_other = {s for s in direct_used_snippets if snippet_categories.get(s) == "other"}
    
    report += f"- **Fields** - {len(direct_fields)} snippets directly used\n"
    for snippet in sorted(list(direct_fields)[:10]):
        usage_count = len(snippet_usage[snippet])
        report += f"  - `{snippet}` (used {usage_count}x)\n"
    if len(direct_fields) > 10:
        report += f"  - ... and {len(direct_fields) - 10} more\n"
    
    report += f"- **Types** - {len(direct_types)} snippets directly used\n"
    for snippet in sorted(list(direct_types)[:10]):
        usage_count = len(snippet_usage[snippet])
        report += f"  - `{snippet}` (used {usage_count}x)\n"
    if len(direct_types) > 10:
        report += f"  - ... and {len(direct_types) - 10} more\n"
    
    if direct_other:
        report += f"- **Other** - {len(direct_other)} snippets directly used\n"
    
    report += f"""
**🔗 TRANSITIVELY USED TYPES ({len(transitive_types)} total - CORRECT BEHAVIOR):**
"""
    
    if transitive_types:
        for snippet in sorted(transitive_types):
            # Find which snippets reference this one
            referencing_snippets = [s for s, deps in snippet_deps.items() if snippet in deps and s in direct_used_snippets]
            if referencing_snippets:
                report += f"- `{snippet}` ← used by `{', '.join(referencing_snippets)}`\n"
            else:
                report += f"- `{snippet}` ← transitive chain\n"
    else:
        report += "- None (all type snippets are directly referenced)\n"
    
    report += f"""
**⚠️ TRANSITIVELY USED FIELDS ({len(transitive_field_issues)} total - MEDIUM PRIORITY OPTIMIZATION):**
"""
    
    if transitive_field_issues:
        report += "**These fields should be directly referenced by schemas, not through other snippets:**\n"
        for snippet in sorted(transitive_field_issues):
            # Find which snippets reference this one
            referencing_snippets = [s for s, deps in snippet_deps.items() if snippet in deps]
            if referencing_snippets:
                report += f"- `{snippet}` ← currently used via `{', '.join(referencing_snippets)}`\n"
            else:
                report += f"- `{snippet}` ← indirect usage\n"
    else:
        report += "- None! All field snippets are properly directly referenced ✅\n"
    
    report += f"""
**🔍 TRULY ORPHANED SNIPPETS ({len(orphaned_snippets)} total):**
"""
    
    if orphaned_snippets:
        orphaned_fields = {s for s in orphaned_snippets if snippet_categories.get(s) == "field"}
        orphaned_types = {s for s in orphaned_snippets if snippet_categories.get(s) == "type"}
        orphaned_other = {s for s in orphaned_snippets if snippet_categories.get(s) == "other"}
        
        if orphaned_fields:
            report += f"**Orphaned Fields ({len(orphaned_fields)}):**\n"
            for snippet in sorted(orphaned_fields):
                report += f"- `{snippet}`\n"
        
        if orphaned_types:
            report += f"**Orphaned Types ({len(orphaned_types)}):**\n"
            for snippet in sorted(orphaned_types):
                report += f"- `{snippet}`\n"
        
        if orphaned_other:
            report += f"**Orphaned Other ({len(orphaned_other)}):**\n"
            for snippet in sorted(orphaned_other):
                report += f"- `{snippet}`\n"
    else:
        report += "- None! All snippets are being used ✅\n"
    
    # Top schemas with inline definitions
    top_inline_schemas = sorted(
        [(path, data['count']) for path, data in schema_inline_counts.items() if data['count'] > 0],
        key=lambda x: x[1],
        reverse=True
    )[:10]
    
    report += f"""
### 2. Source Schemas with Most Inline Definitions

**Top Candidates for Optimization:**
"""
    
    for i, (schema_path, count) in enumerate(top_inline_schemas, 1):
        schema_name = Path(schema_path).name
        report += f"{i}. `{schema_name}` - {count} inline definitions\n"
    
    report += f"""
### 3. Pattern Analysis

**Constraint Patterns:**
"""
    
    for pattern_type, count in constraint_patterns.most_common():
        report += f"- `{pattern_type}` - {count} occurrences\n"
    
    # Most used snippets
    most_used = sorted(
        [(snippet, len(usages)) for snippet, usages in snippet_usage.items()],
        key=lambda x: x[1],
        reverse=True
    )[:10]
    
    report += f"""
### 4. Most Referenced Snippets

**Top 10 Most Used Snippets:**
"""
    
    for snippet, count in most_used:
        category = snippet_categories.get(snippet, "unknown")
        report += f"- `{snippet}` ({category}) - used {count} times\n"
    
    # Snippet dependency analysis
    report += f"""
### 5. Snippet Dependency Chains

**Valid type-to-type dependencies:**
"""
    
    type_deps = {s: deps for s, deps in snippet_deps.items() 
                if snippet_categories.get(s) == "type" and deps}
    
    for snippet in sorted(type_deps.keys()):
        deps = ', '.join(f"`{dep}`" for dep in sorted(type_deps[snippet]))
        report += f"- `{snippet}` → {deps}\n"
    
    # Field dependencies (should be minimal)
    field_deps = {s: deps for s, deps in snippet_deps.items() 
                 if snippet_categories.get(s) == "field" and deps}
    
    if field_deps:
        report += f"""
**Field-to-other dependencies (review these):**
"""
        for snippet in sorted(field_deps.keys()):
            deps = ', '.join(f"`{dep}`" for dep in sorted(field_deps[snippet]))
            report += f"- `{snippet}` → {deps}\n"
    
    report += f"""
## Detailed Usage Analysis

### Direct Snippet Usage Details
"""
    
    for snippet in sorted(direct_used_snippets):
        usages = snippet_usage[snippet]
        category = snippet_categories.get(snippet, "unknown")
        report += f"""
#### `{snippet}` ({category}, used {len(usages)} times)
"""
        for usage in usages[:5]:  # Show first 5 usages
            file_name = Path(usage['file']).name
            report += f"- `{file_name}` at `{usage['location']}`\n"
        
        if len(usages) > 5:
            report += f"- ... and {len(usages) - 5} more usages\n"
    
    report += f"""
## Recommendations

### Priority 1: Address Remaining {len(orphaned_snippets)} Orphaned Snippets
"""
    
    if orphaned_snippets:
        report += "**Consider removing or finding uses for:**\n"
        for snippet in sorted(orphaned_snippets):
            category = snippet_categories.get(snippet, "unknown")
            report += f"- `{snippet}` ({category}) - Review if still needed\n"
    else:
        report += "✅ No orphaned snippets! Excellent snippet utilization.\n"
    
    report += f"""
### Priority 2: Fix {len(transitive_field_issues)} Transitive Field Usage (Medium Priority)
**Fields should be directly referenced by schemas, not through other snippets:**
"""
    
    if transitive_field_issues:
        for snippet in sorted(transitive_field_issues):
            report += f"- `{snippet}` - Make schemas reference this directly\n"
    else:
        report += "✅ All fields are properly directly referenced.\n"
    
    report += f"""
### Priority 3: Optimize High-Inline Schemas
**Focus on schemas with most inline definitions for snippet extraction:**
"""
    
    for i, (schema_path, count) in enumerate(top_inline_schemas[:5], 1):
        schema_name = Path(schema_path).name
        report += f"{i}. `{schema_name}` - {count} inline definitions\n"
    
    report += f"""
### Priority 4: Pattern Consolidation
**Common patterns that could be standardized:**
"""
    
    for pattern_type, count in constraint_patterns.most_common(5):
        report += f"- `{pattern_type}` appears {count} times - consider snippet consolidation\n"
    
    report += f"""
## Success Metrics

**Current Status:**
- Total snippets: {total_snippets}
- Used snippets: {len(all_used_snippets)} ({usage_rate:.1f}%)
  - Directly used: {len(direct_used_snippets)}
  - Transitive types: {len(transitive_types)} (correct)
  - Transitive fields: {len(transitive_field_issues)} (needs optimization)
- Orphaned snippets: {len(orphaned_snippets)}
- Total snippet references: {total_references}
- Inline definitions: {total_inline}
- Schemas analyzed: {total_schemas}

**Quality Indicators:**
- Schema validation: Run `make validate` to verify
- Usage efficiency: {usage_rate:.1f}% snippet utilization
- Reference density: {total_references/total_schemas:.1f} refs per schema
- Architecture: Types use transitive deps ✅, Fields should be direct

---

*Generated by updated snippet analysis script (transitive types only) on {timestamp}*
"""
    
    return report

if __name__ == "__main__":
    try:
        report = analyze_schemas()
        
        # Write report
        output_file = Path("../../SNIPPET_ANALYSIS_REPORT.md")
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(report)
        
        print(f"📊 Analysis complete! Report written to {output_file}")
        print("\n" + "="*60)
        print("SUMMARY")
        print("="*60)
        
        # Print key metrics
        lines = report.split('\n')
        for line in lines:
            if 'source schemas' in line and 'analyzed' in line:
                print(line)
            elif 'snippets exist' in line:
                print(line)
            elif 'Usage rate:' in line:
                print(line)
            elif 'fields used transitively' in line:
                print(line)
        
    except Exception as e:
        print(f"❌ Analysis failed: {e}")
        import traceback
        traceback.print_exc() 