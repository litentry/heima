#!/bin/bash

set -eo pipefail

# this script:
# - scrapes the state of a given parachain using `fork-off-substrate`
# - save the state snapshot to a chain spec JSON
# - use this chain spec to launch a local parachain network

ROOTDIR=$(git rev-parse --show-toplevel)

# setup TMPDIR
export TMPDIR=$(mktemp -d)
cleanup() {
  echo "removing $1 ..."
  rm -rf "$1"
}
trap 'cleanup $TMPDIR' INT TERM EXIT

FORK_OFF_SUBSTRATE_REPO="https://github.com/litentry/fork-off-substrate.git"

function print_divider() {
  echo "------------------------------------------------------------"
}

function usage() {
  print_divider
  echo "Usage: $0 [chain] [ws-rpc-endpoint] [binary]"
  echo
  echo "chain:             rococo|litentry"
  echo "                   default: rococo"
  echo "ws-rpc-endpoint:   the ws rpc endpoint of the parachain"
  echo "                   default: litentry-rococo's rpc endpoint"
  echo "binary:            path to the litentry parachain binary"
  echo "                   default: the binary copied from litentry/litentry-parachain:latest"
  print_divider
}

[ $# -gt 3 ] && (usage; exit 1)

case "$1" in
  help|-h|--help)
    usage
    exit 1
    ;;
  *)
    ;;
esac

ORIG_CHAIN=${1:-rococo}
FORK_CHAIN=${ORIG_CHAIN}-dev

case "$ORIG_CHAIN" in
  rococo)
    ENDPOINT="${2:-wss://rpc.litentry-parachain.litentry.io}"
    ;;
  litentry)
    ENDPOINT="${2:-wss://rpc.litentry-parachain.litentry.io}"
    ;;
  *)
    echo "unsupported chain type"
    exit 1 ;;
esac

cd "$TMPDIR"
git clone "$FORK_OFF_SUBSTRATE_REPO"
cd fork-off-substrate
npm i

mkdir data && cd data

# copy the binary
if [ -z "$3" ]; then
  docker cp "$(docker create --rm litentry/litentry-parachain:latest):/usr/local/bin/litentry-collator" binary
else
  cp -f "$3" binary
fi
chmod a+x binary

# write .env file
cd ..
cat << EOF > .env
WS_RPC_ENDPOINT=$ENDPOINT
ALICE=1
ORIG_CHAIN=$ORIG_CHAIN
FORK_CHAIN=$FORK_CHAIN
EOF

npm start

if [ ! -f data/fork.json ]; then
  echo "cannot find data/fork.json, please check it manually"
  exit 2
fi

cp -f data/fork.json "$ROOTDIR/parachain/docker/"

cd "$ROOTDIR/parachain"
sed -i.bak "s;$FORK_CHAIN;fork.json;" "docker/$ORIG_CHAIN-parachain-launch-config.yml"

# start the network
make "launch-docker-$ORIG_CHAIN"