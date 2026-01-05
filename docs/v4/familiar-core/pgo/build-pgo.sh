#!/bin/bash
# Profile-Guided Optimization (PGO) build script for familiar-core
#
# This script builds an optimized familiar-core using PGO:
# 1. Build instrumented binary
# 2. Run typical workloads to collect profiling data
# 3. Rebuild with PGO optimization
#
# See: https://doc.rust-lang.org/beta/rustc/profile-guided-optimization.html

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAMILIAR_CORE_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_DIR="$(dirname "$FAMILIAR_CORE_DIR")"
PGO_DATA_DIR="${SCRIPT_DIR}/pgo-data"
TARGET="$(rustc -vV | grep host | cut -d' ' -f2)"

echo "=== familiar-core PGO Build ==="
echo "Target: ${TARGET}"
echo "PGO data: ${PGO_DATA_DIR}"
echo ""

# STEP 0: Clean up previous profiling data
echo "[1/5] Cleaning previous PGO data..."
rm -rf "${PGO_DATA_DIR}"
mkdir -p "${PGO_DATA_DIR}"

# STEP 1: Build instrumented binary
echo "[2/5] Building instrumented binary..."
cd "${WORKSPACE_DIR}"
RUSTFLAGS="-Cprofile-generate=${PGO_DATA_DIR}" \
    cargo build --release --target="${TARGET}" \
    -p familiar-core \
    --features mcp \
    --bin familiar-mcp

# Also build the codegen xtask if it exists
if [ -d "${WORKSPACE_DIR}/xtask" ]; then
    echo "      Building instrumented xtask..."
    RUSTFLAGS="-Cprofile-generate=${PGO_DATA_DIR}" \
        cargo build --release --target="${TARGET}" -p xtask
fi

# STEP 2: Run typical workloads to generate profiling data
echo "[3/5] Running profiling workloads..."

# Find the schema directory from schema.lock
SCHEMA_DIR=$(cd "${FAMILIAR_CORE_DIR}" && \
    grep -A5 '\[source\]' schema.lock | grep 'path' | cut -d'"' -f2)
SCHEMA_DIR="${FAMILIAR_CORE_DIR}/${SCHEMA_DIR}/versions/latest/json-schema"

if [ ! -d "${SCHEMA_DIR}" ]; then
    echo "Warning: Schema directory not found at ${SCHEMA_DIR}"
    echo "Using embedded schemas only for profiling"
fi

# Profile: MCP graph loading and queries
MCP_BIN="${WORKSPACE_DIR}/target/${TARGET}/release/familiar-mcp"
if [ -f "${MCP_BIN}" ]; then
    echo "      Profiling MCP graph loading..."
    # Generate artifacts (loads graph, iterates all schemas)
    "${MCP_BIN}" --generate-artifacts || true
    
    # Run multiple times to gather more data
    for i in {1..5}; do
        "${MCP_BIN}" --generate-artifacts >/dev/null 2>&1 || true
    done
fi

# Profile: Codegen (if xtask exists)
XTASK_BIN="${WORKSPACE_DIR}/target/${TARGET}/release/xtask"
if [ -f "${XTASK_BIN}" ]; then
    echo "      Profiling codegen..."
    # Run codegen multiple times
    for i in {1..3}; do
        "${XTASK_BIN}" codegen generate --dry-run 2>/dev/null || true
    done
    
    echo "      Profiling schema graph generation..."
    for i in {1..3}; do
        "${XTASK_BIN}" schemas graph 2>/dev/null || true
    done
fi

# STEP 3: Merge profiling data
echo "[4/5] Merging profiling data..."

# Find llvm-profdata
LLVM_PROFDATA=""
if command -v llvm-profdata &> /dev/null; then
    LLVM_PROFDATA="llvm-profdata"
elif [ -f "$(rustc --print sysroot)/lib/rustlib/${TARGET}/bin/llvm-profdata" ]; then
    LLVM_PROFDATA="$(rustc --print sysroot)/lib/rustlib/${TARGET}/bin/llvm-profdata"
else
    # Try to install via rustup
    echo "      Installing llvm-tools-preview..."
    rustup component add llvm-tools-preview
    LLVM_PROFDATA="$(rustc --print sysroot)/lib/rustlib/${TARGET}/bin/llvm-profdata"
fi

if [ ! -f "${LLVM_PROFDATA}" ] && [ "${LLVM_PROFDATA}" != "llvm-profdata" ]; then
    echo "Error: llvm-profdata not found"
    echo "Install with: rustup component add llvm-tools-preview"
    exit 1
fi

# Merge all .profraw files
PROFRAW_COUNT=$(find "${PGO_DATA_DIR}" -name "*.profraw" | wc -l | tr -d ' ')
if [ "${PROFRAW_COUNT}" -eq 0 ]; then
    echo "Warning: No .profraw files generated. Skipping PGO optimization."
    echo "The instrumented binary may not have been run successfully."
    exit 1
fi

echo "      Found ${PROFRAW_COUNT} profile files"
"${LLVM_PROFDATA}" merge -o "${PGO_DATA_DIR}/merged.profdata" "${PGO_DATA_DIR}"

# STEP 4: Build with PGO optimization
echo "[5/5] Building optimized binary with PGO..."
cd "${WORKSPACE_DIR}"

RUSTFLAGS="-Cprofile-use=${PGO_DATA_DIR}/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release --target="${TARGET}" \
    -p familiar-core \
    --features mcp \
    --bin familiar-mcp

if [ -d "${WORKSPACE_DIR}/xtask" ]; then
    echo "      Building optimized xtask..."
    RUSTFLAGS="-Cprofile-use=${PGO_DATA_DIR}/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
        cargo build --release --target="${TARGET}" -p xtask
fi

echo ""
echo "=== PGO Build Complete ==="
echo ""
echo "Optimized binaries:"
echo "  - ${WORKSPACE_DIR}/target/${TARGET}/release/familiar-mcp"
[ -f "${XTASK_BIN}" ] && echo "  - ${WORKSPACE_DIR}/target/${TARGET}/release/xtask"
echo ""
echo "Profile data saved to: ${PGO_DATA_DIR}/merged.profdata"
echo ""
echo "Benchmark with:"
echo "  hyperfine '${MCP_BIN} --generate-artifacts'"


