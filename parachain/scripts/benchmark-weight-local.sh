#!/usr/bin/env bash

set -eo pipefail

# This script benchmarks the runtime or pallet weight locally.
#
# Benchmarking pallet weight only works for the local pallets. Substrate (or other github) pallets are not supported:
# they are already benchmarked anyway (e.g. SubstrateWeigt)
#
# The `heima-collator` binary must be compiled with `runtime-benchmarks` feature.

function usage() {
    echo "Usage: $0 litentry|paseo pallet-name runtime|pallet"
}

[ $# -ne 3 ] && (usage; exit 1)

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR/parachain"

PALLET=${2//-/_}

echo "running $1-$3 benchmark for $PALLET locally ..."

case "$3" in
    runtime)
        OUTPUT="--output=./runtime/$1/src/weights/$PALLET.rs"
        TEMPLATE=
        CHAIN="--chain=$1-dev"
        ;;
    pallet)
        OUTPUT="$(cargo pkgid -q $2 | sed 's/.*litentry-parachain/\./;s/#.*/\/src\/weights.rs/')"
        TEMPLATE="--template=./templates/benchmark/pallet-weight-template.hbs"
        CHAIN="--chain=$1-dev"
        if [[ $OUTPUT == *"github.com"* ]]; then
            echo "only local pallets can be benchmarked"
            exit 2
        else
            OUTPUT="--output=$OUTPUT"
        fi
        ;;
    *)
        usage
        exit 3
        ;;
esac

if [[ $PALLET == *"parachain_staking"* ]]; then
    echo "will run $PALLET benchmark code"
    STEPS=25
    REPEAT=20
else
    echo "will run other pallet ($PALLET) benchmark code"
    STEPS=20
    REPEAT=50
fi

./target/release/heima-collator benchmark pallet \
          $CHAIN \
          --execution=wasm  \
          --db-cache=20 \
          --wasm-execution=compiled \
          --pallet="$PALLET" \
          --extrinsic=* \
          --heap-pages=4096 \
          --steps="$STEPS" \
          --repeat="$REPEAT" \
          --header=./LICENSE_HEADER \
          $TEMPLATE \
          $OUTPUT
