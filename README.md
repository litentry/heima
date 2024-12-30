
<div align="center">

![](https://res.cloudinary.com/brandpad/image/upload/c_scale,dpr_auto,f_auto,w_768/v1673016042/19618/parachain-logo-color-black-t)

[![general ci](https://github.com/litentry/heima/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/litentry/heima/actions/workflows/ci.yml)
[![release](https://github.com/litentry/heima/actions/workflows/create-release-draft.yml/badge.svg)](https://github.com/litentry/heima/actions/workflows/create-release-draft.yml)
[![runtime upgrade](https://github.com/litentry/heima/actions/workflows/check-runtime-upgrade.yml/badge.svg)](https://github.com/litentry/heima/actions/workflows/check-runtime-upgrade.yml)

</div>

Heima is a substrate-based, EVM-compatible parachain that connects to the relaychain (e.g. [Polkadot](https://polkadot.com/)) which ensures shared security and interoperability. It serves as the backbone of Heima network:
- HEI token native features: transfer, governance, staking ...
- Runtime logic such as enclave management, DID ...
- parentchain of identity-worker, which is a TEE-based sidechain to achieve identity aggregation and crediential issuance without promising users' privacy 
- parentchain of bitacross-worker, which is a TEE-based offchain-worker to bridge assets across chains using native custodian and multisig

## Build parachain

To build the binary:

```
make build-node
```

To build the `litentry/heima` docker image, based on cargo profile:

```
make build-docker-release
make build-docker-production
```

To build the heima runtime wasm:

```
make build-runtime-heima
```

The wasms should be located under `target/release/wbuild/heima-parachain-runtime/`

Similarly, use `make build-runtime-paseo` to build the paseo-parachain-runtime.

## Launch parachain
### Launch a parachain network with relaychains

Heima uses [zombinet](https://github.com/paritytech/zombienet) to spin up a local network with 2 relaychain nodes and 1 parachain node:
```
make launch-network-heima
```
It will firstly look for the `target/release/heima-node` binary - and if not found - copy the binary out from `litentry/heima:latest` image if you are on Linux.

If you see the screenshot below, you can tell the network is successfully spun up and you can access the [polkadot-js block explorer](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944#/explorer) locally:

![](https://github.com/user-attachments/assets/125e43d7-b54f-41af-8de4-07bbe12b9702)

When finished with the network, run

```
make clean-network
```

to stop the processes and tidy things up.

### Launch a standalone parachain node

To speed up the development, the parachain can be launched without relaychain nodes.
In this case, parachain will author blocks by itself with instant block finalisation, please refer to [this PR](https://github.com/litentry/heima/pull/1059).

```
make launch-standalone
```

## How to build and run tee-worker

### Preparation

- Env: [Setup **SGX TEE** Environment](https://web3builders.notion.site/Setup-SGX-TEE-Environment-68066770831b45b7b632e682cf159477?pvs=4) 

### Build

```
cd /tee-worker
source /opt/intel/sgxsdk/environment
SGX_MODE=SW WORKER_DEV=1 make
```

### Launch

Before executing `launch.py`, the following Python libraries need to be installed
```
pip install python-dotenv pycurl docker toml
```

TEE-workers need a running parachain to become operational. We have an all-in-one script `local-setup/launch.py` to launch both parachain and workers:
```
./local-setup/launch.py -p standalone
./local-setup/launch.py -p network
./local-setup/launch.py -p remote
```

They stand for different parachain launching options:
- standalone parachain
- parachain network with relaychains
- parachain is remotely launched (elsewhere), so don't launch parachain in `launch.py`
respectively.

If you see the screenshot below, you can tell the worker is running:

![](https://github.com/cryptoade1/litentry-parachain/assets/88367184/cb1cea60-bc5d-4b62-bae7-503583a135ee)

### TEE worker tests 

Refer to [tee-worker ts-tests](https://github.com/litentry/heima/blob/dev/tee-worker/identity/ts-tests/README.md)

### Teardown

In the worker launch terminal, `Ctrl + C` should interrupt and clean everything up automatically.

### Additional Info:

1. Change the RUST_LOG level: `heima/local-setup/py/worker.py`
2. Check existing ts-tests: `heima/tee-worker/ts-tests/package.json`
3. JSON config parameters: `heima/tee-worker/service/src/cli.yml`