#!/bin/bash
# Export complete source code from familiar-schemas and familiar-architecture
# Includes: All Rust files, JSON schemas, shell scripts, build files, and graph DOT files
# Focus: Schema processing, graph analysis, and Architecture Compiler implementation
# Excludes: tests, target/, build artifacts, node_modules, .git
#
# Usage:
#   ./export_code_for_llm.sh                          # Export to /tmp/familiar_focus.md
#   ./export_code_for_llm.sh --output /path.md        # Custom output path

set -e

# Parse arguments
OUTPUT_FILE="/tmp/familiar_focus.md"
INCLUDE_GENERATED=true  # Default to including generated files as they're essential

while [[ $# -gt 0 ]]; do
    case $1 in
        --output|-o)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --include-generated|-g)
            INCLUDE_GENERATED=true
            shift
            ;;
        --exclude-generated)
            INCLUDE_GENERATED=false
            shift
            ;;
        *)
            OUTPUT_FILE="$1"
            shift
            ;;
    esac
done

V4_ROOT="/Users/erictheiss/familiar/docs/v4"
SCHEMAS_ROOT="/Users/erictheiss/familiar/familiar-schemas"
ARCHITECTURE_ROOT="/Users/erictheiss/familiar/familiar-architecture"

echo "📦 Exporting Architecture Compiler codebase to: $OUTPUT_FILE"
echo "   Focus: familiar-schemas + familiar-architecture (Architecture Compiler implementation)"
[ "$INCLUDE_GENERATED" = true ] && echo "   Mode: including generated files"

# Start the markdown file
cat > "$OUTPUT_FILE" << 'HEADER'
# Familiar Architecture Compiler Export

This document contains the complete source code from familiar-schemas and familiar-architecture for comprehensive analysis of the Architecture Compiler implementation.

## Included Projects

- `familiar-schemas` - Schema processing, code generation, graph analysis, meta-schema definitions
- `familiar-architecture` - Clean room workspace with contracts and verification crates
- **Focus**: All Rust (.rs), JSON (.json), Shell (.sh), TOML (.toml), and DOT (.dot) files
- **Includes**: Complete Architecture Compiler implementation, schema graph, meta-schemas
- **Excludes**: Tests, target/, build artifacts, node_modules, .git

## Architecture Compiler Context

The Architecture Compiler generates Rust code from JSON schemas with full meta-schema support. Key components:

- **Meta-schemas** define architectural patterns (Action, Component, System, etc.)
- **Schema graph** provides dependency analysis and connectivity
- **Code generation** produces type-safe Rust structs from schema definitions
- **Clean architecture** with proper separation of concerns

---

HEADER

