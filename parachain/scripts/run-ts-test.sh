#!/usr/bin/env bash

set -eo pipefail

case "$1" in
    litentry|paseo) export PARACHAIN_TYPE=$1 ;;
    *) echo "usage: ./$0 litentry|paseo"; exit 1 ;;
esac

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR/parachain/ts-tests"

LITENTRY_PARACHAIN_DIR=${LITENTRY_PARACHAIN_DIR:-"/tmp/parachain_dev"}
[ -d "$LITENTRY_PARACHAIN_DIR" ] || mkdir -p "$LITENTRY_PARACHAIN_DIR"

[ -f .env ] || echo "NODE_ENV=ci" > .env
pnpm install
pnpm run test-filter 2>&1 | tee -a "$LITENTRY_PARACHAIN_DIR/parachain_ci_test.log"

$ROOTDIR/parachain/scripts/launch-bridge.sh
pnpm run test-bridge 2>&1 | tee -a "$LITENTRY_PARACHAIN_DIR/parachain_ci_test.log"

pnpm run test-evm-contract 2>&1 | tee -a "$LITENTRY_PARACHAIN_DIR/parachain_ci_test.log"
pnpm run test-precompile-contract 2>&1 | tee -a "$LITENTRY_PARACHAIN_DIR/parachain_ci_test.log"
