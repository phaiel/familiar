#!/bin/bash
# Schema analysis script for CI/CD
# Uses ast-grep for fast, declarative analysis
#
# Usage:
#   ./scripts/analyze-schema.sh          # Run analysis with ast-grep
#   ./scripts/analyze-schema.sh --legacy # Run legacy Rust analyzer
#   ./scripts/analyze-schema.sh --json   # Output JSON only
#
# Requirements:
#   - ast-grep: brew install ast-grep
#   - Rust toolchain (for building the analyzer)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Check for ast-grep
if ! command -v sg &> /dev/null; then
    echo "❌ ast-grep not found. Install with: brew install ast-grep"
    exit 1
fi

# Parse arguments
USE_AST_GREP="--ast-grep"
JSON_OUTPUT=""
NO_SAVE=""

for arg in "$@"; do
    case $arg in
        --legacy)
            USE_AST_GREP=""
            ;;
        --json)
            JSON_OUTPUT="--json"
            ;;
        --no-save)
            NO_SAVE="--no-save"
            ;;
    esac
done

# Build the analyzer if needed
if [[ ! -f "familiar-core/target/release/analyze_schema" ]]; then
    echo "📦 Building schema analyzer..."
    (cd familiar-core && cargo build --release --bin analyze_schema)
fi

# Run the analyzer
echo "🔍 Running schema analysis..."
./familiar-core/target/release/analyze_schema . $USE_AST_GREP $JSON_OUTPUT $NO_SAVE

# Summary
if [[ -z "$JSON_OUTPUT" && -z "$NO_SAVE" ]]; then
    echo ""
    echo "📊 Reports saved to:"
    echo "   - reports/schema_report.json"
    echo "   - reports/schema_report.txt"
    echo "   - reports/schema_report.html"
fi




