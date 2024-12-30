#!/usr/bin/env bash

HEIMA_DIR=${HEIMA_DIR:-"/tmp/parachain_dev"}
[ -d "$HEIMA_DIR" ] || mkdir -p "$HEIMA_DIR"

DOCKER='local'

if [ -n "$1" ]; then
    DOCKER='docker'
fi

if [ -n "$2" ]; then
    ROOTDIR=$2
else
    ROOTDIR="$(git rev-parse --show-toplevel)/scripts/geth"
fi

GETH_BIN="geth"
if ! geth version &>/dev/null; then
    echo "geth could not be found..download now"
    url="https://gethstore.blob.core.windows.net/builds/geth-linux-386-1.13.14-2bd6bd01.tar.gz"
    GETH_BIN="$HEIMA_DIR/geth"
    wget -O "$HEIMA_DIR/geth.tar.gz" -q "$url"
    tar -xf "$HEIMA_DIR/geth.tar.gz" --strip-components 1 -C "$HEIMA_DIR"
    chmod a+x "$GETH_BIN"
fi

DATADIR="${ROOTDIR}/gethdata"
# Exit on failure
set -eo pipefail

# Delete old chain data
rm -rf ${DATADIR}
# Init genesis
eval "${GETH_BIN} init --datadir ${DATADIR} ${ROOTDIR}/genesis.json"
# Copy keystore
rm -rf ${DATADIR}/keystore
cp -r ${ROOTDIR}/keystore ${DATADIR}
# Start geth with rpc, mining and unlocked accounts

start="${GETH_BIN} --datadir ${DATADIR} \
    --nodiscover \
    --unlock '0xff93B45308FD417dF303D6515aB04D9e89a750Ca','0x8e0a907331554AF72563Bd8D43051C2E64Be5d35','0x24962717f8fA5BA3b931bACaF9ac03924EB475a0','0x148FfB2074A9e59eD58142822b3eB3fcBffb0cd7','0x4CEEf6139f00F9F4535Ad19640Ff7A0137708485' \
    --password ${ROOTDIR}/password.txt \
    --ws \
    --ws.port 8546 \
    --ws.addr '0.0.0.0' \
    --networkid 5 \
    --ws.origins='*' \
    --http \
    --http.port 8545 \
    --http.addr '0.0.0.0' \
    --http.corsdomain='*' \
    --miner.gaslimit 8000000 \
    --allow-insecure-unlock \
    --mine \
    --miner.etherbase 0xff93B45308FD417dF303D6515aB04D9e89a750Ca
"
if [ ${DOCKER} = "local" ]; then
    eval ${start} &>'geth.log' &
else
    eval ${start}
fi