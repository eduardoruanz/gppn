#!/usr/bin/env bash
set -euo pipefail

echo "=== GPPN Development Environment Setup ==="

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "Rust: $(rustc --version)"

# Check for Go
if ! command -v go &> /dev/null; then
    echo "Installing Go via Homebrew..."
    brew install go
fi
echo "Go: $(go version)"

# Check for protoc
if ! command -v protoc &> /dev/null; then
    echo "Installing protobuf via Homebrew..."
    brew install protobuf
fi
echo "protoc: $(protoc --version)"

# Check for cmake (needed by rocksdb)
if ! command -v cmake &> /dev/null; then
    echo "Installing cmake via Homebrew..."
    brew install cmake
fi
echo "cmake: $(cmake --version | head -1)"

# Install Rust components
rustup component add clippy rustfmt

# Build the workspace
echo ""
echo "=== Building GPPN workspace ==="
cargo build --workspace

# Run tests
echo ""
echo "=== Running tests ==="
cargo test --workspace

echo ""
echo "=== Setup complete! ==="
echo "Run 'make build' to build, 'make test' to test."
