#!/bin/bash

# this script builds the release artefacts for TEE client and/or the enclave

set -euo pipefail

function usage() {
  echo "Usage: $0 if-build-worker if-build-enclave"
  echo "Example:"
  echo "       $0 true true"
}

[ $# -ne 2 ] && (usage; exit 1)

echo "build worker: $1"
echo "build enclave: $2"

ROOTDIR=$(git rev-parse --show-toplevel)
WORKERDIR="$ROOTDIR/tee-worker/identity"

# hardcoded sgx signing key, adjust it accordingly if you call the script manually
SGX_COMMERCIAL_KEY="/opt/enclave_release/sgx_sign_key.pem"

if [ ! -f "$SGX_COMMERCIAL_KEY" ]; then
    echo "Cannot find SGX sign key under $SGX_COMMERCIAL_KEY"
    exit 1
fi

DESTDIR="$WORKERDIR/enclave_release"
[ -d "$DESTDIR" ] && rm -rf "$DESTDIR"
mkdir -p "$DESTDIR"

cd "$WORKERDIR"

make clean

export SGX_PRODUCTION=1
export SGX_COMMERCIAL_KEY="$SGX_COMMERCIAL_KEY"
if [ "$1" = "true" ]; then
    make service
    cp bin/litentry-worker "$DESTDIR"
fi
if [ "$2" = "true" ]; then
    make bin/enclave.signed.so
    cp bin/enclave.signed.so "$DESTDIR"
    make mrenclave 2>&1 | grep MRENCLAVE | awk '{print $2}' > "$DESTDIR/mrenclave.txt"
fi

echo "Build tee done"
ls -l "$DESTDIR"
