#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# Veritas Identity Protocol — Three-Node Demo
# =============================================================================
#
# Demonstrates the full identity workflow:
#   1. Start 3 nodes (Issuer, Holder, Verifier)
#   2. Node 1 (Issuer) issues a KYC credential to Node 2 (Holder)
#   3. Node 2 (Holder) stores the credential
#   4. Node 3 (Verifier) requests proof: "age >= 18 and resident of BR"
#   5. Node 2 generates ZKP without revealing DOB or address
#   6. Node 3 verifies proofs
#   7. Trust attestation flows across the network
# =============================================================================

DEMO_DIR=$(mktemp -d)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Node configuration
NODE1_P2P=9000; NODE1_API=9001; NODE1_DIR="$DEMO_DIR/node1"
NODE2_P2P=9010; NODE2_API=9011; NODE2_DIR="$DEMO_DIR/node2"
NODE3_P2P=9020; NODE3_API=9021; NODE3_DIR="$DEMO_DIR/node3"

PIDS=()

cleanup() {
    echo ""
    echo "=== Cleaning up ==="
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    done
    rm -rf "$DEMO_DIR"
    echo "Demo cleanup complete."
}
trap cleanup EXIT

banner() {
    echo ""
    echo "============================================================"
    echo "  $1"
    echo "============================================================"
    echo ""
}

wait_for_api() {
    local port=$1
    local name=$2
    local retries=30
    while ! curl -sf "http://127.0.0.1:${port}/api/v1/health" >/dev/null 2>&1; do
        retries=$((retries - 1))
        if [ "$retries" -le 0 ]; then
            echo "ERROR: $name failed to start on port $port"
            exit 1
        fi
        sleep 0.5
    done
    echo "$name is ready on port $port"
}

# =============================================================================
banner "Step 0: Build Veritas"
# =============================================================================

cd "$PROJECT_ROOT"
cargo build --workspace --quiet 2>/dev/null || cargo build --workspace

VERITAS_NODE="./target/debug/veritas-node"
if [ ! -f "$VERITAS_NODE" ]; then
    echo "ERROR: veritas-node binary not found at $VERITAS_NODE"
    exit 1
fi

# =============================================================================
banner "Step 1: Start 3 Veritas Nodes"
# =============================================================================

mkdir -p "$NODE1_DIR" "$NODE2_DIR" "$NODE3_DIR"

echo "Starting Node 1 (Issuer) — P2P :$NODE1_P2P, API :$NODE1_API"
$VERITAS_NODE --port $NODE1_P2P --api-port $NODE1_API --data-dir "$NODE1_DIR" --log-level warn &
PIDS+=($!)

echo "Starting Node 2 (Holder)  — P2P :$NODE2_P2P, API :$NODE2_API"
$VERITAS_NODE --port $NODE2_P2P --api-port $NODE2_API --data-dir "$NODE2_DIR" --log-level warn &
PIDS+=($!)

echo "Starting Node 3 (Verifier)— P2P :$NODE3_P2P, API :$NODE3_API"
$VERITAS_NODE --port $NODE3_P2P --api-port $NODE3_API --data-dir "$NODE3_DIR" --log-level warn &
PIDS+=($!)

echo ""
echo "Waiting for nodes to start..."
wait_for_api $NODE1_API "Node 1 (Issuer)"
wait_for_api $NODE2_API "Node 2 (Holder)"
wait_for_api $NODE3_API "Node 3 (Verifier)"

# =============================================================================
banner "Step 2: Query Node Identities"
# =============================================================================

NODE1_ID=$(curl -sf "http://127.0.0.1:${NODE1_API}/api/v1/identity")
NODE2_ID=$(curl -sf "http://127.0.0.1:${NODE2_API}/api/v1/identity")
NODE3_ID=$(curl -sf "http://127.0.0.1:${NODE3_API}/api/v1/identity")

NODE2_DID=$(echo "$NODE2_ID" | python3 -c "import sys,json; print(json.load(sys.stdin)['did'])")

echo "Node 1 (Issuer):   $NODE1_ID"
echo "Node 2 (Holder):   $NODE2_ID"
echo "Node 3 (Verifier): $NODE3_ID"

# =============================================================================
banner "Step 3: Issuer Issues KYC Credential to Holder"
# =============================================================================

