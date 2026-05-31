#!/bin/bash
# set -eo we don't allow any command failed in this script.
set -eo pipefail

ROOTDIR=$(git rev-parse --show-toplevel)
new_wasm=/tmp/runtime.wasm

function usage() {
  echo
  echo "Usage: $0 <wasm-name> <endpoint> <release-tag>"
  echo "e.g.:"
  echo "    $0 heima wss://rpc.heima-parachain.heima.network v0.9.25-01"
}

[ $# -ne 3 ] && (usage; exit 1)

function print_divider() {
  echo "------------------------------------------------------------"
}

# Download runtime wasm
print_divider
wasm_name="$1-runtime.compact.compressed.wasm"
echo "Download $wasm_name from release tag $3 ..."
gh release download "$3" -p "$wasm_name" || true

# A release may be published without the runtime wasm attached (e.g. an
# assets-less release). In that case there is nothing to test, so skip the
# simulation gracefully instead of failing with a confusing `mv` error.
if [ ! -f "$wasm_name" ] || [ ! -s "$wasm_name" ]; then
  echo "No $wasm_name asset found on release $3 (or it is empty)."
  echo "Skipping runtime upgrade simulation - nothing to test."
  exit 0
fi

mv "$wasm_name" "$new_wasm"
ls -l "$new_wasm"

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
  echo "On-chain runtime version ($onchain_version) >= release version ($release_version)"
  echo "Skipping runtime upgrade test - chain is already up to date"
  exit 0
fi

echo "Upgrade needed: $onchain_version -> $release_version"

# Start Chopsticks to fork the parachain in standalone mode
# Standalone mode is simpler and sufficient for runtime upgrade testing
print_divider
echo "Forking parachain with Chopsticks in standalone mode ..."
npx @acala-network/chopsticks@latest --config=$ROOTDIR/parachain/scripts/chopsticks/$1.yml &
chopsticks_pid=$!
echo "Chopsticks PID: $chopsticks_pid"
echo "Parachain ($1) endpoint: ws://localhost:9944"
sleep 20 # Wait for Chopsticks to initialize

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
echo "PARACHAIN_NAME=$1" >> .env
pnpm install && pnpm run test-runtime-upgrade 2>&1

# Cleanup
print_divider
echo "Stopping Chopsticks..."
kill $chopsticks_pid 2>/dev/null || true
wait $chopsticks_pid 2>/dev/null || true
echo "Runtime upgrade succeed!"