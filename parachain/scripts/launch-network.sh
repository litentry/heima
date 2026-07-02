#!/usr/bin/env bash

# This scripts starts a local network with 2 relaychain nodes + 1 parachain node,
# backed by zombienet

set -eo pipefail

function usage() {
  echo "Usage: $0 heima|paseo"
}

function print_divider() {
  echo "------------------------------------------------------------"
}

[ $# -lt 1 ] && (usage; exit 1)

CHAIN=$1

ZOMBIENET_VERSION=v1.3.117
ZOMBIENET_DIR=$(LC_ALL=C tr -dc A-Za-z0-9 </dev/urandom | head -c 8; echo)

HEIMA_DIR=${HEIMA_DIR:-"/tmp/parachain_dev"}
[ -d "$HEIMA_DIR" ] || mkdir -p "$HEIMA_DIR"

ROOTDIR=$(git rev-parse --show-toplevel)
PARACHAIN_BIN="$ROOTDIR/parachain/target/release/heima-node"
cd "$ROOTDIR"

export PARA_ID=$(grep -i "${CHAIN}_para_id" parachain/primitives/src/lib.rs | sed 's/.* = //;s/\;.*//')
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

cd "$HEIMA_DIR"
export PATH="$HEIMA_DIR:$PATH"
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

echo "searching heima-node binary in target/release/ ..."

if [ -f "$PARACHAIN_BIN" ]; then
  cp "$PARACHAIN_BIN" .
  echo "found one, version:"
  ./heima-node --version
else
  echo "not here, copying from docker image if we are on Linux ..."
  if [ $(uname -s) = "Linux" ]; then
    docker cp "$(docker create --rm litentry/heima:latest):/usr/local/bin/heima-node" .
    chmod +x heima-node
    echo "done, version:"
    ./heima-node --version
  fi
fi

print_divider

echo "launching zombienet network (in background), dir = $ZOMBIENET_DIR ..."
# Log at `text` (not `silent`) and keep the orchestrator output in a file so it is captured
# by CI's "Archive logs if test fails" step. Write it as a sibling of the zombienet dir
# (NOT inside it — zombienet requires `-d` to be a fresh/absent directory and pre-creating
# it breaks spawn). Per-node logs still land in $ZOMBIENET_DIR/*.log.
# (`-l` accepts only table|text|silent; `text` is the plain-text verbose mode.)
nohup $ZOMBIENET_BIN -d $ZOMBIENET_DIR -l text spawn config.toml > "$HEIMA_DIR/zombienet-$ZOMBIENET_DIR.log" 2>&1 &

cd "$ROOTDIR/parachain/ts-tests"

if [ -z "$NODE_ENV" ]; then
    echo "NODE_ENV=ci" > .env
else
    echo "NODE_ENV=$NODE_ENV" > .env
fi
corepack pnpm install

# Wait for the collator RPC to be up before the ts client connects. zombienet spawn +
# (in the release path) docker image load can take a while, and the polkadot-js client
# does not reliably recover if it opens before the RPC port is listening — it then hangs
# in `ApiPromise.create()` forever (past its own timeout). Poll `system_health` on the
# collator's RPC port until it answers, up to ~3 min.
echo "waiting for collator RPC on 127.0.0.1:$COLLATOR_WS_PORT to be ready..."
rpc_ready=false
for i in $(seq 1 90); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
      -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
      "http://127.0.0.1:$COLLATOR_WS_PORT" 2>/dev/null | grep -q '"result"'; then
    echo "collator RPC is ready (after ~$((i*2))s)"
    rpc_ready=true
    break
  fi
  sleep 2
done
if [ "$rpc_ready" != true ]; then
  echo "collator RPC did not come up within ~3min; continuing anyway (wait-finalized-block will report)"
fi

echo "wait for parachain to produce block #1..."
pnpm run wait-finalized-block 2>&1

echo
echo "to stop the network, run 'make clean-network'"

print_divider
