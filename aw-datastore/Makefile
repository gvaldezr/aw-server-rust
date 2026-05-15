.PHONY: help build test test-sqlite test-postgres test-coverage lint fmt clean docker-up docker-down postgres-start postgres-stop

# Variables
CARGO := cargo
DB_URL := postgresql://test:test@localhost:5432/aw_test
DOCKER_COMPOSE := docker-compose -f docker-compose.test.yml

help:
	@echo "=== aw-datastore Testing Framework ==="
	@echo ""
	@echo "Available targets:"
	@echo "  make build              - Build the crate"
	@echo "  make test               - Run all tests (SQLite + PostgreSQL)"
	@echo "  make test-sqlite        - Run SQLite tests only (fast)"
	@echo "  make test-postgres      - Run PostgreSQL tests only"
	@echo "  make test-coverage      - Generate code coverage report"
	@echo "  make lint               - Run format check and clippy"
	@echo "  make fmt                - Auto-format code"
	@echo "  make clean              - Clean build artifacts"
	@echo ""
	@echo "Database targets:"
	@echo "  make postgres-start     - Start PostgreSQL container"
	@echo "  make postgres-stop      - Stop PostgreSQL container"
	@echo "  make postgres-logs      - View PostgreSQL logs"
	@echo ""
	@echo "Development:"
	@echo "  make watch              - Run tests on file changes (requires cargo-watch)"
	@echo ""

build:
	$(CARGO) build

test: test-sqlite test-postgres

test-sqlite:
	@echo "Running SQLite tests..."
	$(CARGO) test --lib --tests --features sqlite
	@echo "✅ SQLite tests passed"

test-postgres: postgres-start
	@echo "Waiting for PostgreSQL..."
	@sleep 2
	@echo "Running PostgreSQL tests..."
	DATABASE_URL=$(DB_URL) $(CARGO) test --lib --tests --features postgres
	@echo "✅ PostgreSQL tests passed"

test-coverage:
	@echo "Generating coverage report..."
	$(CARGO) install cargo-tarpaulin || true
	$(CARGO) tarpaulin \
		--out Html \
		--out Xml \
		--features sqlite,postgres \
		--timeout 300 \
		--exclude-files tests/* \
		--ignore-panics \
		--ignore-timeouts \
		--ignore-unmaintained
	@echo "✅ Coverage report generated: tarpaulin-report.html"

lint:
	@echo "Checking formatting..."
	$(CARGO) fmt -- --check
	@echo "✅ Format check passed"
	@echo ""
	@echo "Running clippy..."
	$(CARGO) clippy --all-targets -- -D warnings
	@echo "✅ Clippy check passed"

fmt:
	@echo "Formatting code..."
	$(CARGO) fmt

clean:
	$(CARGO) clean
	rm -f tarpaulin-report.html cobertura.xml
	$(DOCKER_COMPOSE) down --volumes || true
	@echo "✅ Cleaned"

postgres-start:
	@echo "Starting PostgreSQL container..."
	$(DOCKER_COMPOSE) up -d
	@echo "Waiting for PostgreSQL to be ready..."
	@for i in {1..30}; do \
		if PGPASSWORD=test psql -h localhost -U test -d aw_test -c "SELECT 1" 2>/dev/null; then \
			echo "✅ PostgreSQL is ready"; \
			exit 0; \
		fi; \
		echo "Retry $$i/30..."; \
		sleep 1; \
	done; \
	echo "❌ PostgreSQL failed to start"; \
	exit 1

postgres-stop:
	@echo "Stopping PostgreSQL container..."
	$(DOCKER_COMPOSE) down
	@echo "✅ PostgreSQL stopped"

postgres-logs:
	$(DOCKER_COMPOSE) logs -f postgres

watch:
	@echo "Watching for changes... (Ctrl+C to stop)"
	$(CARGO) watch -x "test --lib --tests --features sqlite"

.DEFAULT_GOAL := help
