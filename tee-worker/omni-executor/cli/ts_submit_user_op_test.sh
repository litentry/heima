#!/bin/bash

# Copyright 2020-2024 Trust Computing GmbH.

# Enable strict error handling: exit on error (-e), error on undefined vars (-u), exit on pipe failures (-o pipefail)
set -euo pipefail

# Parse command line options:
# -u: Node URL to connect to
while getopts ":u" opt; do
    case $opt in
        u)
            NODE_URL=$OPTARG
            ;;
        ?)
            echo "Invalid option: -${OPTARG}."
            exit 1
            ;;
    esac
done

NODE_URL=${NODE_URL:-"http://heima-node:9944"}
echo "Using node url $NODE_URL"

echo "Running SubmitUserOp integration tests"

echo "Installing dependencies and building ts-tests"
cd /ts-tests
pnpm install --force

# Set environment variables for the SubmitUserOp test
export TEST_RPC_URL="http://ethereum-node:8545"
export TEST_CHAIN_ID="31337"
export TEE_WORKER_RPC_URL="http://omni-executor:2100"

# Wait for contract deployment to complete
echo "Waiting for contract deployment..."
DEPLOYED_ADDRESSES_FILE="/shared/deployed-addresses.json"
MAX_WAIT=120  # 2 minutes
WAIT_COUNT=0

# Debug: Check shared directory and its contents
echo "DEBUG: Checking /shared directory..."
ls -la /shared/ || echo "DEBUG: /shared directory does not exist"

# Debug: Check if aa-contracts-deploy container is running/completed
echo "DEBUG: Checking docker containers status..."
docker ps -a | grep aa-contracts || echo "DEBUG: No aa-contracts container found"

while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    if [ -f "$DEPLOYED_ADDRESSES_FILE" ]; then
        echo "Contract deployment file found!"
        break
    fi
    
    # Print more debug info every 20 seconds
    if [ $((WAIT_COUNT % 20)) -eq 0 ] && [ $WAIT_COUNT -gt 0 ]; then
        echo "DEBUG: Still waiting... checking /shared directory contents:"
        ls -la /shared/ || echo "DEBUG: /shared directory not accessible"
        echo "DEBUG: Checking for any JSON files in /shared:"
        find /shared -name "*.json" 2>/dev/null || echo "DEBUG: No JSON files found"
        echo "DEBUG: Docker container status:"
        docker ps -a | grep -E "(aa-contracts|ethereum-node)" || echo "DEBUG: Container check failed"
    fi
    
    sleep 1
    WAIT_COUNT=$((WAIT_COUNT + 1))
    if [ $((WAIT_COUNT % 10)) -eq 0 ]; then
        echo "Still waiting for contracts... (${WAIT_COUNT}s)"
    fi
done

# Contract addresses should be available from the shared volume
if [ -f "/shared/deployed-addresses.json" ]; then
    echo "Loading deployed contract addresses..."
    export TEST_ENTRY_POINT_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.EntryPoint // "0x5FbDB2315678afecb367f032d93F642f64180aa3"')
    export TEST_FACTORY_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.OmniAccountFactory // "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"')
    export TEST_USDC_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.TestUSDC // "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"')
    export TEST_USDT_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.TestUSDT // "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9"')
    echo "Successfully loaded contract addresses from deployment file"
else
    echo "⚠️ Contract deployment file not found, using default addresses..."
    
    # Check aa-contracts-deploy logs to see what went wrong
    echo "DEBUG: Checking aa-contracts-deploy logs..."
    docker logs aa-contracts-deploy 2>&1 | tail -50 || echo "DEBUG: Could not get aa-contracts-deploy logs"
    
    export TEST_ENTRY_POINT_ADDRESS="0x5FbDB2315678afecb367f032d93F642f64180aa3"
    export TEST_FACTORY_ADDRESS="0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
    export TEST_USDC_ADDRESS="0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"
    export TEST_USDT_ADDRESS="0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9"
fi

echo "Environment variables for SubmitUserOp test:"
echo "  RPC URL: $TEST_RPC_URL"
echo "  Chain ID: $TEST_CHAIN_ID" 
echo "  TEE Worker: $TEE_WORKER_RPC_URL"
echo "  EntryPoint: $TEST_ENTRY_POINT_ADDRESS"
echo "  Factory: $TEST_FACTORY_ADDRESS"

echo "Running SubmitUserOp tests"
OMNI_WORKER_ENDPOINT=ws://omni-executor:2100 PARACHAIN_ENDPOINT=ws://heima-node:9944 pnpm --filter jsonrpc-mock-tests test submitUserOp.test.ts

echo "SubmitUserOp tests completed successfully"