#!/bin/bash

# aw-datastore Test Runner
# Automated test execution with Docker PostgreSQL management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DB_URL="postgresql://test:test@localhost:5432/aw_test"
DOCKER_COMPOSE="docker-compose -f docker-compose.test.yml"
COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default mode
MODE="all"
VERBOSE=""

# Functions
print_header() {
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

OPTIONS:
    -m, --mode <mode>       Test mode: all, sqlite, postgres (default: all)
    -v, --verbose          Show detailed output
    -c, --coverage         Generate coverage report
    -b, --benchmark        Run benchmarks
    -h, --help             Show this help message

EXAMPLES:
    $0                      # Run all tests (SQLite + PostgreSQL)
    $0 --mode sqlite        # Run SQLite tests only
    $0 --mode postgres      # Run PostgreSQL tests
    $0 -c                   # Run all tests and coverage
    $0 -b                   # Run benchmarks

EOF
}

cleanup() {
    print_info "Cleaning up..."
    $DOCKER_COMPOSE down 2>/dev/null || true
}

trap cleanup EXIT

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            shift
            ;;
        -c|--coverage)
            COVERAGE="true"
            shift
            ;;
        -b|--benchmark)
            BENCHMARK="true"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate mode
case "$MODE" in
    all|sqlite|postgres)
        ;;
    *)
        print_error "Invalid mode: $MODE"
        usage
        exit 1
        ;;
esac

print_header "aw-datastore Test Runner"
print_info "Mode: $MODE"
print_info "Working directory: $COMPOSE_DIR"

# Run tests
run_sqlite_tests() {
    print_header "Running SQLite Tests"
    if cargo test --lib --tests --features sqlite $VERBOSE; then
        print_success "SQLite tests passed"
        return 0
    else
        print_error "SQLite tests failed"
        return 1
    fi
}

run_postgres_tests() {
    print_header "Starting PostgreSQL"
    
    cd "$COMPOSE_DIR"
    if ! $DOCKER_COMPOSE up -d; then
        print_error "Failed to start PostgreSQL"
        return 1
    fi
    
    # Wait for PostgreSQL
    print_info "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if PGPASSWORD=test psql -h localhost -U test -d aw_test -c "SELECT 1" 2>/dev/null; then
            print_success "PostgreSQL is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "PostgreSQL failed to start"
            return 1
        fi
        echo -n "."
        sleep 1
    done
    echo ""
    
    print_header "Running PostgreSQL Tests"
    if DATABASE_URL="$DB_URL" cargo test --lib --tests --features postgres $VERBOSE; then
        print_success "PostgreSQL tests passed"
        return 0
    else
        print_error "PostgreSQL tests failed"
        return 1
    fi
}

run_lint() {
    print_header "Running Lint Checks"
    
    print_info "Checking formatting..."
    if ! cargo fmt -- --check; then
        print_error "Format check failed"
        return 1
    fi
    print_success "Format check passed"
    
    print_info "Running Clippy..."
    if ! cargo clippy --all-targets -- -D warnings; then
        print_error "Clippy check failed"
        return 1
    fi
    print_success "Clippy check passed"
    return 0
}

run_coverage() {
    print_header "Generating Coverage Report"
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_info "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi
    
    if cargo tarpaulin \
        --out Html \
        --out Xml \
        --features sqlite,postgres \
        --timeout 300 \
        --exclude-files tests/* \
        --ignore-panics \
        --ignore-timeouts; then
        print_success "Coverage report generated: tarpaulin-report.html"
        return 0
    else
        print_error "Coverage generation failed"
        return 1
    fi
}

run_benchmarks() {
    print_header "Running Benchmarks"
    
    print_info "SQLite benchmarks..."
    cargo bench --features sqlite 2>&1 | grep -E 'time:|Benchmarking' || print_warning "No benchmarks found"
    
    print_info "PostgreSQL benchmarks..."
    DATABASE_URL="$DB_URL" cargo bench --features postgres 2>&1 | grep -E 'time:|Benchmarking' || print_warning "No benchmarks found"
    
    return 0
}

# Main execution
FAILED=false

case "$MODE" in
    all)
        run_sqlite_tests || FAILED=true
        run_postgres_tests || FAILED=true
        run_lint || FAILED=true
        ;;
    sqlite)
        run_sqlite_tests || FAILED=true
        ;;
    postgres)
        run_postgres_tests || FAILED=true
        ;;
esac

# Optional: Coverage
if [ "$COVERAGE" = "true" ]; then
    run_coverage || FAILED=true
fi

# Optional: Benchmarks
if [ "$BENCHMARK" = "true" ]; then
    run_benchmarks || FAILED=true
fi

print_header "Test Summary"
if [ "$FAILED" = true ]; then
    print_error "Some tests failed"
    exit 1
else
    print_success "All tests passed!"
    exit 0
fi
