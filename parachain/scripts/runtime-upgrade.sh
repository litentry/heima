#!/bin/bash
# set -eo we don't allow any command failed in this script.
set -eo pipefail

ROOTDIR=$(git rev-parse --show-toplevel)

# the script is used to simulate runtime upgrade, see:
# https://github.com/litentry/heima/issues/378

# The latest state of the blockchain is scraped and used to bootstrap a chain locally via fork-off-substrate,
# see ./scripts/fork-parachain-and-launch.sh
#
# After that, this script:
# 1. get the runtime wasm
# 2. do runtime upgrade using wasm from step 1
# 3. verify if the runtime upgrade is successful

output_wasm=/tmp/runtime.wasm

function usage() {
  echo
  echo "Usage: $0 wasm-name [release-tag]"
}

[ $# -gt 2 ] && (usage; exit 1)

function print_divider() {
  echo "------------------------------------------------------------"
}

print_divider

# 1. download or copy runtime wasm
if [ -z "$2" ]; then
  echo "Copy local wasm $1 ..."
  cp -f "$1" "$output_wasm"
else
  echo "Download $1 from release tag $2 ..."
  gh release download "$2" -p "$1" -O "$output_wasm" || true
fi

if [ -f "$output_wasm" ] && [ -s "$output_wasm" ]; then
  ls -l "$output_wasm"
else
  echo "Cannot find $output_wasm or it has 0 bytes, quit"
  exit 0
fi

print_divider

# 2. check if the released runtime version is greater than the on-chain runtime version,
#    which should be now accessible via localhost:9944
onchain_version=$(curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion", "params": [] }' http://localhost:9944 | jq .result.specVersion)
release_version=$(subwasm --json info "$output_wasm" | jq .core_version.specVersion)

echo "Check runtime version ..."
echo "On-chain: $onchain_version"
echo "Release:  $release_version"

if [ -n "$release_version" ] && \
   [ -n "$onchain_version" ] && \
   [ "$onchain_version" -ge "$release_version" ]; then
  echo "Runtime version not increased, quit"
  exit 0
fi

print_divider

# 3. do runtime upgrade and verify
echo "Do runtime upgrade and verify ..."
cd "$ROOTDIR/parachain/ts-tests"
echo "NODE_ENV=ci" > .env
pnpm install && pnpm run test-runtime-upgrade 2>&1

print_divider

echo "Done"