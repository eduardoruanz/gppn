#!/bin/bash
set -e

export PATH="/opt/homebrew/bin:$PATH"

SERVICES_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Building Veritas Services ==="
echo ""

# Build shared package
echo "--- Building pkg ---"
cd "$SERVICES_DIR/pkg"
go build ./...
echo "pkg: BUILD OK"
echo ""

# Build and test issuer-api
echo "--- Building issuer-api ---"
cd "$SERVICES_DIR/issuer-api"
go mod tidy
go build ./...
echo "issuer-api: BUILD OK"
echo ""

# Build and test verifier-api
echo "--- Building verifier-api ---"
cd "$SERVICES_DIR/verifier-api"
go mod tidy
go build ./...
echo "verifier-api: BUILD OK"
echo ""

# Build and test registry-api
echo "--- Building registry-api ---"
cd "$SERVICES_DIR/registry-api"
go mod tidy
go build ./...
echo "registry-api: BUILD OK"
echo ""

# Build gateway
echo "--- Building gateway ---"
cd "$SERVICES_DIR/gateway"
go mod tidy
go build ./...
echo "gateway: BUILD OK"
echo ""

echo "=== All Veritas services built successfully ==="
