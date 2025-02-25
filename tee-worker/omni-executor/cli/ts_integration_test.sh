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

NODE_URL=${NODE_URL:-"http://litentry-node:9944"}
echo "Using node url $NODE_URL"

function usage() {
    echo ""
    echo "This is a script for omni-executor integration ts tests. Pass test name as first argument"
    echo ""
}

# Exit with usage info if exactly one argument is not provided
[ $# -ne 1 ] && (usage; exit 1)

TEST=$1

echo "Running integration tests for $TEST"

cd /client-api/parachain-api
curl -s -H "Content-Type: application/json" -d '{"id": "1", "jsonrpc": "2.0", "method": "state_getMetadata", "params": []}' $NODE_URL > prepare-build/litentry-parachain-metadata.json
echo "Parachain metadata fetched"

# echo "Installing pnpm"
# npm install --global corepack@latest
# corepack enable pnpm

echo "Installing dependencies and building client-api"
cd /client-api
pnpm install --force
pnpm run build-parachain-api

echo "Installing dependencies and building ts-tests"
cd /ts-tests
pnpm install --force

echo "Running integration tests"
OMNI_WORKER_ENDPOINT=ws://omni-executor:2100 PARACHAIN_ENDPOINT=ws://litentry-node:9944 pnpm --filter integration-tests run test $TEST
