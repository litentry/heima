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

function usage() {
    echo ""
    echo "This is a script for omni-executor client-sdk ts tests."
    echo ""
}

echo "Running client-sdk tests"

cd /client-api/parachain-api
curl -s -H "Content-Type: application/json" -d '{"id": "1", "jsonrpc": "2.0", "method": "state_getMetadata", "params": []}' $NODE_URL > prepare-build/litentry-parachain-metadata.json
echo "Parachain metadata fetched"

echo "Installing dependencies and building parachain-api"
cd /client-api
pnpm install --force
cd /client-api/parachain-api
pnpm build

echo "Installing dependencies and running client-sdk tests"
apt-get update && apt-get install -y jq || true
cd /client-sdk/packages/client-sdk
jq '.peerDependencies["@heima-network/parachain-api"] = "file:/client-api/parachain-api"' package.json > temp.json && mv temp.json package.json
cd /client-sdk
pnpm install --force
pnpm nx run chaindata:build
HEIMA_NETWORK=ws://omni-executor:2100 PARACHAIN_ENDPOINT=ws://heima-node:9944 pnpm nx run client-sdk:test
