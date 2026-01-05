#!/bin/bash
# Schema Analysis Script
#
# Analyzes schema usage across the familiar codebase.
# Dynamically discovers all schemas and suggests improvements.
#
# Usage:
#   ./scripts/analyze_schema.sh              # Human-readable report
#   ./scripts/analyze_schema.sh --json       # JSON output for CI/CD
#   ./scripts/analyze_schema.sh --fix        # Show what would be fixed
#   ./scripts/analyze_schema.sh --fix-apply  # Apply fixes (be careful!)
#   ./scripts/analyze_schema.sh --help       # Show all options
#
# Features:
#   - Dynamic discovery (no hardcoded paths)
#   - Detects unused schemas (Rust, TypeScript, Python)
#   - Suggests schema consolidation
#   - Auto-fix support with dry-run safety

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Build the analyzer if needed
if [ ! -f "$PROJECT_ROOT/familiar-core/target/debug/analyze_schema" ] || \
   [ "$PROJECT_ROOT/familiar-core/src/analysis/scanner.rs" -nt "$PROJECT_ROOT/familiar-core/target/debug/analyze_schema" ]; then
    echo "📦 Building schema analyzer..."
    cd "$PROJECT_ROOT/familiar-core"
    cargo build --bin analyze_schema --quiet 2>/dev/null || cargo build --bin analyze_schema
fi

# Run analysis
echo ""
"$PROJECT_ROOT/familiar-core/target/debug/analyze_schema" "$PROJECT_ROOT" "$@"
