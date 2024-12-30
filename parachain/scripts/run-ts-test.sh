#!/usr/bin/env bash

set -eo pipefail

case "$1" in
    heima|paseo) export PARACHAIN_TYPE=$1 ;;
    *) echo "usage: ./$0 heima|paseo"; exit 1 ;;
esac

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR/parachain/ts-tests"

HEIMA_DIR=${HEIMA_DIR:-"/tmp/parachain_dev"}
[ -d "$HEIMA_DIR" ] || mkdir -p "$HEIMA_DIR"

[ -f .env ] || echo "NODE_ENV=ci" > .env
pnpm install
pnpm run test-filter 2>&1 | tee -a "$HEIMA_DIR/parachain_ci_test.log"

$ROOTDIR/parachain/scripts/launch-bridge.sh
pnpm run test-bridge 2>&1 | tee -a "$HEIMA_DIR/parachain_ci_test.log"

pnpm run test-evm-contract 2>&1 | tee -a "$HEIMA_DIR/parachain_ci_test.log"
pnpm run test-precompile-contract 2>&1 | tee -a "$HEIMA_DIR/parachain_ci_test.log"
