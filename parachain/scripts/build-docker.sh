#!/usr/bin/env bash
set -eo pipefail

function usage() {
  echo "Usage:   $0 release|production [docker-tag] [build-args]"
}

[ $# -lt 1 ] && (usage; exit 1)

ROOTDIR=$(git rev-parse --show-toplevel)
cd "$ROOTDIR"

PROFILE="$1"
TAG="$2"
ARGS="$3"

NOCACHE_FLAG=

case "$PROFILE" in
    release)
        ;;
    production)
        NOCACHE_FLAG="--no-cache" ;;
    *)
        usage; exit 1 ;;
esac

if [ -z "$TAG" ]; then
    TAG_COMMIT=`git rev-list --tags --max-count=1`
    HEAD_COMMIT=`git rev-parse HEAD`
    echo "TAG  commit: $TAG_COMMIT"
    echo "HEAD commit: $HEAD_COMMIT"

    if [ "$TAG_COMMIT" == "$HEAD_COMMIT" ]; then
        TAG=`git describe --tags $TAG_COMMIT`
    else
        TAG=`git rev-parse --short HEAD`
    fi
fi

echo "PROFILE: $PROFILE"
echo "TAG: $TAG"
echo "ARGS: $ARGS"

# ===================================================================================
GITUSER=litentry
GITREPO=heima
PROXY="${HTTP_PROXY//localhost/host.docker.internal}"

# Build the local-builder image
echo "------------------------------------------------------------"
echo "Building local-builder:latest docker image ..."
docker build ${NOCACHE_FLAG} --pull -f ./parachain/docker/Dockerfile \
    --build-arg PROFILE="$PROFILE" \
    --build-arg BUILD_ARGS="$ARGS" \
    --build-arg HTTP_PROXY="$PROXY" \
    --build-arg HTTPS_PROXY="$PROXY" \
    --build-arg http_proxy="$PROXY" \
    --build-arg https_proxy="$PROXY" \
    --add-host=host.docker.internal:host-gateway \
    --network host \
    --target builder \
    --tag local-builder:latest .

docker images

# Build the image
echo "------------------------------------------------------------"
echo "Building ${GITUSER}/${GITREPO}:${TAG} docker image ..."
docker build ${NOCACHE_FLAG} -f ./parachain/docker/Dockerfile \
    --build-arg PROFILE="$PROFILE" \
    --build-arg BUILD_ARGS="$ARGS" \
    --build-arg HTTP_PROXY="$PROXY" \
    --build-arg HTTPS_PROXY="$PROXY" \
    --build-arg http_proxy="$PROXY" \
    --build-arg https_proxy="$PROXY" \
    --add-host=host.docker.internal:host-gateway \
    --network host \
    --target parachain \
    --tag ${GITUSER}/${GITREPO}:${TAG} .

# Tag it with latest if no tag parameter was provided
[ -z "$2" ] && docker tag ${GITUSER}/${GITREPO}:${TAG} ${GITUSER}/${GITREPO}:latest

# Show the list of available images for this repo
echo "Image is ready"
echo "------------------------------------------------------------"
docker images | grep ${GITREPO}

# ===================================================================================
if [ -z "$2" ] || [ "$TAG" = "latest" ]; then
    GITUSER=litentry
    GITREPO=heima-aio
    PROXY="${HTTP_PROXY//localhost/host.docker.internal}"

    # Build the image
    echo "------------------------------------------------------------"
    echo "Building ${GITUSER}/${GITREPO}:${TAG} docker image ..."
    docker build ${NOCACHE_FLAG} -f ./parachain/docker/Dockerfile \
        --build-arg PROFILE="$PROFILE" \
        --build-arg BUILD_ARGS="$ARGS" \
        --build-arg HTTP_PROXY="$PROXY" \
        --build-arg HTTPS_PROXY="$PROXY" \
        --build-arg http_proxy="$PROXY" \
        --build-arg https_proxy="$PROXY" \
        --add-host=host.docker.internal:host-gateway \
        --network host \
        --target chain-aio \
        --tag ${GITUSER}/${GITREPO}:${TAG} .

    # Tag it with latest if no tag parameter was provided
    [ -z "$2" ] && docker tag ${GITUSER}/${GITREPO}:${TAG} ${GITUSER}/${GITREPO}:latest

    # Show the list of available images for this repo
    echo "Image is ready"
    echo "------------------------------------------------------------"
    docker images | grep ${GITREPO}
fi
