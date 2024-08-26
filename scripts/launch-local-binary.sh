#!/usr/bin/env bash

# This scripts starts a local network with 2 relaychain nodes + 1 parachain node.
# The binaries are passed as arguments for this script.
#
# mainly used on CI-runner, where:
# - The `polkadot` binary will be downloaded directly from official release.
# - The `litentry-collator` binary will be copied out from the litentry/litentry-parachain:latest image.
#
# To use this script locally, you might have to first compile the binaries that can run on your OS.

set -eo pipefail

function usage() {
  echo
  echo "Usage:   $0 litentry|litmus|rococo [path-to-polkadot-bin] [path-to-litentry-collator]"
  echo "Default: polkadot bin from the latest official release"
  echo "         litentry-collator bin from litentry/litentry-parachain:latest image"
  echo "         both are of Linux verion"
}

[ $# -lt 1 ] && (usage; exit 1)

CHAIN=$1
POLKADOT_BIN="$2"
PARACHAIN_BIN="$3"

LITENTRY_PARACHAIN_DIR=${LITENTRY_PARACHAIN_DIR:-"/tmp/parachain_dev"}

[ -d "$LITENTRY_PARACHAIN_DIR" ] || mkdir -p "$LITENTRY_PARACHAIN_DIR"
ROOTDIR=$(git rev-parse --show-toplevel)

cd "$ROOTDIR"
PARACHAIN_ID=$(grep -i "${CHAIN}_para_id" primitives/core/src/lib.rs | sed 's/.* = //;s/\;.*//')
export PARACHAIN_ID

function print_divider() {
  echo "------------------------------------------------------------"
}

print_divider

if [ -z "$POLKADOT_BIN" ]; then
  echo "no polkadot binary provided, download now ..."
  # TODO: find a way to get stable download link
  # https://api.github.com/repos/paritytech/polkadot/releases/latest is not reliable as
  # polkadot could publish release which has no binary
  #
  for f in polkadot-execute-worker polkadot-prepare-worker polkadot; do
    url="https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-v1.1.0/$f"
    # eventually `POLKADOT_BIN` is the `polkadot` binary
    POLKADOT_BIN="$LITENTRY_PARACHAIN_DIR/$f"
    wget -O "$POLKADOT_BIN" -q "$url"
    chmod a+x "$POLKADOT_BIN"
  done
fi

if [ ! -s "$POLKADOT_BIN" ]; then
  echo "$POLKADOT_BIN is 0 bytes, download URL: $url"
  exit 1
fi

if ! "$POLKADOT_BIN" --version &> /dev/null; then
  echo "Cannot execute $POLKADOT_BIN, wrong executable?"
  usage
  exit 1
fi

if [ -z "$PARACHAIN_BIN" ]; then
  echo "no litentry-collator binary provided, build it now ..."
  make build-node
  PARACHAIN_BIN="$ROOTDIR/target/release/litentry-collator"
  chmod a+x "$PARACHAIN_BIN"
fi

if ! "$PARACHAIN_BIN" --version &> /dev/null; then
  echo "Cannot execute $PARACHAIN_BIN, wrong executable?"
  usage
  exit 1
fi

cd "$LITENTRY_PARACHAIN_DIR"

echo "starting dev network with binaries ..."

# generate chain spec
ROCOCO_CHAINSPEC=rococo-local-chain-spec.json
$POLKADOT_BIN build-spec --chain rococo-local --disable-default-bootnode --raw > $ROCOCO_CHAINSPEC

# generate genesis state and wasm for registration
$PARACHAIN_BIN export-genesis-state --chain $CHAIN-dev > genesis-state
$PARACHAIN_BIN export-genesis-wasm --chain $CHAIN-dev > genesis-wasm

# run alice and bob as relay nodes
$POLKADOT_BIN --chain $ROCOCO_CHAINSPEC --alice --tmp --port ${AlicePort:-30336} --rpc-port ${AliceWSPort:-9946} &> "relay.alice.log" &
sleep 10

$POLKADOT_BIN --chain $ROCOCO_CHAINSPEC --bob --tmp --port ${BobPort:-30337} --rpc-port ${BobWSPort:-9947}  &> "relay.bob.log" &
sleep 10

# run a litentry-collator instance
$PARACHAIN_BIN --alice --collator --force-authoring --tmp --chain $CHAIN-dev \
  --unsafe-rpc-external --rpc-cors=all \
  --port ${CollatorPort:-30333} --rpc-port ${CollatorWSPort:-9944} --execution wasm \
  --state-pruning archive --blocks-pruning archive \
  --enable-evm-rpc \
  -- \
  --execution wasm --chain $ROCOCO_CHAINSPEC --port 30332 --rpc-port 9943 &> "para.alice.log" &
sleep 10

# Prepare Node.js enviroment
cd "$ROOTDIR/ts-tests"

if [[ -z "${NODE_ENV}" ]]; then
    echo "NODE_ENV=ci" > .env
else
    echo "NODE_ENV=${NODE_ENV}" > .env
fi
corepack pnpm install


echo "register parathread now ..."
corepack pnpm run register-parathread 2>&1 | tee "$LITENTRY_PARACHAIN_DIR/register-parathread.log"

print_divider

echo "upgrade parathread to parachain in 90s..."
# Wait for 90s to allow onboarding finish, after that we do the upgrade
sleep 90
corepack pnpm run upgrade-parathread 2>&1 | tee "$LITENTRY_PARACHAIN_DIR/upgrade-parathread.log"

print_divider

echo "wait for parachain to produce block #1..."
pnpm run wait-finalized-block 2>&1

echo
echo "Check $LITENTRY_PARACHAIN_DIR for generated files if need"

print_divider
