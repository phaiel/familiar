#!/bin/bash
# Benchmark familiar-core performance before/after PGO
#
# Requires: hyperfine (cargo install hyperfine)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAMILIAR_CORE_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_DIR="$(dirname "$FAMILIAR_CORE_DIR")"
TARGET="$(rustc -vV | grep host | cut -d' ' -f2)"

MCP_BIN="${WORKSPACE_DIR}/target/${TARGET}/release/familiar-mcp"
XTASK_BIN="${WORKSPACE_DIR}/target/${TARGET}/release/xtask"

# Check for hyperfine
if ! command -v hyperfine &> /dev/null; then
    echo "Installing hyperfine..."
    cargo install hyperfine
fi

echo "=== familiar-core Performance Benchmarks ==="
echo ""

# Benchmark MCP graph loading
if [ -f "${MCP_BIN}" ]; then
    echo "### MCP Graph Loading ###"
    hyperfine \
        --warmup 2 \
        --runs 10 \
        "${MCP_BIN} --generate-artifacts"
    echo ""
fi

# Benchmark xtask codegen
if [ -f "${XTASK_BIN}" ]; then
    echo "### Codegen (dry-run) ###"
    hyperfine \
        --warmup 1 \
        --runs 5 \
        "${XTASK_BIN} codegen generate --dry-run"
    echo ""
    
    echo "### Schema Graph Generation ###"
    hyperfine \
        --warmup 1 \
        --runs 5 \
        "${XTASK_BIN} schemas graph"
    echo ""
fi

echo "=== Benchmark Complete ==="


