#!/bin/bash

# Script to update demo app addresses after running deploy-local.sh
# This extracts the deployed contract addresses and creates/updates the .env.local file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_APP_DIR="$SCRIPT_DIR/aa-demo-app"
ENV_FILE="$DEMO_APP_DIR/.env.local"
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-local}"
DEPLOYMENT_FILE="$SCRIPT_DIR/deployments/$DEPLOYMENT_ENV/$DEPLOYMENT_ENV.json"

echo "🔍 Extracting deployed contract addresses..."

# Function to extract addresses from deployment file (primary method)
extract_from_deployment() {
    if [ -f "$DEPLOYMENT_FILE" ] && command -v jq &> /dev/null; then
        echo "📋 Using deployment artifacts from: $DEPLOYMENT_FILE"
        
        # Extract addresses using jq
        ENTRYPOINT_ADDRESS=$(jq -r '.contracts.EntryPointV1.address // empty' "$DEPLOYMENT_FILE")
        FACTORY_ADDRESS=$(jq -r '.contracts.OmniAccountFactoryV1.address // empty' "$DEPLOYMENT_FILE")
        PAYMASTER_ADDRESS=$(jq -r '.contracts.SimplePaymaster.address // .contracts.DemoPaymaster.address // empty' "$DEPLOYMENT_FILE")
        
        # Extract test token addresses based on metadata
        USDC_ADDRESS=$(jq -r '.contracts | to_entries[] | select(.value.metadata.symbol == "USDC") | .value.address // empty' "$DEPLOYMENT_FILE" | head -1)
        USDT_ADDRESS=$(jq -r '.contracts | to_entries[] | select(.value.metadata.symbol == "USDT") | .value.address // empty' "$DEPLOYMENT_FILE" | head -1)
        
        # If only one TestToken exists, use it for both
        if [ -z "$USDC_ADDRESS" ] || [ -z "$USDT_ADDRESS" ]; then
            TEST_TOKEN=$(jq -r '.contracts.TestToken.address // empty' "$DEPLOYMENT_FILE")
            USDC_ADDRESS=${USDC_ADDRESS:-$TEST_TOKEN}
            USDT_ADDRESS=${USDT_ADDRESS:-$TEST_TOKEN}
        fi
        
        if [ -n "$ENTRYPOINT_ADDRESS" ] && [ -n "$FACTORY_ADDRESS" ]; then
            echo "✅ Successfully extracted addresses from deployment artifacts"
            return 0
        fi
    fi
    return 1
}

