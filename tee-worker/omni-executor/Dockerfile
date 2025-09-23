# syntax=docker/dockerfile:1

### Builder Stage - Without Gramine
##################################################
FROM rust:1.87-bookworm AS plain-builder

ARG CARGO_FEATURES=""

RUN apt-get update && apt-get install -y clang

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN if [ -n "$CARGO_FEATURES" ]; then \
        cargo build --release --features "$CARGO_FEATURES"; \
    else \
        cargo build --release; \
    fi


### Deploy image (for CI - No Gramine)
##################################################
FROM node:20-bookworm-slim AS executor-worker

ENV SRC_DIR=/home/ubuntu/tee-worker/omni-executor
ENV BIN_DIR=/usr/local/bin
WORKDIR $BIN_DIR

RUN apt update && apt install -y ca-certificates curl
RUN corepack enable && corepack prepare pnpm@8.7.6 --activate && corepack enable pnpm

COPY --from=plain-builder $SRC_DIR/target/release/executor-worker $BIN_DIR/executor-worker
COPY --from=plain-builder $SRC_DIR/local/keystore $BIN_DIR/local/keystore

RUN chmod +x $BIN_DIR/executor-worker
RUN ls -al $BIN_DIR
RUN ldd $BIN_DIR/executor-worker && $BIN_DIR/executor-worker --version
CMD ["$BIN_DIR/executor-worker"]


### Builder Stage - With Gramine (SGX Hardware)
##################################################
FROM gramineproject/gramine:1.9-jammy AS sgx-builder

ARG OE_WILDMETA_BACKEND_ECDSA_PUBKEY="020000000000000000000000000000000000000000000000000000000000000000"
ENV OE_WILDMETA_BACKEND_ECDSA_PUBKEY=$OE_WILDMETA_BACKEND_ECDSA_PUBKEY

RUN apt-get update

RUN apt-get install -y \
    build-essential \
    curl \
    libssl-dev \
    clang \
    pkg-config \
    ca-certificates

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN make SGX=1 SGX_DEBUG=0 SOURCE_DATE_EPOCH=1700000000 CARGO_PROFILE_RELEASE_DEBUG=false


### Release image (with SGX Hardware)
##################################################
FROM gramineproject/gramine:1.9-jammy AS release
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor
ENV BIN_DIR=/wkdir
WORKDIR $BIN_DIR

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl \
    libssl-dev \
    clang \
    pkg-config \
    ca-certificates

RUN mkdir -p /wkdir /wkdir/local /wkdir/storage_db

COPY --from=sgx-builder $BUILD_DIR/omni-executor $BIN_DIR/omni-executor
COPY --from=sgx-builder $BUILD_DIR/omni-executor.manifest.sgx $BIN_DIR/omni-executor.manifest.sgx
COPY --from=sgx-builder $BUILD_DIR/omni-executor.sig $BIN_DIR/omni-executor.sig

RUN groupadd -g 1000 ubuntu && \
    useradd -r -u 1000 -g ubuntu ubuntu && \
    mkdir -p /dev/attestation && \
    chown -R ubuntu:ubuntu /dev/attestation /wkdir

RUN chmod +x $BIN_DIR/omni-executor && \
    chown ubuntu:ubuntu * && \
    ldd $BIN_DIR/omni-executor && $BIN_DIR/omni-executor --version && \
    ls -al $BIN_DIR

USER ubuntu

ENTRYPOINT [ "/usr/bin/gramine-sgx" ]

CMD ["$BIN_DIR/omni-executor"]