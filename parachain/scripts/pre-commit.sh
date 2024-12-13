#!/usr/bin/env bash
# It is recommended to execute the script before commit
# which will help us to reduce test/fmt/clippy failures in CI

set -eo pipefail

function worker_clippy() {
    cargo clippy --release -- -D warnings
    cargo clippy --release --features evm -- -D warnings
    cargo clippy --release --features sidechain -- -D warnings
    cargo clippy --release --features offchain-worker -- -D warnings
    cargo clippy --release --features development -- -D warnings
    cargo clippy --release --features evm,development -- -D warnings
    cargo clippy --release --features sidechain,development -- -D warnings
    cargo clippy --release --features offchain-worker,development -- -D warnings
}

function bitacross_clippy() {
    cargo clippy --release -- -D warnings
    cargo clippy --release --features offchain-worker -- -D warnings
}

function parachain_check() {
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --locked --release -p pallet-* --lib
    cargo test --locked --release -p pallet-* --lib --features=runtime-benchmarks
    cargo test --locked --release -p paseo-parachain-runtime --lib
    cargo test --locked --release -p litentry-parachain-runtime --lib
}

function clean_up() {
    cd "$root_dir/parachain"
    cargo clean
    cd "$root_dir/tee-worker/identity"
    cargo clean
    cd "$root_dir/tee-worker/identity/enclave-runtime"
    cargo clean
    cd "$root_dir/tee-worker/bitacross"
    cargo clean
    cd "$root_dir/tee-worker/bitacross/enclave-runtime"
    cargo clean
}

root_dir=$(git rev-parse --show-toplevel)

start=$(date +%s)

clean_up

cd "$root_dir"
make fmt

echo "[Step 1], Parachain clippy"
cd "$root_dir/parachain" && parachain_check

echo "[Step 2], identity-worker clippy"
cd "$root_dir/tee-worker/identity" && worker_clippy

echo "[Step 3], identity-worker enclave clippy"
cd "$root_dir/tee-worker/identity/enclave-runtime" && worker_clippy

echo "[Step 4], identity-worker cargo test"
cd "$root_dir/tee-worker/identity"
RUST_LOG=info SKIP_WASM_BUILD=1 cargo test --release --features development -- --show-output

echo "[Step 5], identity-worker service test"
clean_up
cd "$root_dir/tee-worker/identity"
SGX_MODE=SW SKIP_WASM_BUILD=1 make
cd "$root_dir/tee-worker/identity/bin"
./litentry-worker test --all

echo "[Step 6], bitacross-worker clippy"
cd "$root_dir/tee-worker/bitacross" && bitacross_clippy

echo "[Step 7], bitacross-worker enclave clippy"
cd "$root_dir/tee-worker/bitacross/enclave-runtime" && bitacross_clippy

echo "[Step 8], bitacross-worker cargo test"
cd "$root_dir/tee-worker/bitacross"
RUST_LOG=info SKIP_WASM_BUILD=1 cargo test --release -- --show-output

echo "[Step 9], bitacross-worker service test"
clean_up
cd "$root_dir/tee-worker/bitacross"
SGX_MODE=SW SKIP_WASM_BUILD=1 make
cd "$root_dir/tee-worker/bitacross/bin"
./bitacross-worker test --all

end=$(date +%s)
echo "Elapsed Time: $((end-start)) seconds"
