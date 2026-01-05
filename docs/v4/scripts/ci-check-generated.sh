#!/bin/bash
# CI Check: Verify generated files are up-to-date
#
# This script fails if generated TypeScript or schema files
# differ from what's committed in the repo.
#
# Usage: ./scripts/ci-check-generated.sh
#
# Run this in CI to catch cases where someone modified Rust types
# but forgot to regenerate the TypeScript bindings.

set -e

echo "=== CI Check: Generated Files ==="
echo

# Check TypeScript bindings
echo "1. Checking TypeScript bindings..."
cargo xtask check-ts

echo
echo "✅ All generated files are up-to-date"

