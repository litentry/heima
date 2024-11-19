# syntax=docker/dockerfile:1

### Builder Stage
##################################################
# rust:1.82 is a Debian GNU/Linux 12 (bookworm) image
FROM rust:1.82-bookworm AS builder

# It's easier to install gramine rather than to install cargo.
RUN curl -fsSLo /usr/share/keyrings/gramine-keyring.gpg https://packages.gramineproject.io/gramine-keyring.gpg
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gramine-keyring.gpg] https://packages.gramineproject.io/ bookworm main" \
| tee /etc/apt/sources.list.d/gramine.list

RUN curl -fsSLo /usr/share/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu jammy main" \
| tee /etc/apt/sources.list.d/intel-sgx.list

RUN apt update && apt install -y gramine

# Build
ARG SGX_MODE=SW
ENV SGX_MODE=$SGX_MODE

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN make


### Base Runner Stage
##################################################
FROM node:20-bookworm-slim AS runner

# Install required dependencies.
# TODO: Adapt later based on CI tests requirement
RUN apt update && apt install -y libssl-dev iproute2 jq curl protobuf-compiler python3 python-is-python3 build-essential
RUN corepack enable && corepack prepare pnpm@8.7.6 --activate && corepack enable pnpm

### Deploy image (for CI - No SGX Hardware)
##################################################
FROM runner AS executor-worker

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor
WORKDIR $BUILD_DIR
ENV EXECUT_DIR=/usr/local/bin

#TODO: Copy over any test related setup here

# It's easier to install gramine rather than to install cargo.
RUN curl -fsSLo /usr/share/keyrings/gramine-keyring.gpg https://packages.gramineproject.io/gramine-keyring.gpg
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gramine-keyring.gpg] https://packages.gramineproject.io/ bookworm main" \
| tee /etc/apt/sources.list.d/gramine.list

RUN curl -fsSLo /usr/share/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu jammy main" \
| tee /etc/apt/sources.list.d/intel-sgx.list

RUN apt update && apt install -y gramine ca-certificates

COPY --from=local-builder:latest $BUILD_DIR/omni-executor.manifest $EXECUT_DIR/omni-executor.manifest
COPY --from=local-builder:latest $BUILD_DIR/target/release/executor-worker $EXECUT_DIR/executor-worker

RUN chmod +x $EXECUT_DIR/executor-worker
RUN ls -al $EXECUT_DIR
RUN ldd $EXECUT_DIR/executor-worker && $EXECUT_DIR/executor-worker --version
CMD ["$EXECUT_DIR/executor-worker"]


### Release image (with SGX Hardware)
##################################################
# FROM ubuntu:22.04 AS omni-release
# LABEL maintainer="Trust Computing GmbH <info@litentry.com>"
# COPY --from=local-builder:latest /home/ubuntu/tee-worker/omni-executor/target/release/executor-worker /usr/local/bin/executor-worker

# RUN \
#   apt-get update && \
#   apt-get install -y ca-certificates && \
#   apt-get clean

# CMD ["executor-worker"]