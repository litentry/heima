#!/bin/bash

# Copyright 2020-2024 Trust Computing GmbH.

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
NPORT=${NPORT:-9912}
NODEURL=${NODEURL:-"ws://litentry-node"}
NODEHTTPURL=${NODEHTTPURL:-"http://litentry-node"}
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

cd /client-api/parachain-api
curl -s -H "Content-Type: application/json" -d '{"id": "1", "jsonrpc": "2.0", "method": "state_getMetadata", "params": []}' $NODEHTTPURL:$NPORT > prepare-build/litentry-parachain-metadata.json
echo "update parachain metadata"

cd  /client-api/sidechain-api
${CLIENT} print-sgx-metadata-raw > prepare-build/litentry-sidechain-metadata.json
echo "update sidechain metadata"


cd /client-api
pnpm install
pnpm run build

if [ "$TEST" = "assertion_contracts.test.ts" ]; then
    cd /
    ls assertion-contracts/
    cp -r assertion-contracts /ts-tests/integration-tests/contracts

    cd /ts-tests
    curl -L https://foundry.paradigm.xyz | bash
    source /root/.bashrc
    apt install -y git
    foundryup

    pnpm install
    pnpm --filter integration-tests run compile-contracts

else
    cd /ts-tests
    pnpm install

fi

NODE_ENV=staging pnpm --filter integration-tests run test $TEST
