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
NPORT=${NPORT:-9944}
NODEURL=${NODEURL:-"ws://127.0.0.1"}
NODEHTTPURL=${NODEHTTPURL:-"http://127.0.0.1"}
WORKER1PORT=${WORKER1PORT:-2000}
WORKER1URL=${WORKER1URL:-"wss://127.0.0.1"}

CLIENT_BIN=${CLIENT_BIN:-"./../bin/litentry-cli"}

CLIENT="${CLIENT_BIN} -p ${NPORT} -P ${WORKER1PORT} -u ${NODEURL} -U ${WORKER1URL}"

echo "Using client binary $CLIENT_BIN"
echo "Using node uri $NODEURL:$NPORT"
echo "Using trusted-worker uri $WORKER1URL:$WORKER1PORT"
echo "Using node http uri $NODEHTTPURL:$NPORT"
echo "waiting 20 secs worker to run successfully"
sleep 20
cd /client-api
curl -s -H "Content-Type: application/json" -d '{"id": "1", "jsonrpc": "2.0", "method": "state_getMetadata", "params": []}' $NODEHTTPURL:$NPORT > metadata-parachain.json
echo "update parachain metadata"

cd  /client-api/sidechain-api
${CLIENT} print-sgx-metadata-raw > metadata-sidechain.json
echo "update sidechain metadata"

cd /client-api
pnpm install
pnpm run build