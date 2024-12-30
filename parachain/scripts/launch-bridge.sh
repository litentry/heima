#!/usr/bin/env bash

HEIMA_DIR=${HEIMA_DIR:-"/tmp/parachain_dev"}
[ -d "$HEIMA_DIR" ] || mkdir -p "$HEIMA_DIR"

ROOTDIR=$(git rev-parse --show-toplevel)

if [[ "$(docker image inspect litentry/chainbridge:latest 2>/dev/null)" == "[]" ]]; then
    echo "litentry/chainbridge:latest image not found..."
    ${ROOTDIR}/parachain/scripts/build-bridge.sh
fi

echo "------------------------------------------------------------"

docker run -d --rm --name chainbridge litentry/chainbridge bash -c 'ls /go/bin/ && sleep 5'
docker cp chainbridge:/go/bin/chainbridge ${HEIMA_DIR}/
echo "copy binary:chainbridge to ${HEIMA_DIR}"

echo "------------------------------------------------------------"

docker rm -f geth &>/dev/null

# use the last stable release v1.13.14
# the `latest` image introduces the geth-1.14 unstable version which emits an error when starting the network:
# ```
#  Fatal: Failed to register the Ethereum service: only PoS networks are supported, please transition old ones with Geth v1.13.x
# ```
#
# TODO - make it work with latest image
docker run -d --rm --entrypoint 'sh' --name 'geth' \
    -u "$(id -u)":"$(id -g)" -v ${ROOTDIR}/parachain/scripts/geth:/data/ -p 8546:8546 -p 8545:8545 \
    ethereum/client-go:v1.13.14 /data/run_geth.sh docker /data
echo "runing geth...(container name: geth)"
