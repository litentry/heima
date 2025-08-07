# global args that are used across multiple stages
ARG PROFILE

# ==========================
# stage 1: build
# https://hub.docker.com/_/rust
# ==========================
FROM rust:bullseye as builder

WORKDIR /heima/parachain
COPY . /heima/

RUN apt-get update && apt-get install -yq clang libclang-dev cmake protobuf-compiler

ARG BUILD_ARGS
ARG PROFILE

RUN rustup toolchain install
RUN cargo build --locked --profile $PROFILE $BUILD_ARGS

# ==========================
# stage 2: packaging parachain image
# ==========================
FROM ubuntu:22.04 AS parachain
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ARG PROFILE

COPY --from=local-builder:latest /heima/parachain/target/$PROFILE/heima-node /usr/local/bin/

# install netcat for healthcheck
RUN apt-get update && \
    apt-get install -yq netcat ca-certificates && \
    update-ca-certificates && \
    apt-get clean && \
    rm -rf /var/cache/apt/lists && \
    cp /usr/bin/nc /usr/local/bin/

RUN useradd -m -u 1000 -U -s /bin/sh -d /heima heima && \
    mkdir -p /data /heima/.local/share && \
    chown -R heima:heima /data && \
    ln -s /data /heima/.local/share/heima-node && \
    # check if executable works in this container
    /usr/local/bin/heima-node --version

USER heima
EXPOSE 30333 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/heima-node"]
CMD ["--help"]