#!/usr/bin/env bash
# Check for banned Windmill patterns using ast-grep
# This script should be run as part of CI to prevent re-introduction of deprecated code.
#
# Usage: ./scripts/check-banned-patterns.sh
# Exit codes:
#   0 - No banned patterns found
#   1 - Banned patterns detected

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🔍 Checking for banned Windmill patterns..."

# Track if any patterns are found
FOUND_VIOLATIONS=0

# Pattern 1: state.windmill access
echo ""
echo "Checking for state.windmill access..."
if ast-grep --pattern 'state.windmill' --lang rust "${PROJECT_ROOT}/services" 2>/dev/null | grep -q .; then
    echo "❌ Found state.windmill usage!"
    ast-grep --pattern 'state.windmill' --lang rust "${PROJECT_ROOT}/services" 2>/dev/null || true
    FOUND_VIOLATIONS=1
else
    echo "✅ No state.windmill access found"
fi

# Pattern 2: WindmillClient instantiation
echo ""
echo "Checking for WindmillClient::new()..."
if ast-grep --pattern 'WindmillClient::new($$$)' --lang rust "${PROJECT_ROOT}/services" 2>/dev/null | grep -q .; then
    echo "❌ Found WindmillClient::new() usage!"
    ast-grep --pattern 'WindmillClient::new($$$)' --lang rust "${PROJECT_ROOT}/services" 2>/dev/null || true
    FOUND_VIOLATIONS=1
else
    echo "✅ No WindmillClient::new() found"
fi

# Pattern 3: Direct windmill imports
echo ""
echo "Checking for windmill module imports..."
if grep -r "use.*clients::windmill" "${PROJECT_ROOT}/services" --include="*.rs" 2>/dev/null | grep -v "^Binary" | grep -q .; then
    echo "❌ Found windmill client imports!"
    grep -rn "use.*clients::windmill" "${PROJECT_ROOT}/services" --include="*.rs" 2>/dev/null || true
    FOUND_VIOLATIONS=1
else
    echo "✅ No windmill client imports found"
fi

# Pattern 4: WindmillConfig usage
echo ""
echo "Checking for WindmillConfig usage..."
if grep -r "WindmillConfig" "${PROJECT_ROOT}/services" --include="*.rs" 2>/dev/null | grep -v "^Binary" | grep -q .; then
    echo "❌ Found WindmillConfig usage!"
    grep -rn "WindmillConfig" "${PROJECT_ROOT}/services" --include="*.rs" 2>/dev/null || true
    FOUND_VIOLATIONS=1
else
    echo "✅ No WindmillConfig usage found"
fi

# Summary
echo ""
echo "========================================"
if [ $FOUND_VIOLATIONS -eq 1 ]; then
    echo "❌ FAILED: Banned Windmill patterns detected!"
    echo ""
    echo "Action required:"
    echo "  - Remove all Windmill references from services/"
    echo "  - Use EnvelopeProducer.send_command() for messaging"
    echo "  - See docs/SCHEMA_MIGRATION_PROJECT.md for migration guide"
    exit 1
else
    echo "✅ PASSED: No banned Windmill patterns found"
    exit 0
fi