# Function to extract address from forge output or broadcast file
extract_addresses() {
    # Check for new broadcast file first (DeployLocalWithPaymaster)
    local broadcast_file="$SCRIPT_DIR/broadcast/DeployLocalWithPaymaster.s.sol/1337/run-latest.json"
    
    # If new broadcast file doesn't exist, check for DeployLocal.s.sol
    if [ ! -f "$broadcast_file" ]; then
        broadcast_file="$SCRIPT_DIR/broadcast/DeployLocal.s.sol/1337/run-latest.json"
        if [ -f "$broadcast_file" ]; then
            echo "ℹ️  Using DeployLocal.s.sol broadcast file"
        fi
    else
        echo "ℹ️  Using DeployLocalWithPaymaster.s.sol broadcast file"
    fi
    
    # Check for old broadcast file format as last resort
    if [ ! -f "$broadcast_file" ]; then
        local old_broadcast_file="$SCRIPT_DIR/broadcast/DeployAA.s.sol/1337/run-latest.json"
        if [ -f "$old_broadcast_file" ]; then
            broadcast_file="$old_broadcast_file"
            echo "⚠️  Using old broadcast file format. Consider re-running local-deploy.sh"
        fi
    fi
    
    if [ ! -f "$broadcast_file" ]; then
        echo "❌ Broadcast file not found. Please run local-deploy.sh first."
        echo "   Expected locations:"
        echo "   - $SCRIPT_DIR/broadcast/DeployLocalWithPaymaster.s.sol/1337/run-latest.json"
        echo "   - $SCRIPT_DIR/broadcast/DeployLocal.s.sol/1337/run-latest.json"
        exit 1
    fi
    
    # Extract addresses using grep and sed
    ENTRYPOINT_ADDRESS=$(grep -A2 '"contractName": "EntryPointV1"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    # Try both SmartAccountFactory (old) and OmniAccountFactory (new) names
    FACTORY_ADDRESS=$(grep -A2 '"contractName": "OmniAccountFactoryV1"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    if [ -z "$FACTORY_ADDRESS" ]; then
        FACTORY_ADDRESS=$(grep -A2 '"contractName": "SmartAccountFactory"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    fi
    
    # Extract test token addresses
    USDC_ADDRESS=$(grep -A2 '"contractName": "TestToken"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    USDT_ADDRESS=$(grep -A2 '"contractName": "TestToken"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | tail -1)
    
    # Extract DemoPaymaster address (try both DemoPaymaster and SimplePaymaster for backwards compatibility)
    PAYMASTER_ADDRESS=$(grep -A2 '"contractName": "DemoPaymaster"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    if [ -z "$PAYMASTER_ADDRESS" ]; then
        PAYMASTER_ADDRESS=$(grep -A2 '"contractName": "SimplePaymaster"' "$broadcast_file" | grep '"contractAddress"' | sed 's/.*"contractAddress": "\(.*\)".*/\1/' | head -1)
    fi
    
    # OmniAccount implementation is created by the factory, need to find it differently
    # For now, we'll leave it empty as it's deployed by the factory
    OMNI_ACCOUNT_IMPL_ADDRESS=""
    
    echo "Found addresses:"
    echo "  EntryPoint: $ENTRYPOINT_ADDRESS"
    echo "  OmniAccountFactory: $FACTORY_ADDRESS"
    echo "  Paymaster: $PAYMASTER_ADDRESS"
    echo "  Test USDC: $USDC_ADDRESS"
    echo "  Test USDT: $USDT_ADDRESS"
}

# Function to create or update .env.local
update_env_file() {
    echo "📝 Updating $ENV_FILE..."
    
    # Create backup if file exists
    if [ -f "$ENV_FILE" ]; then
        cp "$ENV_FILE" "$ENV_FILE.backup"
        echo "  Created backup: $ENV_FILE.backup"
    fi
    
    # Create new env file
    cat > "$ENV_FILE" << EOF
# Auto-generated by update-demo-addresses.sh
# Last updated: $(date)

# Chain Configuration (Anvil local)
NEXT_PUBLIC_CHAIN_ID=1337

# Contract Addresses
NEXT_PUBLIC_ENTRYPOINT_ADDRESS=$ENTRYPOINT_ADDRESS
NEXT_PUBLIC_FACTORY_ADDRESS=$FACTORY_ADDRESS
NEXT_PUBLIC_OMNI_ACCOUNT_IMPL_ADDRESS=$OMNI_ACCOUNT_IMPL_ADDRESS
NEXT_PUBLIC_PAYMASTER_ADDRESS=$PAYMASTER_ADDRESS

# Token Addresses
NEXT_PUBLIC_USDC_ADDRESS=$USDC_ADDRESS
NEXT_PUBLIC_USDT_ADDRESS=$USDT_ADDRESS

# Local RPC URL
NEXT_PUBLIC_RPC_URL=http://localhost:8545

# TEE Worker RPC URL
NEXT_PUBLIC_TEE_WORKER_RPC_URL=https://staging-dex-worker.heima.network

# Add your WalletConnect Project ID here
# NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=your_project_id_here
EOF

    echo "✅ Updated $ENV_FILE with deployed addresses"
}

# Function to show next steps
show_next_steps() {
    echo ""
    echo "🎉 Demo app configuration updated!"
    echo ""
    echo "Next steps:"
    echo "1. cd aa-demo-app"
    echo "2. pnpm install (if not already done)"
    echo "3. pnpm dev"
    echo "4. Open http://localhost:3000"
    echo ""
    echo "Contract addresses:"
    echo "  EntryPoint: $ENTRYPOINT_ADDRESS"
    echo "  Factory: $FACTORY_ADDRESS"
    echo "  Paymaster: $PAYMASTER_ADDRESS"
    echo "  Test USDC: $USDC_ADDRESS"
    echo "  Test USDT: $USDT_ADDRESS"
    echo ""
    echo "Make sure Anvil is running on port 8545 (started by deploy-local.sh)"
}

# Main execution
# Try deployment artifacts first, fall back to broadcast files
if ! extract_from_deployment; then
    echo "📂 Deployment artifacts not found, falling back to broadcast files..."
    extract_addresses
fi

# Run ABI sync if the script exists
if [ -f "$SCRIPT_DIR/sync-abis-from-deployment.sh" ]; then
    echo ""
    echo "🔄 Syncing ABIs from deployment artifacts..."
    "$SCRIPT_DIR/sync-abis-from-deployment.sh"
fi

update_env_file
show_next_steps