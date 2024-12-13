# syntax=docker/dockerfile:1
# Copyright 2021 Integritee AG
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#           http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This is a multi-stage docker file, where the first stage is used
# for building and the second deploys the built application.

### Builder Stage
##################################################
FROM litentry/litentry-tee-dev:latest AS builder
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

# set environment variables
ENV SGX_SDK=/opt/sgxsdk
ENV PATH="$PATH:${SGX_SDK}/bin:${SGX_SDK}/bin/x64:/opt/rust/bin"
ENV PKG_CONFIG_PATH="${PKG_CONFIG_PATH}:${SGX_SDK}/pkgconfig"
ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:${SGX_SDK}/sdk_libs"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

ENV SCCACHE_CACHE_SIZE="20G"
ENV SCCACHE_DIR="/opt/rust/sccache"
ENV RUSTC_WRAPPER="/opt/rust/bin/sccache"

# Default SGX MODE is software mode
ARG SGX_MODE=SW
ENV SGX_MODE=$SGX_MODE

ARG SGX_PRODUCTION=0
ENV SGX_PRODUCTION=$SGX_PRODUCTION

ENV HOME=/home/ubuntu

ARG WORKER_MODE_ARG
ENV WORKER_MODE=$WORKER_MODE_ARG

ARG WORKER_ENV_DATA_PROVIDERS_CONFIG
ENV WORKER_ENV_DATA_PROVIDERS_CONFIG=$WORKER_ENV_DATA_PROVIDERS_CONFIG

ARG WORKER_MOCK_SERVER
ENV WORKER_MOCK_SERVER=$WORKER_MOCK_SERVER

ARG ADDITIONAL_FEATURES_ARG
ENV ADDITIONAL_FEATURES=$ADDITIONAL_FEATURES_ARG

ARG IMAGE_FOR_RELEASE=false
ENV IMAGE_FOR_RELEASE=$IMAGE_FOR_RELEASE

ARG FINGERPRINT=none

ARG SGX_COMMERCIAL_KEY
ENV SGX_COMMERCIAL_KEY=$SGX_COMMERCIAL_KEY

WORKDIR $HOME/tee-worker/identity
COPY . $HOME

RUN \
  if [ "$IMAGE_FOR_RELEASE" = "true" ]; then \
    echo "Omit cache for release image"; \
    unset RUSTC_WRAPPER; \
    make; \
  else \
    rm -rf /opt/rust/registry/cache && mv /home/ubuntu/worker-cache/registry/cache /opt/rust/registry && \
    rm -rf /opt/rust/registry/index && mv /home/ubuntu/worker-cache/registry/index /opt/rust/registry && \
    rm -rf /opt/rust/git/db && mv /home/ubuntu/worker-cache/git/db /opt/rust/git && \
    rm -rf /opt/rust/sccache && mv /home/ubuntu/worker-cache/sccache /opt/rust && \
    make && sccache --show-stats; \
  fi

RUN make mrenclave 2>&1 | grep MRENCLAVE | awk '{print $2}' > mrenclave.txt
RUN cargo test --release


### Base Runner Stage
##################################################
FROM node:18-bookworm-slim AS runner

RUN apt update && apt install -y libssl-dev iproute2 jq curl protobuf-compiler python3 python-is-python3 build-essential
RUN corepack enable && corepack prepare pnpm@8.7.6 --activate && corepack enable pnpm


### Deployed CLI client
##################################################
FROM runner AS deployed-client
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ARG SCRIPT_DIR=/usr/local/worker-cli
ARG LOG_DIR=/usr/local/log

ENV SCRIPT_DIR=${SCRIPT_DIR}
ENV LOG_DIR=${LOG_DIR}

COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/bin/litentry-cli /usr/local/bin
COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/cli/*.sh /usr/local/worker-cli/

RUN chmod +x /usr/local/bin/litentry-cli ${SCRIPT_DIR}/*.sh
RUN mkdir ${LOG_DIR}

RUN ldd /usr/local/bin/litentry-cli && /usr/local/bin/litentry-cli --version

ENTRYPOINT ["/usr/local/bin/litentry-cli"]


### Deployed worker service
##################################################
FROM runner AS deployed-worker
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

WORKDIR /usr/local/bin

COPY --from=local-builder:latest /opt/sgxsdk /opt/sgxsdk
COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/bin/* /usr/local/bin
COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/cli/*.sh /usr/local/worker-cli/
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libsgx* /lib/x86_64-linux-gnu/
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libdcap* /lib/x86_64-linux-gnu/
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libprotobuf* /lib/x86_64-linux-gnu/

RUN touch spid.txt key.txt
RUN chmod +x /usr/local/bin/litentry-worker
RUN ls -al /usr/local/bin

# checks
ENV SGX_SDK=/opt/sgxsdk
ENV SGX_ENCLAVE_SIGNER=$SGX_SDK/bin/x64/sgx_sign
ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/opt/intel/sgx-aesm-service/aesm:$SGX_SDK/sdk_libs
ENV AESM_PATH=/opt/intel/sgx-aesm-service/aesm

RUN ldd /usr/local/bin/litentry-worker && /usr/local/bin/litentry-worker --version

# TODO: use entrypoint and aesm service launch, see P-295 too
ENTRYPOINT ["/usr/local/bin/litentry-worker"]


### Release worker image
##################################################
FROM ubuntu:22.04 AS worker-release
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

RUN apt update && apt install -y libssl-dev iproute2 curl protobuf-compiler

# Adding default user litentry with uid 1000
ARG UID=1000
RUN adduser -u ${UID} --disabled-password --gecos '' litentry
RUN adduser -u ${UID} litentry sudo
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
# to fix Multi-node distributed worker encounters SGX permission errors.
RUN groupadd -g 121 sgx_prv && \
    groupadd -g 108 sgx && \
    usermod -aG sgx litentry && \
    usermod -aG sgx_prv litentry

COPY --from=local-builder:latest /opt/sgxsdk /opt/sgxsdk
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libsgx* /lib/x86_64-linux-gnu/
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libdcap* /lib/x86_64-linux-gnu/
COPY --from=local-builder:latest /lib/x86_64-linux-gnu/libprotobuf* /lib/x86_64-linux-gnu/

ENV DEBIAN_FRONTEND=noninteractive
ENV TERM=xterm
ENV SGX_SDK=/opt/sgxsdk
ENV PATH="$PATH:${SGX_SDK}/bin:${SGX_SDK}/bin/x64:/opt/rust/bin"
ENV PKG_CONFIG_PATH="${PKG_CONFIG_PATH}:${SGX_SDK}/pkgconfig"
ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:${SGX_SDK}/sdk_libs"

RUN mkdir -p /origin /data

COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/bin/* /origin
COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/mrenclave.txt /origin
COPY --from=local-builder:latest /home/ubuntu/tee-worker/identity/entrypoint.sh /usr/local/bin/entrypoint.sh

WORKDIR /origin

RUN touch spid.txt key.txt && \
    cp ./litentry-* /usr/local/bin/ && \
    chmod +x /usr/local/bin/litentry-* && \
    chmod +x /usr/local/bin/entrypoint.sh && \
    ls -al /usr/local/bin

RUN ldd /usr/local/bin/litentry-worker && /usr/local/bin/litentry-worker --version

ENV DATA_DIR=/data

USER litentry
WORKDIR /data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]