.PHONY: build test lint fmt proto-gen docker-build testnet-local clean check

# Default target
all: build test lint

# Build all Rust workspace members
build:
	cargo build --workspace

# Build in release mode
build-release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Check formatting
fmt:
	cargo fmt --all -- --check

# Format code
fmt-fix:
	cargo fmt --all

# Check compilation without building
check:
	cargo check --workspace

# Generate protobuf code (handled by build.rs, this is for manual regeneration)
proto-gen:
	@echo "Protobuf code is generated automatically by build.rs during cargo build"
	cargo build -p gppn-core

# Build Docker images
docker-build:
	docker build -f infra/docker/Dockerfile.node -t gppn-node .
	docker build -f infra/docker/Dockerfile.sa-ethereum -t gppn-sa-ethereum services/sa-ethereum/

# Start local testnet with 3 nodes
testnet-local:
	docker compose -f infra/docker/docker-compose.yml up -d

# Stop local testnet
testnet-stop:
	docker compose -f infra/docker/docker-compose.yml down

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Build and test TypeScript SDK
sdk-ts:
	cd sdks/typescript && npm ci && npm test && npm run build

# Build Go services
go-build:
	@for dir in services/sa-ethereum services/sa-bitcoin services/sa-stablecoin services/explorer-api services/gateway; do \
		echo "Building $$dir..."; \
		cd $$dir && go build ./... && cd -; \
	done

# Run Go tests
go-test:
	@for dir in services/sa-ethereum services/sa-bitcoin services/sa-stablecoin services/explorer-api services/gateway; do \
		echo "Testing $$dir..."; \
		cd $$dir && go test ./... && cd -; \
	done

# Full CI pipeline
ci: fmt lint test
	@echo "CI pipeline passed"
