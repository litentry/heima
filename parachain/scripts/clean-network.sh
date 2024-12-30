#!/usr/bin/env bash

# no `set -e` here as we allow commands to fail in this script

HEIMA_DIR=${HEIMA_DIR:-"/tmp/parachain_dev"}

case $(uname -s) in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) echo "Unsupported os"; exit 1 ;;
esac

case $(uname -m) in
  aarch64*) arch=arm64 ;;
  x86_64) arch=x64 ;;
  *) echo "Unsuppported arch"; exit 1 ;;
esac

ZOMBIENET_BIN=zombienet-${os}-${arch}
ZOMBIENET_PID=$(pidof $ZOMBIENET_BIN)

if [ -z $ZOMBIENET_PID ]; then
  # the network might not be started with zombienet
  killall polkadot || true
  killall heima-node || true
else
  kill -2 $ZOMBIENET_PID
fi

docker ps -q -f name=geth | xargs -r docker stop 

rm -rf "$HEIMA_DIR"

echo "cleaned up"