<div align="center">

<img src="https://github.com/user-attachments/assets/63bf12cf-f6cd-4021-8806-80405399c7cb" alt="Heima" width="420" />

[![general ci](https://github.com/litentry/heima/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/litentry/heima/actions/workflows/ci.yml)

</div>

Evolved from Litentry, **Heima Network** is a substrate-based, EVM-compatible L1 that connects to a Polkadot-class relay chain for shared security and interoperability. It is built for chain abstraction and cross-chain operations, with:

- **HEI token** — the native token: transfers, governance, staking.
- **Runtime & pallets** — omni-account (cross-chain account abstraction), parachain-staking, omni-bridge, TEE worker registration (teebag), extrinsic filtering, and full EVM support.
- **omni-executor** — a TEE (Gramine/SGX) worker that executes cross-chain UserOperations (EIP-4337 account abstraction) across Ethereum, Solana and other chains.

### Repository layout

- [`parachain/`](parachain/) — the node, runtimes (`heima`, `paseo`) and pallets.
- [`tee-worker/omni-executor/`](tee-worker/omni-executor/) — the cross-chain TEE worker.
- [`local-setup/`](local-setup/) — helper scripts for local development.

All commands below are run from the repository root unless noted otherwise.

## Parachain

The parachain ships two runtimes: `heima-runtime` (paraID 2013) and `paseo-runtime` (paraID 2106, for the [Paseo](https://github.com/paseo-network) testnet), both produced by the `heima-node` binary.

### Build

```sh
make build-node                 # build the heima-node binary
make build-runtime-heima        # build the heima runtime wasm
make build-runtime-paseo        # build the paseo runtime wasm
```

Runtime wasms land under `parachain/target/release/wbuild/<runtime>/`.

To build the `litentry/heima` docker image (by cargo profile):

```sh
make build-docker-release       # `release` profile
make build-docker-production    # `production` profile
```

### Launch

Spin up a local network (2 relaychain validators + 1 collator) with [zombienet](https://github.com/paritytech/zombienet):

```sh
make launch-network-heima       # or: make launch-network-paseo
```

Once it is up, the chain is reachable in the [polkadot-js explorer](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944#/explorer) at `ws://127.0.0.1:9944`. Tear it down with:

```sh
make clean-network
```

For faster iteration, run a single standalone node (no relaychain, instant block finality):

```sh
make launch-standalone
```

### Test

```sh
make test-cargo-all             # cargo tests across the workspace
make test-ts-heima              # TypeScript integration tests (or: test-ts-paseo)
```

The TypeScript suite lives in [`parachain/ts-tests/`](parachain/ts-tests/).

## omni-executor

`omni-executor` is a Rust TEE worker (running under Gramine/SGX) that executes cross-chain UserOperations — EIP-4337 account abstraction — across Ethereum, Solana and other chains, exposing a JSON-RPC server for authenticated submissions.

It vendors its Solidity dependencies as git submodules, so initialize them first:

```sh
git submodule update --init --recursive
```

### Build

```sh
cd tee-worker/omni-executor
cargo build --release           # host build, for local development
make SGX=1                      # build & sign for Gramine/SGX
```

### Run locally

A docker-compose setup brings up the worker together with a local Anvil EVM node and auto-deployed account-abstraction contracts. From `tee-worker/omni-executor/`:

```sh
make build-docker               # or: make build-docker-test (with test endpoints)
make start-local                # or: make start-test
make stop-local                 # tear down (or: make stop-test)
```

### Test

TypeScript integration tests live in [`tee-worker/omni-executor/ts-tests/`](tee-worker/omni-executor/ts-tests/).

For full details — environment variables, RPC endpoints, and CLI usage — see [`tee-worker/omni-executor/README.md`](tee-worker/omni-executor/README.md) and [`tee-worker/omni-executor/LOCAL_DEV.md`](tee-worker/omni-executor/LOCAL_DEV.md).
