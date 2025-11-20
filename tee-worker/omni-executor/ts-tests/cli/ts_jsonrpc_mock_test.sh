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

echo "Running JSON-RPC tests"

echo "Installing dependencies and building ts-tests"
cd /ts-tests
pnpm install --force

echo "Running JSON-RPC tests"
OMNI_WORKER_ENDPOINT=http://omni-executor:2100 pnpm --filter jsonrpc-mock-tests test jsonrpc_mock_test.test.ts hyperliquid_signature_data.test.ts

echo "JSON-RPC tests completed successfully"