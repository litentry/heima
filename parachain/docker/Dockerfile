# global args that are used across multiple stages
ARG PROFILE

# ==========================
# stage 1: build
# https://hub.docker.com/_/rust
# our host is based on bullseye/sid (ubuntu 20)
# ==========================
FROM rust:bullseye as builder

WORKDIR /litentry/parachain
COPY . /litentry/

RUN apt-get update && apt-get install -yq clang libclang-dev cmake protobuf-compiler

ARG BUILD_ARGS
ARG PROFILE

RUN cargo build --locked --profile $PROFILE $BUILD_ARGS

# ==========================
# stage 2.1: packaging parachain image
# ==========================
FROM ubuntu:22.04 AS parachain
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ARG PROFILE

COPY --from=local-builder:latest /litentry/parachain/target/$PROFILE/heima-node /usr/local/bin/

# install netcat for healthcheck
RUN apt-get update && \
    apt-get install -yq netcat ca-certificates && \
    update-ca-certificates && \
    apt-get clean && \
    rm -rf /var/cache/apt/lists && \
    cp /usr/bin/nc /usr/local/bin/

RUN useradd -m -u 1000 -U -s /bin/sh -d /litentry litentry && \
    mkdir -p /data /litentry/.local/share && \
    chown -R litentry:litentry /data && \
    ln -s /data /litentry/.local/share/heima-node && \
    # check if executable works in this container
    /usr/local/bin/heima-node --version

USER litentry
EXPOSE 30333 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/heima-node"]
CMD ["--help"]

# ==========================
# stage 2.2: packaging aio image
# ==========================

FROM ubuntu:22.04 AS chain-aio
LABEL maintainer="Trust Computing GmbH <info@litentry.com>"

ARG PROFILE
ENV NVM_DIR=/opt/nvm
ENV ZOMBIENET_VERSION=v1.3.109

# install netcat for healthcheck
RUN apt-get update && \
    apt install -yq build-essential ocaml ocamlbuild automake autoconf libtool wget \
    python-is-python3 libssl-dev git cmake perl python3 dkms pip python3-click clang-13 \
    lldb-13 lld-13 clangd net-tools libssl-dev libcurl4-openssl-dev protobuf-compiler \
    libprotobuf-dev debhelper cmake reprepro unzip pkgconf libboost-dev libboost-system-dev \
    libboost-thread-dev lsb-release libsystemd0 pkgconf jq python3-pip curl && \
    apt-get install -yq ca-certificates && \
    update-ca-certificates && \
    curl -fsSL https://deb.nodesource.com/setup_21.x | sudo -E bash - && \
    apt-get update && apt-get install -y nodejs && \
    npm install -g pnpm && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-key del 23E7166788B63E1E && \
    curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && \
    apt update && apt install -y yarn  && \
    mkdir -p $NVM_DIR && \
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash && \
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && \
    [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion" && \
    nvm install 18 && \
    nvm use 18 && \
    apt-get clean && \
    rm -rf /var/cache/apt/lists

RUN echo "downloading zombienet-linux-x64 ..." && \
    curl -L -s -O "https://github.com/paritytech/zombienet/releases/download/$ZOMBIENET_VERSION/zombienet-linux-x64" && \
    chmod +x zombienet-linux-x64 && \
    cp zombienet-linux-x64 /usr/local/bin/zombienet && \
    zombienet version

RUN useradd -m -u 1000 -U -s /bin/sh -d /litentry litentry && \
	mkdir -p /opt/litentry/parachain /code && \
	chown -R litentry:litentry /opt/litentry

COPY --from=local-builder:latest /litentry/parachain/target/$PROFILE/heima-node /usr/local/bin/
RUN chmod +x /usr/local/bin/heima-node && \
    # check if executable works in this container
    /usr/local/bin/heima-node --version

COPY ./parachain /code/parachain
COPY ./parachain/docker/entrypoint.sh /usr/local/bin/
COPY ./common /code/common
RUN chown -R litentry:litentry /code
RUN chmod +x /usr/local/bin/entrypoint.sh

USER litentry

EXPOSE 9615 9944 9946 9947 30332 30333 30336 30337
VOLUME ["/opt/litentry"]

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]