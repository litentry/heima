#!/bin/bash
# set -eo we don't allow any command failed in this script.
set -eo pipefail

ROOTDIR=$(git rev-parse --show-toplevel)
new_wasm=/tmp/runtime.wasm

function usage() {
  echo
  echo "Usage: $0 <wasm-name> <endpoint> <release-tag> "
  echo "e.g.:"
  echo "    $0 litentry wss://rpc.litentry-parachain.litentry.io v0.9.21-01"
}

[ $# -ne 3 ] && (usage; exit 1)

function print_divider() {
  echo "------------------------------------------------------------"
}

# Download runtime wasm
print_divider
echo "Download $1-parachain-runtime.compact.compressed.wasm from release tag $3 ..."
gh release download "$3" -p "$1-parachain-runtime.compact.compressed.wasm" -O "$new_wasm" || true

if [ -f "$new_wasm" ] && [ -s "$new_wasm" ]; then
  ls -l "$new_wasm"
else
  echo "Cannot find $new_wasm or it has 0 bytes, quit"
  exit 0
fi

# Install tools
print_divider
wget -q https://github.com/vi/websocat/releases/latest/download/websocat.x86_64-unknown-linux-musl -O websocat
chmod +x websocat
echo "Websocat version: $(./websocat --version)"

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
echo "nvm version: $(nvm --version)"

# Check if the released runtime version is greater than the on-chain runtime version,
print_divider
echo "Check runtime version ..."
release_version=$(subwasm --json info "$new_wasm" | jq .core_version.specVersion)

PAYLOAD='{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion", "params": [] }'
RETRY_INTERVAL=5
MAX_RETRIES=10
i=0
while ((i<MAX_RETRIES)); do
    echo "Attempt $i: Trying to fetch on-chain version from $2..."
    response=$(echo "$PAYLOAD" | ./websocat "$2") || echo ""
    onchain_version=$(echo "$response" | jq -r .result.specVersion 2>/dev/null)
    if [[ -n "$onchain_version" && "$onchain_version" != "null" ]]; then
        break
    else
        echo "Invalid or no response. Retrying in $RETRY_INTERVAL seconds..."
        sleep $RETRY_INTERVAL
    fi
    i=$((i + 1))
done
if [ "$i" -ge $MAX_RETRIES ]; then
  echo "Failed to fetch on-chain version after $MAX_RETRIES attempts."
  exit 1
fi

echo "On-chain: $onchain_version"
echo "Release:  $release_version"

if [ -n "$release_version" ] && \
   [ -n "$onchain_version" ] && \
   [ "$onchain_version" -ge "$release_version" ]; then
  echo "Current On-chain runtime is up to date, quit"
  exit 1
fi

# 4. do runtime upgrade and verify
print_divider
echo "Do runtime upgrade and verify ..."

nvm install 20
echo "start chopsticks: $1"
npx @acala-network/chopsticks@1.0.1 --endpoint=$2  --port=9944  --mock-signature-host=true --db=./new-db.sqlite  --runtime-log-level=5 --allow-unresolved-imports=true --wasm-override $new_wasm &
PID=$!
echo "Chopsticks fork parachain PID: $PID"
sleep 30

echo "after chopsticks: $1"
new_onchain_version=$(curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion", "params": [] }' http://localhost:9944 | jq .result.specVersion)
if [ -n "$new_onchain_version" ] && \
   [ "$new_onchain_version" -ne "$release_version" ]; then
  echo "On-chain new: $new_onchain_version"
  echo "Runtime version NOT increased successfully, quit"
  exit 1
fi

echo "Runtime upgrade succeed: $new_onchain_version"

print_divider
echo "Done"

