#!/bin/bash
set -e

export PATH="/opt/homebrew/bin:$PATH"

SERVICES_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Building GPPN Services ==="
echo ""

# Build shared package
echo "--- Building pkg ---"
cd "$SERVICES_DIR/pkg"
go build ./...
echo "pkg: BUILD OK"
echo ""

# Build and test sa-ethereum
echo "--- Building sa-ethereum ---"
cd "$SERVICES_DIR/sa-ethereum"
go mod tidy
go build ./...
echo "sa-ethereum: BUILD OK"
go test ./...
echo "sa-ethereum: TESTS OK"
echo ""

# Build and test sa-bitcoin
echo "--- Building sa-bitcoin ---"
cd "$SERVICES_DIR/sa-bitcoin"
go mod tidy
go build ./...
echo "sa-bitcoin: BUILD OK"
go test ./...
echo "sa-bitcoin: TESTS OK"
echo ""

# Build and test sa-stablecoin
echo "--- Building sa-stablecoin ---"
cd "$SERVICES_DIR/sa-stablecoin"
go mod tidy
go build ./...
echo "sa-stablecoin: BUILD OK"
go test ./...
echo "sa-stablecoin: TESTS OK"
echo ""

# Build explorer-api
echo "--- Building explorer-api ---"
cd "$SERVICES_DIR/explorer-api"
go mod tidy
go build ./...
echo "explorer-api: BUILD OK"
echo ""

# Build gateway
echo "--- Building gateway ---"
cd "$SERVICES_DIR/gateway"
go mod tidy
go build ./...
echo "gateway: BUILD OK"
echo ""

echo "=== All services built successfully ==="
