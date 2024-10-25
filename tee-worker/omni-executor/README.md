# Omni-executor worker

! Connect to trusted RPC endpoints ! 

## Running inside TEE

Gramine is required for running inside TEE, please refer to [installation options](https://gramine.readthedocs.io/en/stable/installation.html).

1. `make SGX=1` to build and sign application
2. `RUST_LOG=info gramine-sgx omni-executor -- <parentchain-rpc-url> <ethereum-rpc-url>`


## Running whole setup locally

Build omni-executor docker image first `make build-docker`.
Start local environment using `docker-compose up` command.