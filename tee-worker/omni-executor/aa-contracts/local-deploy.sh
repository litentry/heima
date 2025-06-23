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
    echo "📦 Deploying EntryPoint contract..."
    ENTRYPOINT_OUTPUT=$(forge create --from $DEPLOYER_ADDRESS --unlocked --broadcast EntryPoint)
    ENTRYPOINT_ADDRESS=$(echo "$ENTRYPOINT_OUTPUT" | grep "Deployed to:" | awk '{print $3}')
    
    if [ -z "$ENTRYPOINT_ADDRESS" ]; then
        echo "❌ Failed to deploy EntryPoint contract"
        exit 1
    fi
    
    echo "✅ EntryPoint deployed at: $ENTRYPOINT_ADDRESS"
    
    echo "📦 Deploying SmartAccountFactory contract..."
    FACTORY_OUTPUT=$(forge create --from $DEPLOYER_ADDRESS --unlocked --broadcast SmartAccountFactory --constructor-args $ENTRYPOINT_ADDRESS)
    FACTORY_ADDRESS=$(echo "$FACTORY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')
    
    if [ -z "$FACTORY_ADDRESS" ]; then
        echo "❌ Failed to deploy SmartAccountFactory contract"
        exit 1
    fi
    
    echo "✅ SmartAccountFactory deployed at: $FACTORY_ADDRESS"
    
    echo "📦 Deploying SimplePaymaster contract..."
    PAYMASTER_OUTPUT=$(forge create --from $DEPLOYER_ADDRESS --unlocked --broadcast SimplePaymaster --constructor-args $ENTRYPOINT_ADDRESS $OMNI_EXECUTOR_SIGNER)
    PAYMASTER_ADDRESS=$(echo "$PAYMASTER_OUTPUT" | grep "Deployed to:" | awk '{print $3}')
    
    if [ -z "$PAYMASTER_ADDRESS" ]; then
        echo "❌ Failed to deploy SimplePaymaster contract"
        exit 1
    fi
    
    echo "✅ SimplePaymaster deployed at: $PAYMASTER_ADDRESS"
    
    echo ""
    echo "🎉 All contracts deployed successfully!"
    echo ""
    echo "Contract Addresses:"
    echo "==================="
    echo "EntryPoint:         $ENTRYPOINT_ADDRESS"
    echo "SmartAccountFactory: $FACTORY_ADDRESS"
    echo "SimplePaymaster:    $PAYMASTER_ADDRESS"
    echo ""
    echo "Anvil RPC URL: http://$ANVIL_HOST:$ANVIL_PORT"
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
    anvil --host $ANVIL_HOST --port $ANVIL_PORT &
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