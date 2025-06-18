#!/bin/bash
# set -eo we don't allow any command failed in this script.
set -eo pipefail

ROOTDIR=$(git rev-parse --show-toplevel)
new_wasm=/tmp/runtime.wasm

function usage() {
  echo
  echo "Usage: $0 <wasm-name> <endpoint> <release-tag>"
  echo "e.g.:"
  echo "    $0 heima wss://rpc.heima-parachain.heima.network:443 v0.9.25-01"
}

[ $# -ne 3 ] && (usage; exit 1)

function print_divider() {
  echo "------------------------------------------------------------"
}

# Download runtime wasm
print_divider
echo "Download $1-runtime.compact.compressed.wasm from release tag $3 ..."
gh release download "$3" -p "$1-runtime.compact.compressed.wasm" -O "$new_wasm" || true

if [ -f "$new_wasm" ] && [ -s "$new_wasm" ]; then
  ls -l "$new_wasm"
else
  echo "Cannot find $new_wasm or it has 0 bytes, quit"
  exit 1
fi

# Check runtime version
print_divider
echo "Check runtime version ..."
release_version=$(subwasm --json info "$new_wasm" | jq .core_version.specVersion)
onchain_version=$(curl -s -H 'Content-Type: application/json' -d '{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion", "params": [] }' ${2//wss/https} | jq .result.specVersion)

if [ -z "$onchain_version" ]; then
  echo "Failed to fetch on-chain version"
  exit 1
fi

echo "On-chain: $onchain_version"
echo "Release:  $release_version"

if [ "$onchain_version" -ge "$release_version" ]; then
  echo "Current On-chain runtime is up to date, quit"
  exit 1
fi

# Start Chopsticks to fork the chain
print_divider
echo "Forking parachain with Chopsticks ..."
npx @acala-network/chopsticks@latest --config=$ROOTDIR/parachain/scripts/chopsticks/$1.yml &
chopsticks_pid=$!
echo "Chopsticks fork parachain PID: $chopsticks_pid"
sleep 30 # Wait for Chopsticks to initialize

# Check if Chopsticks is running
if ! ps -p $chopsticks_pid > /dev/null; then
  echo "Chopsticks failed to start, quit"
  exit 1
fi

# Create a Node.js script to perform the runtime upgrade
print_divider
echo "Performing runtime upgrade ..."

cd "$ROOTDIR/parachain/ts-tests"
echo "NODE_ENV=ci" > .env
pnpm install && pnpm run test-runtime-upgrade 2>&1

# Cleanup
print_divider
echo "Runtime upgrade succeed!"