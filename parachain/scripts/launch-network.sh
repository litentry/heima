#!/usr/bin/env bash

# This scripts starts a local network with 2 relaychain nodes + 1 parachain node,
# backed by zombienet

set -eo pipefail

function usage() {
  echo "Usage: $0 litentry|paseo"
}

function print_divider() {
  echo "------------------------------------------------------------"
}

[ $# -lt 1 ] && (usage; exit 1)

CHAIN=$1

ZOMBIENET_VERSION=v1.3.117
ZOMBIENET_DIR=$(LC_ALL=C tr -dc A-Za-z0-9 </dev/urandom | head -c 8; echo)

LITENTRY_PARACHAIN_DIR=${LITENTRY_PARACHAIN_DIR:-"/tmp/parachain_dev"}
[ -d "$LITENTRY_PARACHAIN_DIR" ] || mkdir -p "$LITENTRY_PARACHAIN_DIR"

ROOTDIR=$(git rev-parse --show-toplevel)
PARACHAIN_BIN="$ROOTDIR/parachain/target/release/litentry-collator"
cd "$ROOTDIR"

export PARA_ID=$(grep -i "${CHAIN}_para_id" common/primitives/core/src/lib.rs | sed 's/.* = //;s/\;.*//')
export PARA_CHAIN_SPEC=${CHAIN}-dev
export COLLATOR_WS_PORT=${CollatorWSPort:-9944}

case $(uname -s) in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) echo "Unsupported os"; exit 1 ;;
esac

case $(uname -m) in
  aarch64*) arch=arm64 ;;
  x86_64) arch=x64 ;;
  *) echo "Unsuppported arch"; exit 1 ;;
esac

ZOMBIENET_BIN=zombienet-${os}-${arch}

cd "$LITENTRY_PARACHAIN_DIR"
export PATH="$LITENTRY_PARACHAIN_DIR:$PATH"
cp "$ROOTDIR/parachain/zombienet/config.toml" .

if ! $ZOMBIENET_BIN version &> /dev/null; then
  echo "downloading $ZOMBIENET_BIN ..."
  curl -L -s -O "https://github.com/paritytech/zombienet/releases/download/$ZOMBIENET_VERSION/$ZOMBIENET_BIN"
  chmod +x "$ZOMBIENET_BIN"
fi

echo "checking $ZOMBIENET_BIN version ..."
$ZOMBIENET_BIN version

echo "downloading polkadot ..."
$ZOMBIENET_BIN setup polkadot -y || true

echo "searching litentry-collator binary in target/release/ ..."

if [ -f "$PARACHAIN_BIN" ]; then
  cp "$PARACHAIN_BIN" .
  echo "found one, version:"
  ./litentry-collator --version
else
  echo "not here, copying from docker image if we are on Linux ..."
  if [ $(uname -s) = "Linux" ]; then
    docker cp "$(docker create --rm litentry/litentry-parachain:latest):/usr/local/bin/litentry-collator" .
    chmod +x litentry-collator
    echo "done, version:"
    ./litentry-collator --version
  fi
fi

print_divider

echo "launching zombienet network (in background), dir = $ZOMBIENET_DIR ..."
nohup $ZOMBIENET_BIN -d $ZOMBIENET_DIR -l silent spawn config.toml > /dev/null 2>&1 &

cd "$ROOTDIR/parachain/ts-tests"

if [ -z "$NODE_ENV" ]; then
    echo "NODE_ENV=ci" > .env
else
    echo "NODE_ENV=$NODE_ENV" > .env
fi
corepack pnpm install

echo "wait for parachain to produce block #1..."
pnpm run wait-finalized-block 2>&1

echo
echo "to stop the network, run 'make clean-network'"

print_divider
