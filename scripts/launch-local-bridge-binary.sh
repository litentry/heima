#!/usr/bin/env bash

LITENTRY_PARACHAIN_DIR=${LITENTRY_PARACHAIN_DIR:-"/tmp/parachain_dev"}
[ -d "$LITENTRY_PARACHAIN_DIR" ] || mkdir -p "$LITENTRY_PARACHAIN_DIR"

ROOTDIR=$(git rev-parse --show-toplevel)

GOPATH=${HOME}/go go install github.com/litentry/ChainBridge/cmd/chainbridge@sol0.8.19

cp ${HOME}/go/bin/chainbridge $LITENTRY_PARACHAIN_DIR/chainbridge

${ROOTDIR}/scripts/geth/run_geth.sh

