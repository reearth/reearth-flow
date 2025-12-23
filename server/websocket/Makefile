.PHONY: all build build-release run run-auth test test-all lint fmt check clean \
        docker-build docker-run help gen-thrift clean-thrift

# Default target
all: check

# ============================================================================
# Build Commands
# ============================================================================

## Build the project in debug mode
build:
	@echo "Building websocket server (debug)..."
	cargo build

## Build the project in release mode
build-release:
	@echo "Building websocket server (release)..."
	cargo build --release

## Build with auth feature
build-auth:
	@echo "Building websocket server with auth feature..."
	cargo build --features auth

# ============================================================================
# Run Commands
# ============================================================================

## Run the websocket server
run:
	@echo "Starting websocket server..."
	cargo run --bin websocket

## Run the websocket server with auth feature
run-auth:
	@echo "Starting websocket server with auth..."
	cargo run --bin websocket --features auth

## Run in release mode
run-release:
	@echo "Starting websocket server (release)..."
	cargo run --bin websocket --release

## Run with hot reload (requires cargo-watch)
watch:
	@echo "Starting websocket server with hot reload..."
	cargo watch -x 'run --bin websocket'

# ============================================================================
# Test Commands
# ============================================================================

## Run unit tests
test:
	@echo "Running unit tests..."
	cargo test --lib

## Run all tests including integration tests
test-all:
	@echo "Running all tests..."
	cargo test

## Run tests with auth feature
test-auth:
	@echo "Running tests with auth feature..."
	cargo test --features auth

## Run tests with output
test-verbose:
	@echo "Running tests with verbose output..."
	cargo test -- --nocapture

# ============================================================================
# Code Quality Commands
# ============================================================================

## Run clippy linter
lint:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

## Run clippy with auth feature
lint-auth:
	@echo "Running clippy with auth feature..."
	cargo clippy --features auth -- -D warnings

## Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

## Check code formatting
fmt-check:
	@echo "Checking code format..."
	cargo fmt -- --check

## Quick compile check
check:
	@echo "Running cargo check..."
	cargo check

## Check with auth feature
check-auth:
	@echo "Running cargo check with auth feature..."
	cargo check --features auth

## Run all checks (format, lint, test)
ci: fmt-check lint test
	@echo "All CI checks passed!"

# ============================================================================
# Clean Commands
# ============================================================================

## Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

## Clean generated Thrift code
clean-thrift:
	@echo "Cleaning up generated Thrift code..."
	@rm -f src/thrift/document.rs
	@echo "Cleanup completed."

# ============================================================================
# Docker Commands
# ============================================================================

## Build Docker image
docker-build:
	@echo "Building Docker image..."
	docker build -f Dockerfile.websocket -t reearth-flow-websocket .

## Run Docker container
docker-run:
	@echo "Running Docker container..."
	docker run -p 8000:8000 --env-file .env reearth-flow-websocket

## Start services with docker-compose
docker-up:
	@echo "Starting services with docker-compose..."
	docker-compose up -d

## Stop services
docker-down:
	@echo "Stopping services..."
	docker-compose down

# ============================================================================
# Thrift Code Generation
# ============================================================================

## Generate Thrift code
gen-thrift:
	@echo "Generating Rust Thrift code..."
	@mkdir -p src/thrift
	@rm -f src/document.rs
	@thrift -r --gen rs:thrift_crate=thrift -out src thrift/document.thrift
	@if [ -f src/document.rs ]; then \
		mv src/document.rs src/thrift/; \
	fi
	@echo "Thrift code generation completed."

# ============================================================================
# Documentation
# ============================================================================

## Generate documentation
doc:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

## Generate documentation with private items
doc-private:
	@echo "Generating documentation (including private items)..."
	cargo doc --no-deps --document-private-items --open

# ============================================================================
# Benchmarks
# ============================================================================

## Run benchmarks
bench:
	@echo "Running benchmarks..."
	cargo bench

# ============================================================================
# Help
# ============================================================================

## Show this help message
help:
	@echo "Available targets:"
	@echo ""
	@echo "Build:"
	@echo "  build          - Build the project in debug mode"
	@echo "  build-release  - Build the project in release mode"
	@echo "  build-auth     - Build with auth feature"
	@echo ""
	@echo "Run:"
	@echo "  run            - Run the websocket server"
	@echo "  run-auth       - Run with auth feature"
	@echo "  run-release    - Run in release mode"
	@echo "  watch          - Run with hot reload (requires cargo-watch)"
	@echo ""
	@echo "Test:"
	@echo "  test           - Run unit tests"
	@echo "  test-all       - Run all tests"
	@echo "  test-auth      - Run tests with auth feature"
	@echo "  test-verbose   - Run tests with verbose output"
	@echo ""
	@echo "Code Quality:"
	@echo "  lint           - Run clippy linter"
	@echo "  lint-auth      - Run clippy with auth feature"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  check          - Quick compile check"
	@echo "  check-auth     - Check with auth feature"
	@echo "  ci             - Run all CI checks"
	@echo ""
	@echo "Clean:"
	@echo "  clean          - Clean build artifacts"
	@echo "  clean-thrift   - Clean generated Thrift code"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build   - Build Docker image"
	@echo "  docker-run     - Run Docker container"
	@echo "  docker-up      - Start services with docker-compose"
	@echo "  docker-down    - Stop services"
	@echo ""
	@echo "Documentation:"
	@echo "  doc            - Generate documentation"
	@echo "  doc-private    - Generate docs with private items"
	@echo ""
	@echo "Other:"
	@echo "  gen-thrift     - Generate Thrift code"
	@echo "  bench          - Run benchmarks"
	@echo "  help           - Show this help message"
