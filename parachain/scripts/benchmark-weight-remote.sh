#!/usr/bin/env bash

set -eo pipefail

# This script benchmarks the runtime weight on the remote host.
#
# For now it doesn't support benchmarking the pallet weight, which is less
# important from where we stand as it will be overriden by runtime weight anyway

function usage() {
  echo "Usage: $0 litentry|paseo branch-or-tag pallet-names"
  echo "       branch-or-tag will be used to checkout codes"
  echo "       pallet-names can be either * or a comma listed pallet names"
  echo "e.g.:  $0 litentry dev *"
  echo "       $0 litentry dev frame-system,pallet-proxy,pallet-collective"
}

[ $# -ne 3 ] && (usage; exit 1)

# pull docker image
docker pull litentry/litentry-parachain:runtime-benchmarks

# clone the repo
TMPDIR=/tmp
cd "$TMPDIR"
[ -d litentry-parachain ] && rm -rf litentry-parachain
git clone https://github.com/litentry/litentry-parachain
cd litentry-parachain/parachain
git checkout "$2"

# copy binary out
docker cp "$(docker create --rm litentry/litentry-parachain:runtime-benchmarks):/usr/local/bin/litentry-collator" .
chmod a+x litentry-collator

# populate PALLETS
PALLETS=
case "$3" in
  '*')
    PALLETS=$(grep -F '[pallet_' runtime/$1/src/lib.rs | tr -d '\t' | grep -v "^ *//" | sed 's/.*\[//;s/,.*//' | paste -s -d' ' -)
    PALLETS="frame_system cumulus_pallet_xcmp_queue $PALLETS"
    ;;
  *)
    PALLETS=$(echo "$3" | tr ',' ' ')
    ;;
esac
PALLETS=${PALLETS//-/_}

echo "chain: $1"
echo "pallets: $PALLETS"

if [ -z "$PALLETS" ]; then
  echo "no pallets found"
  exit 1
fi

for p in $PALLETS; do
  echo "Start benchmarking pallet: $p for $1 ..."

  if [[ $p == *"parachain_staking"* ]]; then
      echo "will run $p benchmark code"
      STEPS=25
      REPEAT=20
  else
      echo "will run other pallet ($p) benchmark code"
      STEPS=20
      REPEAT=50
  fi

  # filter out the flooding warnings from pallet_scheduler:
  # Warning: There are more items queued in the Scheduler than expected from the runtime configuration.
  #          An update might be needed
  RUST_LOG=runtime::scheduler=error ./litentry-collator benchmark pallet \
        --chain=$1-dev \
        --execution=wasm  \
        --db-cache=20 \
        --wasm-execution=compiled \
        --pallet="$p" \
        --extrinsic=* \
        --heap-pages=4096 \
        --steps="$STEPS" \
        --repeat="$REPEAT" \
        --header=./LICENSE_HEADER \
        --output=./runtime/$1/src/weights/"$p".rs

  echo "================================================================"
  echo ""
done