#!/bin/bash

# AA Contracts Deployment Script
# This script starts an Anvil node and deploys the Account Abstraction contracts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Starting AA Contracts deployment..."

# Default values
ANVIL_PORT=8545
ANVIL_HOST=127.0.0.1
CHAIN_ID=1337
DEPLOYER_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
OMNI_EXECUTOR_SIGNER="0x90F79bf6EB2c4f870365E785982E1f101E93b906"

# Function to check if anvil is running
check_anvil() {
    curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://$ANVIL_HOST:$ANVIL_PORT > /dev/null 2>&1
}

# Function to wait for anvil to be ready
wait_for_anvil() {
    echo "⏳ Waiting for Anvil to be ready..."
    for i in {1..30}; do
        if check_anvil; then
            echo "✅ Anvil is ready!"
            return 0
        fi
        sleep 1
    done
    echo "❌ Anvil failed to start after 30 seconds"
    exit 1
}

# Function to deploy contracts
deploy_contracts() {
    echo "📦 Deploying contracts using Forge script..."
    
    # Set the private key for the deployer (using Anvil's default account 0)
    export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    
    # Run the deployment script
    forge script script/DeployAA.s.sol:DeployAA \
        --rpc-url http://$ANVIL_HOST:$ANVIL_PORT \
        --broadcast \
        --chain-id $CHAIN_ID \
        -vvv

    if [ $? -ne 0 ]; then
        echo "❌ Failed to deploy contracts"
        exit 1
    fi

    # Extract addresses from broadcast file
    BROADCAST_FILE="broadcast/DeployAA.s.sol/$CHAIN_ID/run-latest.json"
    
    if [ -f "$BROADCAST_FILE" ]; then
        ENTRYPOINT_ADDRESS=$(grep -A2 '"contractName": "EntryPoint"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        FACTORY_ADDRESS=$(grep -A2 '"contractName": "SmartAccountFactory"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        
        echo ""
        echo "🎉 All contracts deployed successfully!"
        echo ""
        echo "Contract Addresses:"
        echo "==================="
        echo "EntryPoint:         $ENTRYPOINT_ADDRESS"
        echo "SmartAccountFactory: $FACTORY_ADDRESS"
        echo ""
        echo "Anvil RPC URL: http://$ANVIL_HOST:$ANVIL_PORT"
    else
        echo "❌ Broadcast file not found after deployment"
        exit 1
    fi
}

# Function to cleanup
cleanup() {
    echo "🧹 Cleaning up..."
    if [ ! -z "$ANVIL_PID" ]; then
        kill $ANVIL_PID 2>/dev/null || true
        wait $ANVIL_PID 2>/dev/null || true
        echo "✅ Anvil stopped"
    fi
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Check if Anvil is already running
if check_anvil; then
    echo "✅ Anvil is already running on port $ANVIL_PORT"
    ANVIL_RUNNING=true
else
    echo "🔧 Starting Anvil node..."
    anvil --host $ANVIL_HOST --port $ANVIL_PORT --chain-id $CHAIN_ID &
    ANVIL_PID=$!
    ANVIL_RUNNING=false
    wait_for_anvil
fi

# Compile contracts
echo "🔨 Compiling contracts..."
forge compile

# Deploy contracts
deploy_contracts

# Keep Anvil running if we started it
if [ "$ANVIL_RUNNING" = false ]; then
    echo ""
    echo "🔄 Anvil is running in the background (PID: $ANVIL_PID)"
    echo "   Press Ctrl+C to stop the deployment script and Anvil"
    echo "   Or run 'kill $ANVIL_PID' to stop Anvil manually"
    echo ""

    # Wait for interrupt
    wait $ANVIL_PID
fi
