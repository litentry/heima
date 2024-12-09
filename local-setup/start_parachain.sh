#!/bin/bash
set -euo pipefail

# check the port before launching the parachain
# this script is called in bothe launch.sh and launch.py
#
# please note this check doesn't apply to standalone litentry-node
# as it's started without any pre-check script bound
#
# 9944: default ws port for parachain node
# 30333: default p2p port for relaychain node
# 4545: default untrusted-http-port for tee-worker (see config.json)
for p in ${CollatorWSPort:-9944} ${CollatorPort:-30333} ${UntrustedHttpPort:-4545}; do
  if [ ! -z "$(netstat -nat | grep :$p)" ]; then
    echo "port $p is in use, quit now"
    exit 1
  fi
done

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR/parachain"
make launch-docker-paseo
