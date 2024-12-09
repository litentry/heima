#!/usr/bin/env bash

set -euo pipefail

PARACHAIN="paseo"
ROOTDIR=$(git rev-parse --show-toplevel)
ROOTDIR="${ROOTDIR}/tee-worker"

function usage() {
    echo "Usage: $0 <dev|staging|prod|mock>"
    echo ""
    echo "  All mode apply to ${PARACHAIN} context."
    echo "      dev: start worker(s) together with local ${PARACHAIN} for development"
    echo "  staging: start worker(s) sync with staging ${PARACHAIN} on tee-staging server"
    echo "     prod: start worker(s) sync with production ${PARACHAIN} on polkadot.js"
    echo "     mock: start worker(s) together with local ${PARACHAIN} for development"
}

function start_local_parachain() {
    cd ${ROOTDIR}
    echo "------------------------------------------------------------"
    echo "Start local parachain: ${PARACHAIN} ..."
    # TODO: only `paseo` is supported for the moment. And it's hard-coded inside `start_parachain.sh`
    ./../local-setup/start_parachain.sh
    if [ $? -ne 0 ]; then
        exit 1
    fi
}

function start_worker_for_dev() {
    start_local_parachain
    cd ${ROOTDIR}
    worker_num=2
    echo "------------------------------------------------------------"
    echo "Start ${worker_num} workers with dev ${PARACHAIN} ..."
    ./scripts/launch_local_worker.sh -c true -n ${worker_num} -m "dev"
}

function start_worker_for_staging() {
    cd ${ROOTDIR}
    worker_num=2
    # staging_parachain_url
    url="wss://tee-staging.litentry.io"
    # staging_parachain_port
    port=443
    echo "------------------------------------------------------------"
    echo "Start ${worker_num} workers with staging ${PARACHAIN} ..."
    ./scripts/launch_local_worker.sh -c true -n ${worker_num} -u ${url} -p ${port} -m "staging"
}

function start_worker_for_prod() {
    cd ${ROOTDIR}
    worker_num=2
    # production_parachain_url
    url="wss://rpc.${PARACHAIN}-parachain-sg.litentry.io"
    # production_parachain_port
    port=443
    echo "------------------------------------------------------------"
    echo "Start ${worker_num} workers with production ${PARACHAIN} ..."
    ./scripts/launch_local_worker.sh -c true -n ${worker_num} -u ${url} -p ${port} -m "prod"
}

function start_worker_for_mock() {
    start_local_parachain
    cd ${ROOTDIR}
    worker_num=2
    echo "------------------------------------------------------------"
    echo "Start ${worker_num} workers with local ${PARACHAIN} ..."
    ./scripts/launch_local_worker.sh -c true -n ${worker_num} -m "mock"
}


[ $# -ne 1 ] && (usage; exit 1)
MODE=$1

if [ "$MODE" = "dev" ] || [ "$MODE" = "staging" ] || [ "$MODE" = "prod" ] || [ "$MODE" = "mock" ]; then
    echo "Launch in $MODE mode"
    start_worker_for_$MODE
else
    echo "Unknow mode: $MODE"
    usage; exit 1
fi

echo "Done"






