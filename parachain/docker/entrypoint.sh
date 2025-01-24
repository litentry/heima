#!/bin/bash
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

PARACHAIN_BASEDIR="/opt/litentry/parachain"
REPO_DIR="/code"
CHAIN='paseo'
ZOMBIENET_DIR=$(LC_ALL=C tr -dc A-Za-z0-9 </dev/urandom | head -c 8; echo)

check(){
    if [ -z "$CHAIN" ]; then
        export CHAIN="paseo"
    fi

    ALLOWED_VALUES=("paseo" "litentry")

    if [[ " ${ALLOWED_VALUES[@]} " =~ " ${CHAIN} " ]]; then
        echo "CHAIN is set to a valid value: $CHAIN"
    else
        echo "Error: CHAIN environment variable must be one of: ${ALLOWED_VALUES[@]}"
        exit 1
    fi
}

init(){
    export PARA_ID=$(grep -i "${CHAIN}_para_id" ${REPO_DIR}/common/primitives/core/src/lib.rs | sed 's/.* = //;s/\;.*//')
    export PARA_CHAIN_SPEC=${CHAIN}-dev
    export COLLATOR_WS_PORT=${CollatorWSPort:-9944}
}

run_zombienet(){
    cd "$PARACHAIN_BASEDIR" || exit
    cp ${REPO_DIR}/parachain/zombienet/config.toml .
    zombienet setup polkadot -y || true
    export PATH=${PATH}:${PARACHAIN_BASEDIR}
    cp /usr/local/bin/heima-collator .
    nohup zombienet -d $ZOMBIENET_DIR -l silent spawn config.toml >> ${PARACHAIN_BASEDIR}/zombienet.log 2>&1 &
    zombienet_pid=$!
}

register_parachain(){
    echo "Register parathread now ..."
    cd "$REPO_DIR/parachain/ts-tests" || exit
    if [[ -z "$NODE_ENV" ]]; then
        echo "NODE_ENV=ci" > .env
    else
        echo "NODE_ENV=$NODE_ENV" > .env
    fi

    corepack pnpm install

    echo "wait for parachain to produce block #1..."
    pnpm run wait-finalized-block 2>&1

    echo
    echo "done. please check $PARACHAIN_BASEDIR for generated files if need"
}

print_help(){
    echo "Parachain ${CHAIN} initialized successfully!"
    echo "If you need to monitor the logs, please try the command 'docker exec <containers name> tail -f ${PARACHAIN_BASEDIR}/zombienet.log'."
    echo "Next, it will enter daemon mode."
}

watch_pid(){
    wait -n ${zombienet_pid}
    EXIT_STATUS=$?
    kill ${zombienet_pid}
    exit $EXIT_STATUS
}

main(){
    check
    init
    run_zombienet
    register_parachain
    print_help
    watch_pid
}

main 
