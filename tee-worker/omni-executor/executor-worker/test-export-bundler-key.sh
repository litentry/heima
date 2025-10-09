#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="/tmp/executor-worker-test-$$"
AUTHORIZED_KEY_PATH="${TEST_DIR}/authorized_key.bin"
WORKER_DATA_DIR="${TEST_DIR}/worker-data"
WORKER_PORT=3456
WORKER_PID=""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"

    if [ -n "$WORKER_PID" ] && kill -0 "$WORKER_PID" 2>/dev/null; then
        echo "Stopping worker (PID: $WORKER_PID)..."
        kill "$WORKER_PID" 2>/dev/null || true
        wait "$WORKER_PID" 2>/dev/null || true
    fi

    if [ -d "$TEST_DIR" ]; then
        echo "Removing test directory: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi

    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set up trap to cleanup on exit
trap cleanup EXIT INT TERM

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Testing export-bundler-key Command                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}\n"

# Step 1: Create test directory structure
echo -e "${YELLOW}📁 Step 1: Creating test directories...${NC}"
mkdir -p "$TEST_DIR"
mkdir -p "$WORKER_DATA_DIR"
echo -e "${GREEN}✅ Test directories created${NC}\n"

# Step 2: Generate authorized key
echo -e "${YELLOW}🔑 Step 2: Generating authorized key...${NC}"
dd if=/dev/urandom of="$AUTHORIZED_KEY_PATH" bs=32 count=1 2>/dev/null
echo -e "${GREEN}✅ Authorized key generated: $AUTHORIZED_KEY_PATH${NC}\n"

# Step 3: Get public key from authorized key
echo -e "${YELLOW}🔐 Step 3: Extracting public key...${NC}"
PUBKEY_OUTPUT=$(cargo run --example print_ecdsa_pubkey "$AUTHORIZED_KEY_PATH" 2>&1 | tail -1)
AUTHORIZED_PUBKEY="${PUBKEY_OUTPUT##*=}"

if [ -z "$AUTHORIZED_PUBKEY" ] || [ ${#AUTHORIZED_PUBKEY} -ne 66 ]; then
    echo -e "${RED}❌ Failed to extract public key${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Public key extracted: $AUTHORIZED_PUBKEY${NC}\n"

# Step 4: Build worker
echo -e "${YELLOW}🔨 Step 4: Building worker...${NC}"
cargo build --bin executor-worker 2>&1 | grep -E "(Compiling|Finished|error)" || true
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Worker built successfully${NC}\n"

# Step 5: Start worker
echo -e "${YELLOW}🚀 Step 5: Starting worker...${NC}"

# Set up minimal environment variables
export OE_BUNDLER_KEY_EXPORT_AUTHORIZED_PUBKEY="$AUTHORIZED_PUBKEY"
export OE_PUMPX_WORKER_URL="ws://127.0.0.1:$WORKER_PORT"
export RUST_LOG=info

# Start worker in background
cargo run --bin executor-worker -- run \
    --local-directory-path "$WORKER_DATA_DIR" \
    > "${TEST_DIR}/worker.log" 2>&1 &

WORKER_PID=$!

echo "Worker started (PID: $WORKER_PID)"
echo -e "Waiting for worker to be ready..."

# Wait for worker to be ready (check if WebSocket port is listening)
MAX_WAIT=30
ELAPSED=0
while [ $ELAPSED -lt $MAX_WAIT ]; do
    if lsof -i ":$WORKER_PORT" -t >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Worker is ready!${NC}\n"
        break
    fi

    # Check if worker process is still running
    if ! kill -0 "$WORKER_PID" 2>/dev/null; then
        echo -e "${RED}❌ Worker process died. Check logs:${NC}"
        tail -20 "${TEST_DIR}/worker.log"
        exit 1
    fi

    sleep 1
    ELAPSED=$((ELAPSED + 1))
    echo -n "."
done

if [ $ELAPSED -ge $MAX_WAIT ]; then
    echo -e "\n${RED}❌ Worker failed to start within ${MAX_WAIT}s${NC}"
    echo "Worker logs:"
    tail -50 "${TEST_DIR}/worker.log"
    exit 1
fi

# Step 6: Run export-bundler-key command
echo -e "${YELLOW}🔓 Step 6: Running export-bundler-key command...${NC}"
echo -e "${BLUE}Command:${NC} cargo run --bin executor-worker -- export-bundler-key \\"
echo -e "  --authorized-key-path $AUTHORIZED_KEY_PATH \\"
echo -e "  --worker-url ws://127.0.0.1:$WORKER_PORT\n"

cargo run --bin executor-worker -- export-bundler-key \
    --authorized-key-path "$AUTHORIZED_KEY_PATH" \
    --worker-url "ws://127.0.0.1:$WORKER_PORT"

EXPORT_RESULT=$?

echo ""

if [ $EXPORT_RESULT -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ Test Completed Successfully!                          ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
else
    echo -e "${RED}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ Test Failed                                            ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════╝${NC}"
    echo -e "\n${YELLOW}Worker logs (last 50 lines):${NC}"
    tail -50 "${TEST_DIR}/worker.log"
fi

exit $EXPORT_RESULT
