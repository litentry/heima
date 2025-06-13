# Use minimal base image with system deps
FROM ubuntu:24.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    llvm \
    clang \
    protobuf-compiler \
    npm \
    python3 \
    python3-pip

# Install Node.js LTS (for anchor tests)
RUN npm install -g n
RUN n lts
ENV PATH="/usr/local/n/versions/node/$(n --lts)/bin:$PATH"

# Install Yarn
RUN npm install -g yarn

# Install rustup & pin Rust version to 1.87.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.87.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Confirm Rust version
RUN rustc --version

# Install Solana CLI (latest stable from solana-install.solana.workers.dev)
RUN curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash

# Set Solana CLI path
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install avm & anchor (latest stable Anchor 0.31.1)
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked
RUN avm install 0.31.1 && avm use 0.31.1

# Set workdir inside container
WORKDIR /project

# Copy everything into container
COPY . .

# Build using anchor
# RUN export PATH="/root/.local/share/solana/install/active_release/bin:${PATH}" && anchor build
# RUN export PATH="/root/.local/share/solana/install/active_release/bin:${PATH}" && anchor test -- --features test-skip-auth


# Default command
CMD ["bash"]