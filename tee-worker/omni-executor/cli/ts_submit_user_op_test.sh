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
export TEST_CHAIN_ID="1337"
export TEE_WORKER_RPC_URL="http://omni-executor:2100"

# Contract addresses should be available from the shared volume
if [ -f "/shared/deployed-addresses.json" ]; then
    echo "Loading deployed contract addresses..."
    export TEST_ENTRY_POINT_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.EntryPoint // "0x5FbDB2315678afecb367f032d93F642f64180aa3"')
    export TEST_FACTORY_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.OmniAccountFactory // "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"')
    export TEST_USDC_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.TestUSDC // "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"')
    export TEST_USDT_ADDRESS=$(cat /shared/deployed-addresses.json | jq -r '.TestUSDT // "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9"')
else
    echo "Using default contract addresses..."
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