# Function to add a file to the output
add_file() {
    local filepath="$1"
    local display_path="$2"

    # Skip build artifacts, tests, and large verbose files
    case "$display_path" in
        */target/*|*/.cargo/*|*.lock|package-lock.json|yarn.lock)
            echo "  [skipped build artifact: $display_path]"
            return
            ;;
        */tests/*|*_test.rs|*_tests.rs|*/test/*)
            echo "  [skipped test file: $display_path]"
            return
            ;;
    esac

    # Skip generated files unless --include-generated is used
    if [ "$INCLUDE_GENERATED" = false ]; then
        case "$display_path" in
            */generated.rs|*generated.rs)
                echo "  [skipped generated file: $display_path]"
                return
                ;;
        esac
    fi

    # Get file extension for syntax highlighting
    local ext="${filepath##*.}"
    local lang=""
    case "$ext" in
        rs) lang="rust" ;;
        toml) lang="toml" ;;
        yaml|yml) lang="yaml" ;;
        json) lang="json" ;;
        sh) lang="bash" ;;
        md) lang="markdown" ;;
        proto) lang="protobuf" ;;
        dot) lang="dot" ;;
        *) lang="" ;;
    esac

    echo "" >> "$OUTPUT_FILE"
    echo "## \`$display_path\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "\`\`\`$lang" >> "$OUTPUT_FILE"
    cat "$filepath" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
}

# Function to process directory with selective inclusion
process_directory() {
    local dir="$1"
    local base_name="$2"
    local include_patterns="$3"  # Optional: specific patterns to include

    echo "Processing: $base_name"

    # Build exclusion patterns
    local excludes=(
        "*/target/*"
        "*/.cargo/*"
        "*/.git/*"
        "*Cargo.lock"
        "*/node_modules/*"
        "*/.next/*"
        "*/coverage/*"
        "*/.turbo/*"
        "*/.venv/*"
        "*/venv/*"
        "*/.env/*"
        "*/site-packages/*"
        "*/dist/*"
        "*/build/*"
        "*/__pycache__/*"
        "*/tests/*"
        "*_test.rs"
        "*_tests.rs"
        "*/test/*"
        "*/examples/*"
        "*/benches/*"
        "*/migrations/*"
        "*/.llm_cache.json"
        "*/reports/*"
    )

    # Build find command
    local find_cmd="find \"$dir\" -type f \\( -name \"*.rs\" -o -name \"*.toml\" -o -name \"*.yaml\" -o -name \"*.yml\" -o -name \"*.json\" -o -name \"*.sh\" -o -name \"*.md\" \\)"

    for pattern in "${excludes[@]}"; do
        find_cmd+=" ! -path \"$pattern\""
    done

    find_cmd+=" ! -name \".DS_Store\" ! -name \"package-lock.json\" ! -name \"yarn.lock\""

    # Apply selective inclusion if specified
    if [ -n "$include_patterns" ]; then
        local temp_file=$(mktemp)
        eval "$find_cmd" | grep -E "$include_patterns" > "$temp_file"
        while read -r file; do
            local rel_path="${file#$dir/}"
            add_file "$file" "$base_name/$rel_path"
        done < "$temp_file"
        rm "$temp_file"
    else
        eval "$find_cmd" | sort | while read -r file; do
            local rel_path="${file#$dir/}"
            add_file "$file" "$base_name/$rel_path"
        done
    fi
}

# Process crate completely (all src/ files)
process_crate() {
    local dir="$1"
    local base_name="$2"

    echo "Processing $base_name (complete)"

    # Include all Rust, JSON, Shell, and TOML files from src/
    if [ -d "$dir/src" ]; then
        find "$dir/src" \( -name "*.rs" -o -name "*.json" -o -name "*.sh" -o -name "*.toml" \) -type f | sort | while read -r file; do
            rel_path="${file#$dir/}"
            add_file "$file" "$base_name/$rel_path"
        done
    fi

    # Include build files
    if [ -f "$dir/Cargo.toml" ]; then
        add_file "$dir/Cargo.toml" "$base_name/Cargo.toml"
    fi
    if [ -f "$dir/schemas.toml" ]; then
        add_file "$dir/schemas.toml" "$base_name/schemas.toml"
    fi

    # For familiar-schemas, include all JSON schemas and the DOT graph file
    if [[ "$base_name" == "familiar-schemas" ]]; then
        if [ -d "$dir/versions/latest/json-schema" ]; then
            echo "  Including all JSON schemas..."
            find "$dir/versions/latest/json-schema" -name "*.json" | sort | while read -r file; do
                rel_path="${file#$dir/}"
                add_file "$file" "$base_name/$rel_path"
            done
        fi

        # Include the schema graph DOT file
        if [ -f "$dir/schemas.dot" ]; then
            echo "  Including schema graph DOT file..."
            add_file "$dir/schemas.dot" "$base_name/schemas.dot"
        fi
    fi
}


# Process the Architecture Compiler crates
crates=(
    "$SCHEMAS_ROOT:familiar-schemas"
    "$ARCHITECTURE_ROOT:familiar-architecture"
)

for crate_spec in "${crates[@]}"; do
    IFS=':' read -r crate_dir crate_name <<< "$crate_spec"
    if [ -d "$crate_dir" ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "# $crate_name" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        process_crate "$crate_dir" "$crate_name"
    fi
done

# Summary
echo "" >> "$OUTPUT_FILE"
echo "---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "*End of codebase export*" >> "$OUTPUT_FILE"

# Stats
FILE_COUNT=$(grep -c "^## \`" "$OUTPUT_FILE" || echo "0")
LINES=$(wc -l < "$OUTPUT_FILE" | tr -d ' ')
SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)

echo ""
echo "✅ Export complete!"
echo "   Files: $FILE_COUNT"
echo "   Lines: $LINES"
echo "   Size:  $SIZE"
echo "   Output: $OUTPUT_FILE"
echo ""
echo "To use:"
echo "   cat $OUTPUT_FILE | pbcopy    # Copy to clipboard (macOS)"
echo "   open $OUTPUT_FILE            # Open in editor"
