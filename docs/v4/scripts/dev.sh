#!/bin/bash
#
# Familiar Development Startup Script
#
# Starts all services and runs database migrations for local development.
#
# Usage:
#   ./scripts/dev.sh          # Start everything
#   ./scripts/dev.sh db       # Start only databases
#   ./scripts/dev.sh migrate  # Run migrations only
#   ./scripts/dev.sh api      # Start API only
#   ./scripts/dev.sh ui       # Start UI only
#   ./scripts/dev.sh stop     # Stop all services

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Change to project root
cd "$PROJECT_ROOT"

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Set defaults if not in .env
export DATABASE_URL="${DATABASE_URL:-postgres://familiar:familiarpass@localhost:5432/familiar}"
export QDRANT_URL="${QDRANT_URL:-http://localhost:6333}"
export PORT="${PORT:-3001}"

wait_for_postgres() {
    log_info "Waiting for PostgreSQL to be ready..."
    local retries=30
    while [ $retries -gt 0 ]; do
        # Use docker exec to check if postgres is ready inside the container
        if docker exec familiar-tigerdata pg_isready -U familiar -d familiar > /dev/null 2>&1; then
            log_success "PostgreSQL is ready!"
            return 0
        fi
        retries=$((retries - 1))
        sleep 1
    done
    log_error "PostgreSQL did not become ready in time"
    return 1
}

wait_for_qdrant() {
    log_info "Waiting for Qdrant to be ready..."
    local retries=30
    while [ $retries -gt 0 ]; do
        if curl -s http://localhost:6333/readiness > /dev/null 2>&1; then
            log_success "Qdrant is ready!"
            return 0
        fi
        retries=$((retries - 1))
        sleep 1
    done
    log_warning "Qdrant did not become ready (non-critical)"
    return 0
}

start_databases() {
    log_info "Starting databases (TigerData, Qdrant, Windmill, MinIO)..."
    docker compose up -d tigerdata qdrant windmill_db windmill_server windmill_worker windmill_lsp minio
    
    wait_for_postgres
    wait_for_qdrant
}

run_migrations() {
    log_info "Running database migrations..."
    
    if [ ! -f "$SCRIPT_DIR/migrate.sh" ]; then
        log_error "migrate.sh not found"
        return 1
    fi
    
    "$SCRIPT_DIR/migrate.sh" "$DATABASE_URL"
}

start_api() {
    log_info "Starting familiar-api on port $PORT..."
    cd "$PROJECT_ROOT/services/familiar-api"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Install Rust: https://rustup.rs"
        return 1
    fi
    
    cargo run &
    API_PID=$!
    log_success "API started (PID: $API_PID)"
}

start_ui() {
    log_info "Starting familiar-ui on port 3000..."
    cd "$PROJECT_ROOT/services/familiar-ui"
    
    # Check if npm is available
    if ! command -v npm &> /dev/null; then
        log_error "npm not found. Install Node.js: https://nodejs.org"
        return 1
    fi
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing UI dependencies..."
        npm install
    fi
    
    npm run dev &
    UI_PID=$!
    log_success "UI started (PID: $UI_PID)"
}

stop_all() {
    log_info "Stopping all services..."
    docker compose down
    
    # Kill any running dev processes
    pkill -f "cargo run" 2>/dev/null || true
    pkill -f "next dev" 2>/dev/null || true
    pkill -f "npm run dev" 2>/dev/null || true
    
    log_success "All services stopped"
}

print_status() {
    echo ""
    echo "=================================================="
    echo "Familiar Development Environment"
    echo "=================================================="
    echo ""
    echo "Services:"
    echo "  - TigerData (PostgreSQL): localhost:5432"
    echo "  - Qdrant:                 localhost:6333"
    echo "  - Windmill:               http://localhost:8001"
    echo "  - MinIO:                  http://localhost:9002"
    echo "  - MinIO Console:          http://localhost:9003"
    echo ""
    echo "Application:"
    echo "  - API:                    http://localhost:$PORT"
    echo "  - UI:                     http://localhost:3000"
    echo ""
    echo "Database URL: $DATABASE_URL"
    echo ""
    echo "Press Ctrl+C to stop"
    echo "=================================================="
}

# Main command handling
case "${1:-all}" in
    db|databases)
        start_databases
        ;;
    migrate|migrations)
        run_migrations
        ;;
    api)
        start_api
        wait
        ;;
    ui)
        start_ui
        wait
        ;;
    stop)
        stop_all
        ;;
    all|"")
        echo ""
        echo "🧵 Starting Familiar Development Environment..."
        echo ""
        
        start_databases
        run_migrations
        start_api
        start_ui
        
        print_status
        
        # Wait for any background process to exit
        wait
        ;;
    *)
        echo "Usage: $0 [db|migrate|api|ui|stop|all]"
        echo ""
        echo "Commands:"
        echo "  db       - Start databases only (Docker)"
        echo "  migrate  - Run database migrations"
        echo "  api      - Start familiar-api"
        echo "  ui       - Start familiar-ui"
        echo "  stop     - Stop all services"
        echo "  all      - Start everything (default)"
        exit 1
        ;;
esac

