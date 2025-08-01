#!/bin/bash

# Remote deployment script with enhanced artifact generation
# 
# Required environment variables (in .env file):
#   RPC_URL - The RPC endpoint for the target network
#   PRIVATE_KEY - The deployer's private key
#   ETHERSCAN_API_KEY - API key for contract verification (optional)
#
# Optional environment variables:
#   DEPLOYMENT_ENV - Environment subdirectory (e.g., "staging", "production")
#   PAYMASTER_INITIAL_DEPOSIT - Initial ETH deposit for paymaster (default: 1 ETH)
#   INITIAL_BUNDLER - Address of the initial bundler (default: deployer)

set -e

# Load environment variables
source .env

# Enable deployment artifact generation
export SAVE_DEPLOYMENT_FILE=true

# Set deployment environment if not already set
# Examples: "staging", "production", or leave empty for flat structure
export DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-}"

echo "🚀 Starting remote deployment..."
if [ -n "$DEPLOYMENT_ENV" ]; then
    echo "📁 Environment: $DEPLOYMENT_ENV"
fi

# Run deployment
if [ -n "$ETHERSCAN_API_KEY" ]; then
    echo "🔍 Running deployment with contract verification..."
    forge script script/Deploy.s.sol:Deploy \
      --rpc-url $RPC_URL \
      --private-key $PRIVATE_KEY \
      --broadcast \
      --verify \
      --etherscan-api-key $ETHERSCAN_API_KEY \
      -vvv
else
    echo "🚀 Running deployment without verification (no ETHERSCAN_API_KEY set)..."
    forge script script/Deploy.s.sol:Deploy \
      --rpc-url $RPC_URL \
      --private-key $PRIVATE_KEY \
      --broadcast \
      -vvv
fi

# Check if deployment succeeded
if [ $? -eq 0 ]; then
    echo "✅ Deployment successful"
    
    # Enrich deployment artifacts with ABIs if Node.js is available
    if command -v node &> /dev/null && [ -f "extract-abis.js" ]; then
        echo "📝 Enriching deployment artifacts with ABIs..."
        node extract-abis.js || echo "⚠️  Failed to enrich ABIs, but deployment was successful"
    else
        echo "⚠️  Node.js not found or extract-abis.js missing. Deployment artifacts will not include ABIs."
        echo "   Run 'node extract-abis.js' manually to add ABIs."
    fi
else
    echo "❌ Deployment failed"
    exit 1
fi

echo "✨ Deployment complete!"
