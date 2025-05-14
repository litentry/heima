#!/bin/bash

# Copyright 2020-2024 Trust Computing GmbH.

# Enable strict error handling: exit on error (-e), error on undefined vars (-u), exit on pipe failures (-o pipefail)
set -euo pipefail

while getopts ":p:A:u:W:V:C:" opt; do
    case $opt in
        p)
            NPORT=$OPTARG
            ;;
        A)
            WORKER1PORT=$OPTARG
            ;;
        u)
            NODEURL=$OPTARG
            ;;
        W)
            NODEHTTPURL=$OPTARG
            ;;
        V)
            WORKER1URL=$OPTARG
            ;;
        C)
            CLIENT_BIN=$OPTARG
            ;;
    esac
done

# Using default port if none given as arguments.
NPORT=${NPORT:-9944}
NODEURL=${NODEURL:-"ws://heima-node"}
NODEHTTPURL=${NODEHTTPURL:-"http://heima-node"}
WORKER1PORT=${WORKER1PORT:-2011}
WORKER1URL=${WORKER1URL:-"ws://litentry-worker-1"}

CLIENT_BIN=${CLIENT_BIN:-"/usr/local/bin/litentry-cli"}

CLIENT="${CLIENT_BIN} -p ${NPORT} -P ${WORKER1PORT} -u ${NODEURL} -U ${WORKER1URL}"

function usage() {
    echo ""
    echo "This is a script for tee-worker integration ts tests. Pass test name as first argument"
    echo ""

}

[ $# -ne 1 ] && (usage; exit 1)
TEST=$1

echo "Using client binary $CLIENT_BIN"
echo "Using node uri $NODEURL:$NPORT"
echo "Using trusted-worker uri $WORKER1URL:$WORKER1PORT"
echo "Using node http uri $NODEHTTPURL:$NPORT"
echo ""

cd /client-api
curl -s -H "Content-Type: application/json" -d '{"id": "1", "jsonrpc": "2.0", "method": "state_getMetadata", "params": []}' $NODEHTTPURL:$NPORT > metadata-parachain.json
echo "update parachain metadata"


# Due to the sidechain being built into the client-api, we need to get all the metadata before the client-api can be built normally.
cd  /client-api
${CLIENT} print-sgx-metadata-raw > metadata-sidechain.json
echo "update sidechain metadata"


cd /client-api
pnpm install
pnpm run build

echo "Installing dependencies and building ts-tests"
cd /ts-tests
pnpm install --force

echo "Running integration tests"
OMNI_WORKER_ENDPOINT=ws://omni-executor:2100 PARACHAIN_ENDPOINT=ws://heima-node:9944 pnpm --filter integration-tests run test $TEST
