#!/usr/bin/env bash
set -eo pipefail

function usage() {
  echo "Usage: $0 litentry|paseo [docker-tag]"
}

[ $# -lt 1 ] && (usage; exit 1)

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR/parachain"

CHAIN=$1
TAG=${2:-latest}
image="litentry/litentry-parachain:$TAG"

docker pull "$image"

docker run --rm "$image" build-spec --chain=generate-$CHAIN > node/res/chain_specs/$CHAIN-plain.json
docker run --rm "$image" build-spec --chain=generate-$CHAIN --raw > node/res/chain_specs/$CHAIN.json

echo "Done, please check node/res/chain_specs/"
ls -l node/res/chain_specs/
