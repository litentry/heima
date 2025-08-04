#!/bin/bash

# AA Contracts Deployment Script
# This script starts an Anvil node and deploys the Account Abstraction contracts
#
# Usage:
#   ./local-deploy.sh                   # Deploy with SimplePaymaster (default)
#   PAYMASTER_TYPE=demo ./local-deploy.sh  # Deploy with DemoPaymaster (no bundler restrictions)

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

# Paymaster configuration
# Set PAYMASTER_TYPE=demo to deploy DemoPaymaster (no bundler restrictions)
# Default is SimplePaymaster (requires authorized bundlers)
PAYMASTER_TYPE="${PAYMASTER_TYPE:-simple}"

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
    echo "📦 Deploying contracts using Foundry script..."
    
    # Export environment variables for the script
    export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"  # Anvil's first private key
    export OMNI_EXECUTOR_SIGNER=$OMNI_EXECUTOR_SIGNER
    export PAYMASTER_TYPE=$PAYMASTER_TYPE
    export SAVE_DEPLOYMENT_FILE=true  # Enable saving enhanced deployment artifacts
    export DEPLOYMENT_ENV=local  # Use local environment subdirectory
    
    # Show which paymaster is being deployed
    if [ "$PAYMASTER_TYPE" = "demo" ]; then
        echo "🎮 Deploying with DemoPaymaster (no bundler restrictions)"
    else
        echo "🔒 Deploying with SimplePaymaster (requires authorized bundlers)"
    fi
    
    # Run the deployment script
    DEPLOY_OUTPUT=$(forge script script/DeployLocalWithPaymaster.s.sol:DeployLocalWithPaymaster --rpc-url http://$ANVIL_HOST:$ANVIL_PORT --broadcast --legacy)
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to deploy contracts"
        echo "$DEPLOY_OUTPUT"
        exit 1
    fi
    
    echo "$DEPLOY_OUTPUT"
    
    # Extract addresses from the broadcast file
    BROADCAST_FILE="$SCRIPT_DIR/broadcast/DeployLocalWithPaymaster.s.sol/$CHAIN_ID/run-latest.json"
    
    if [ -f "$BROADCAST_FILE" ]; then
        ENTRYPOINT_ADDRESS=$(grep -A2 '"contractName": "EntryPointV1"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        FACTORY_ADDRESS=$(grep -A2 '"contractName": "OmniAccountFactoryV1"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        # Try to find DemoPaymaster first, fall back to SimplePaymaster
        PAYMASTER_ADDRESS=$(grep -A2 '"contractName": "DemoPaymaster"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        if [ -z "$PAYMASTER_ADDRESS" ]; then
            PAYMASTER_ADDRESS=$(grep -A2 '"contractName": "SimplePaymaster"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        fi
        
        # Extract test token addresses
        USDC_ADDRESS=$(grep -A2 '"contractName": "TestToken"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
        USDT_ADDRESS=$(grep -A2 '"contractName": "TestToken"' "$BROADCAST_FILE" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | tail -1)
        
        echo ""
        echo "🎉 All contracts deployed successfully!"
        echo ""
        echo "Contract Addresses:"
        echo "==================="
        echo "EntryPoint:         $ENTRYPOINT_ADDRESS"
        echo "OmniAccountFactory: $FACTORY_ADDRESS"
        echo "Paymaster:          $PAYMASTER_ADDRESS"
        echo ""
        echo "Test Token Addresses:"
        echo "===================="
        echo "Test USDC:          $USDC_ADDRESS"
        echo "Test USDT:          $USDT_ADDRESS"
        echo ""
        echo "Anvil RPC URL: http://$ANVIL_HOST:$ANVIL_PORT"
        echo ""
        echo "📋 To use these addresses in the demo app:"
        echo "   Run: ./update-demo-addresses.sh"
        echo ""
        echo "Or manually update your .env.local file:"
        echo "NEXT_PUBLIC_ENTRYPOINT_ADDRESS=$ENTRYPOINT_ADDRESS"
        echo "NEXT_PUBLIC_FACTORY_ADDRESS=$FACTORY_ADDRESS"
        echo "NEXT_PUBLIC_PAYMASTER_ADDRESS=$PAYMASTER_ADDRESS"
        echo "NEXT_PUBLIC_TEST_USDC_ADDRESS=$USDC_ADDRESS"
        echo "NEXT_PUBLIC_TEST_USDT_ADDRESS=$USDT_ADDRESS"
        
        # Auto-sync ABIs if deployment artifacts were saved
        if [ -f "$SCRIPT_DIR/deployments/$DEPLOYMENT_ENV/$DEPLOYMENT_ENV.json" ] && [ -f "$SCRIPT_DIR/sync-abis-from-deployment.sh" ]; then
            echo ""
            echo "🔄 Auto-syncing ABIs to demo app..."
            "$SCRIPT_DIR/sync-abis-from-deployment.sh"
        fi
    else
        echo "⚠️  Contracts deployed but broadcast file not found"
        echo "Check the output above for contract addresses"
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
    anvil -vv --host $ANVIL_HOST --chain-id $CHAIN_ID --port $ANVIL_PORT &
    ANVIL_PID=$!
    ANVIL_RUNNING=false
    wait_for_anvil
fi

# Compile contracts
echo "🔨 Compiling contracts..."
forge compile

# Deploy contracts
deploy_contracts

# Enrich deployment artifacts with ABIs if Node.js is available
if command -v node &> /dev/null && [ -f "$SCRIPT_DIR/extract-abis.js" ]; then
    echo ""
    echo "📝 Enriching deployment artifacts with ABIs..."
    node "$SCRIPT_DIR/extract-abis.js" || echo "⚠️  Failed to enrich ABIs, but deployment was successful"
fi

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
