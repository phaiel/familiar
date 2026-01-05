#!/bin/bash
# 
# Familiar Database Migration Script
#
# Runs all SQL migrations in order against the TigerData database.
# Uses Docker to execute psql inside the container (no local psql needed).
#
# Usage:
#   ./scripts/migrate.sh                    # Uses default container
#   ./scripts/migrate.sh --local            # Use local psql with DATABASE_URL
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$PROJECT_ROOT/familiar-core/migrations"

# Container name for TigerData
CONTAINER_NAME="familiar-tigerdata"
DB_USER="familiar"
DB_NAME="familiar"

# Check for --local flag
USE_LOCAL=false
if [ "$1" = "--local" ]; then
    USE_LOCAL=true
fi

echo "=================================================="
echo "Familiar Database Migration"
echo "=================================================="
echo ""
echo "Migrations: $MIGRATIONS_DIR"
echo ""

# Function to run SQL via Docker
run_sql_docker() {
    docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" "$@"
}

# Function to run SQL file via Docker
run_sql_file_docker() {
    local file="$1"
    docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" < "$file"
}

# Check if migrations directory exists
if [ ! -d "$MIGRATIONS_DIR" ]; then
    echo "Error: Migrations directory not found: $MIGRATIONS_DIR"
    exit 1
fi

# Check if container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Error: Container '$CONTAINER_NAME' is not running"
    echo "Start it with: docker compose up -d tigerdata"
    exit 1
fi

# Test database connection
echo "Testing database connection..."
if ! run_sql_docker -c "SELECT 1" > /dev/null 2>&1; then
    echo "Error: Cannot connect to database inside container"
    exit 1
fi
echo "Connection successful!"
echo ""

# Run migrations in order
MIGRATIONS=(
    "001_init_physics_engine.sql"
    "002_conversation_and_tenants.sql"
    "003_auth_and_onboarding.sql"
)

for migration in "${MIGRATIONS[@]}"; do
    MIGRATION_FILE="$MIGRATIONS_DIR/$migration"
    
    if [ ! -f "$MIGRATION_FILE" ]; then
        echo "Warning: Migration file not found: $migration (skipping)"
        continue
    fi
    
    echo "Running migration: $migration"
    
    # Run migration, capturing output
    if run_sql_file_docker "$MIGRATION_FILE" > /dev/null 2>&1; then
        echo "  ✓ Success"
    else
        # Try again with error output to see what went wrong
        echo "  Attempting migration with verbose output..."
        if run_sql_file_docker "$MIGRATION_FILE" 2>&1; then
            echo "  ✓ Success (with warnings)"
        else
            echo "  ✗ Failed"
            echo ""
            echo "Migration failed. Database may be in inconsistent state."
            echo "Check the error above and fix the issue before retrying."
            exit 1
        fi
    fi
done

echo ""
echo "=================================================="
echo "All migrations completed successfully!"
echo "=================================================="

