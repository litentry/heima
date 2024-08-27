#!/usr/bin/env bash

set -eo pipefail

function usage() {
  echo "Usage: $0 litentry|rococo"
}

[ $# -ne 1 ] && (usage; exit 1)

CHAIN=$1

function print_divider() {
  echo "------------------------------------------------------------"
}

ROOTDIR=$(git rev-parse --show-toplevel)

cd "$ROOTDIR"
PARACHAIN_ID=$(grep -i "${CHAIN}_para_id" primitives/core/src/lib.rs | sed 's/.* = //;s/\;.*//')
export PARACHAIN_ID

cd "$ROOTDIR/docker/generated-$CHAIN/"

docker compose up -d --build

print_divider

# Install Node.js dependencies in the middle.
# It also buys `docker compose` some time.
cd "$ROOTDIR/ts-tests"
if [[ -z "${NODE_ENV}" ]]; then
    echo "NODE_ENV=ci" > .env
else
    echo "NODE_ENV=${NODE_ENV}" > .env
fi

pnpm install

print_divider

echo "Extending leasing period..."
pnpm run upgrade-parathread 2>&1

print_divider

echo "Waiting for parachain to produce block #1..."
pnpm run wait-finalized-block 2>&1

exit 0
