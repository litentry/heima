#!/usr/bin/env bash

# This scripts starts a standalone node without relaychain network locally, with the parachain runtime

set -eo pipefail

LITENTRY_PARACHAIN_DIR=${LITENTRY_PARACHAIN_DIR:-"/tmp/parachain_dev"}
[ -d "$LITENTRY_PARACHAIN_DIR" ] || mkdir -p "$LITENTRY_PARACHAIN_DIR"

ROOTDIR=$(git rev-parse --show-toplevel)
PARACHAIN_BIN="$ROOTDIR/parachain/target/release/heima-node"

cd "$ROOTDIR"

if [ ! -f "$PARACHAIN_BIN" ]; then
  echo "no heima-node found, build it now ..."
  make build-node
fi

if ! "$PARACHAIN_BIN" --version &> /dev/null; then
  echo "Cannot execute $PARACHAIN_BIN, wrong executable?"
  exit 1
fi

function print_divider() {
  echo "------------------------------------------------------------"
}

echo "Starting heima-node in standalone mode ..."

$PARACHAIN_BIN --dev --rpc-external --port "${CollatorPort:-30333}" --rpc-port "${CollatorWSPort:-9944}" \
  &> "$LITENTRY_PARACHAIN_DIR/para.alice.log" &

sleep 10

print_divider

# Check parachain status
echo "wait for parachain to produce block #1..."
cd "$ROOTDIR/parachain/ts-tests"

if [[ -z "${NODE_ENV}" ]]; then
    echo "NODE_ENV=ci" > .env
else
    echo "NODE_ENV=${NODE_ENV}" > .env
fi
corepack pnpm install
pnpm run wait-finalized-block 2>&1

print_divider
