#!/bin/bash

# Script to sync ABIs from deployment artifacts to demo app
# This extracts ABIs from the deployment JSON and creates individual ABI files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_APP_DIR="$SCRIPT_DIR/aa-demo-app"
ABI_DIR="$DEMO_APP_DIR/src/contracts/abis"

# Default to local deployment, can be overridden
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-local}"
DEPLOYMENT_FILE="$SCRIPT_DIR/deployments/$DEPLOYMENT_ENV/$DEPLOYMENT_ENV.json"

echo "🔄 Syncing ABIs from deployment artifacts..."
echo "   Using deployment: $DEPLOYMENT_FILE"

# Check if deployment file exists
if [ ! -f "$DEPLOYMENT_FILE" ]; then
    echo "❌ Deployment file not found: $DEPLOYMENT_FILE"
    echo "   Please run ./deploy-local.sh first"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "❌ jq is required but not installed. Please install jq first."
    echo "   On macOS: brew install jq"
    echo "   On Ubuntu: sudo apt-get install jq"
    exit 1
fi

# Create ABI directory if it doesn't exist
mkdir -p "$ABI_DIR"

# Function to extract and save ABI
extract_abi() {
    local contract_name=$1
    local output_name=$2
    local deployment_name=${3:-$contract_name}
    
    echo "  Extracting $deployment_name -> $output_name.json"
    
    # Extract just the ABI array from the deployment file
    jq ".contracts.\"$deployment_name\".abi" "$DEPLOYMENT_FILE" > "$ABI_DIR/$output_name.json" 2>/dev/null || {
        echo "    ⚠️  Contract $deployment_name not found in deployment"
        return 1
    }
    
    # Wrap in full artifact format for compatibility
    local temp_file=$(mktemp)
    echo "{\"abi\":" > "$temp_file"
    cat "$ABI_DIR/$output_name.json" >> "$temp_file"
    echo "}" >> "$temp_file"
    mv "$temp_file" "$ABI_DIR/$output_name.json"
    
    echo "    ✓ Saved to $output_name.json"
}

echo ""
echo "📋 Extracting contract ABIs..."

# Extract core contracts with V1 -> non-V1 mapping
extract_abi "EntryPointV1" "EntryPoint"
extract_abi "EntryPointV1" "IEntryPoint"  # Also save as IEntryPoint for compatibility
extract_abi "OmniAccountFactoryV1" "OmniAccountFactory"
extract_abi "SimplePaymaster" "SimplePaymaster"
extract_abi "ERC20PaymasterV1" "ERC20PaymasterV1"  # Extract ERC20 paymaster if deployed

# Extract TestToken as StandardERC20
extract_abi "TestToken" "StandardERC20"
extract_abi "TestToken" "TestToken"  # Also keep as TestToken

# Special handling for OmniAccount ABI
# Since OmniAccount is created by the factory, we need to extract it from the compiled artifacts
echo ""
echo "📋 Extracting OmniAccount ABI from compiled artifacts..."

# First, try to find the OmniAccountV1 artifact
OMNI_ACCOUNT_ARTIFACT="$SCRIPT_DIR/out/src/accounts/OmniAccountV1.sol/OmniAccountV1.json"
if [ ! -f "$OMNI_ACCOUNT_ARTIFACT" ]; then
    # Try alternative location
    OMNI_ACCOUNT_ARTIFACT="$SCRIPT_DIR/out/OmniAccountV1.sol/OmniAccountV1.json"
fi

if [ -f "$OMNI_ACCOUNT_ARTIFACT" ]; then
    echo "  Found OmniAccountV1 artifact"
    # Extract just the ABI and save it
    jq '{abi: .abi}' "$OMNI_ACCOUNT_ARTIFACT" > "$ABI_DIR/OmniAccount.json"
    echo "    ✓ Saved to OmniAccount.json"
else
    echo "  ⚠️  OmniAccountV1 artifact not found. Running forge build..."
    (cd "$SCRIPT_DIR" && forge build)
    
    # Try again after build
    OMNI_ACCOUNT_ARTIFACT="$SCRIPT_DIR/out/src/accounts/OmniAccountV1.sol/OmniAccountV1.json"
    if [ -f "$OMNI_ACCOUNT_ARTIFACT" ]; then
        jq '{abi: .abi}' "$OMNI_ACCOUNT_ARTIFACT" > "$ABI_DIR/OmniAccount.json"
        echo "    ✓ Saved to OmniAccount.json"
    else
        echo "    ❌ Could not find OmniAccountV1 artifact after build"
    fi
fi

# Count successfully extracted ABIs
ABI_COUNT=$(ls -1 "$ABI_DIR"/*.json 2>/dev/null | wc -l)

echo ""
echo "✅ ABI sync complete!"
echo "   Extracted $ABI_COUNT ABI files to: $ABI_DIR"
echo ""
echo "📝 Next steps:"
echo "   1. Run ./update-demo-addresses.sh to update contract addresses"
echo "   2. cd aa-demo-app && pnpm dev"