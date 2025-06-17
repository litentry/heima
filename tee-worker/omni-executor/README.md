# Omni-executor worker

! Connect to trusted RPC endpoints ! 

## Running inside TEE

Gramine is required for running inside TEE, please refer to [installation options](https://gramine.readthedocs.io/en/stable/installation.html).

1. `make SGX=1` to build and sign application
2. `RUST_LOG=info gramine-sgx omni-executor -- <parentchain-rpc-url> <ethereum-rpc-url>`


## Running whole setup locally

1. (Optional) Configure environment variables:

   ```bash
   cp .env.example .env
   # Edit .env file to configure required environment variables
   ```

2. Build omni-executor docker image:

   ```bash
   make build-docker
   ```

3. Start local omni-executor:
   ```bash
   make start-local
   ```

4. Start local omni-executor for integration test:
   ```bash
   make start-test
   ```

First service run will generate substrate account, it needs to set as omni executor in `omniAccount` pallet. 
