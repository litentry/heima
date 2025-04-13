# syntax=docker/dockerfile:1

### Builder Stage - Without Gramine
##################################################
# rust:1.82 is a Debian GNU/Linux 12 (bookworm) image
FROM rust:1.82-bookworm AS plain-builder

RUN apt-get update && apt-get install -y clang


ENV SRC_DIR=/home/ubuntu/tee-worker/omni-executor
COPY tee-worker/omni-executor $SRC_DIR
COPY common /home/ubuntu/common
WORKDIR $SRC_DIR

RUN cargo build --release


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
# rust:1.82 is a Debian GNU/Linux 12 (bookworm) image
FROM gramineproject/gramine:stable-jammy AS sgx-builder

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl \
    libssl-dev \
    pkg-config

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN apt-get install -y clang

# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version

# Build
ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN make


### Release image (with SGX Hardware)
##################################################
FROM gramineproject/gramine:stable-jammy AS omni-release
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor
ENV BIN_DIR=/usr/local/bin
WORKDIR $BIN_DIR

RUN apt update && apt install -y ca-certificates && apt clean

COPY --from=sgx-builder $BUILD_DIR/omni-executor $BIN_DIR/omni-executor
COPY --from=sgx-builder $BUILD_DIR/omni-executor.manifest $BIN_DIR/omni-executor.manifest
COPY --from=sgx-builder $BUILD_DIR/omni-executor.manifest.sgx $BIN_DIR/omni-executor.manifest.sgx
COPY --from=sgx-builder $BUILD_DIR/omni-executor.sig $BIN_DIR/omni-executor.sig

RUN groupadd -g 1000 ubuntu && \
    useradd -r -u 1000 -g ubuntu ubuntu && \
    mkdir -p /dev/attestation && \
    chown -R ubuntu:ubuntu /dev/attestation

RUN chmod +x $BIN_DIR/omni-executor
RUN chown ubuntu:ubuntu *
RUN ldd $BIN_DIR/omni-executor && $BIN_DIR/omni-executor --version
RUN ls -al $BIN_DIR

USER ubuntu
CMD ["$BIN_DIR/omni-executor"]