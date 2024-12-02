# syntax=docker/dockerfile:1

### Builder Stage - Without Gramine
##################################################
# rust:1.82 is a Debian GNU/Linux 12 (bookworm) image
FROM rust:1.82-bookworm AS plain-builder

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN cargo build --release


### Deploy image (for CI - No Gramine)
##################################################
FROM node:20-bookworm-slim AS executor-worker

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor
ENV EXECUT_DIR=/usr/local/bin
WORKDIR $EXECUT_DIR

RUN apt update && apt install -y ca-certificates
COPY --from=plain-builder $BUILD_DIR/target/release/executor-worker $EXECUT_DIR/executor-worker

RUN chmod +x $EXECUT_DIR/executor-worker
RUN ls -al $EXECUT_DIR
RUN ldd $EXECUT_DIR/executor-worker && $EXECUT_DIR/executor-worker --version
CMD ["$EXECUT_DIR/executor-worker"]



### Builder Stage - With Gramine (SGX Hardware)
##################################################
# rust:1.82 is a Debian GNU/Linux 12 (bookworm) image
FROM rust:1.82-bookworm AS sgx-builder

# It's easier to install gramine rather than to install cargo.
RUN curl -fsSLo /usr/share/keyrings/gramine-keyring.gpg https://packages.gramineproject.io/gramine-keyring.gpg
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/gramine-keyring.gpg] https://packages.gramineproject.io/ bookworm main" \
| tee /etc/apt/sources.list.d/gramine.list

RUN curl -fsSLo /usr/share/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu jammy main" \
| tee /etc/apt/sources.list.d/intel-sgx.list

RUN apt update && apt install -y gramine

# Build
ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor

COPY . $HOME
WORKDIR $BUILD_DIR

RUN make


### Release image (with SGX Hardware)
##################################################
FROM gramineproject/gramine:1.8-jammy AS omni-release
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ENV HOME=/home/ubuntu
ENV BUILD_DIR=$HOME/tee-worker/omni-executor
ENV EXECUT_DIR=/usr/local/bin
WORKDIR $EXECUT_DIR

RUN apt update && apt install -y ca-certificates && apt clean

COPY --from=sgx-builder $BUILD_DIR/omni-executor-worker $EXECUT_DIR/omni-executor-worker
COPY --from=sgx-builder $BUILD_DIR/omni-executor.manifest $EXECUT_DIR/omni-executor.manifest
COPY --from=sgx-builder $BUILD_DIR/omni-executor.manifest.sgx $EXECUT_DIR/omni-executor.manifest.sgx
COPY --from=sgx-builder $BUILD_DIR/omni-executor.sig $EXECUT_DIR/omni-executor.sig

RUN groupadd -g 1000 appgroup && \
    useradd -r -u 1000 -g appgroup appuser && \
    mkdir -p /dev/attestation && \
    chown -R appuser:appgroup /dev/attestation

RUN chmod +x $EXECUT_DIR/omni-executor-worker
RUN chown appuser:appgroup *
RUN ldd $EXECUT_DIR/omni-executor-worker && $EXECUT_DIR/omni-executor-worker --version
RUN ls -al $EXECUT_DIR

USER appuser
CMD ["$EXECUT_DIR/omni-executor-worker"]