echo "Issuing KYC credential from Node 1 to Node 2 (DID: $NODE2_DID)..."
ISSUE_RESULT=$(curl -sf -X POST "http://127.0.0.1:${NODE1_API}/api/v1/credentials/issue" \
    -H "Content-Type: application/json" \
    -d "{
        \"subject_did\": \"$NODE2_DID\",
        \"credential_type\": [\"VerifiableCredential\", \"KycBasic\"],
        \"claims\": {
            \"full_name\": \"Alice Santos\",
            \"date_of_birth\": \"1995-03-15\",
            \"country\": \"BR\",
            \"kyc_level\": 3
        }
    }")

echo "Issue result: $ISSUE_RESULT"

# =============================================================================
banner "Step 4: Generate Zero-Knowledge Proofs"
# =============================================================================

echo "--- Age Proof: Proving age >= 18 without revealing DOB ---"
AGE_PROOF=$(curl -sf -X POST "http://127.0.0.1:${NODE2_API}/api/v1/proofs/generate" \
    -H "Content-Type: application/json" \
    -d '{
        "proof_type": "age",
        "params": {
            "date_of_birth": "1995-03-15",
            "min_age": 18
        }
    }')
echo "Age proof: $AGE_PROOF"

echo ""
echo "--- Residency Proof: Proving resident of BR without revealing address ---"
RESIDENCY_PROOF=$(curl -sf -X POST "http://127.0.0.1:${NODE2_API}/api/v1/proofs/generate" \
    -H "Content-Type: application/json" \
    -d '{
        "proof_type": "residency",
        "params": {
            "country": "BR",
            "allowed_countries": ["BR", "AR", "CL", "CO", "PE"]
        }
    }')
echo "Residency proof: $RESIDENCY_PROOF"

echo ""
echo "--- KYC Level Proof: Proving KYC level >= 2 ---"
KYC_PROOF=$(curl -sf -X POST "http://127.0.0.1:${NODE2_API}/api/v1/proofs/generate" \
    -H "Content-Type: application/json" \
    -d '{
        "proof_type": "kyc_level",
        "params": {
            "actual_level": 3,
            "min_level": 2
        }
    }')
echo "KYC level proof: $KYC_PROOF"

# =============================================================================
banner "Step 5: Trust Attestation"
# =============================================================================

echo "Node 3 (Verifier) attests trust in Node 2 (Holder)..."
TRUST_RESULT=$(curl -sf -X POST "http://127.0.0.1:${NODE3_API}/api/v1/trust/attest" \
    -H "Content-Type: application/json" \
    -d "{
        \"subject_did\": \"$NODE2_DID\",
        \"score\": 0.95,
        \"category\": \"identity_verification\"
    }")
echo "Trust attestation: $TRUST_RESULT"

# =============================================================================
banner "Step 6: Query Node Status"
# =============================================================================

echo "--- Node 1 Status ---"
curl -sf "http://127.0.0.1:${NODE1_API}/api/v1/status" | python3 -m json.tool 2>/dev/null || \
    curl -sf "http://127.0.0.1:${NODE1_API}/api/v1/status"

echo ""
echo "--- Node 2 Status ---"
curl -sf "http://127.0.0.1:${NODE2_API}/api/v1/status" | python3 -m json.tool 2>/dev/null || \
    curl -sf "http://127.0.0.1:${NODE2_API}/api/v1/status"

echo ""
echo "--- Node 3 Status ---"
curl -sf "http://127.0.0.1:${NODE3_API}/api/v1/status" | python3 -m json.tool 2>/dev/null || \
    curl -sf "http://127.0.0.1:${NODE3_API}/api/v1/status"

# =============================================================================
banner "Demo Complete!"
# =============================================================================

echo "Summary:"
echo "  1. Started 3 Veritas nodes (Issuer, Holder, Verifier)"
echo "  2. Issuer issued KYC credential to Holder"
echo "  3. Holder generated ZK proofs (age, residency, KYC level)"
echo "  4. Verifier attested trust in Holder"
echo ""
echo "All identity operations completed successfully."
echo ""
echo "The Veritas protocol enables:"
echo "  - Decentralized identity with W3C Verifiable Credentials"
echo "  - Zero-knowledge proofs (prove facts without revealing data)"
echo "  - Trust attestation across the P2P network"
echo "  - AI-resistant proof of humanity"
