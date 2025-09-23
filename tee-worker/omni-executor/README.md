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
   # for local (production build - excludes test endpoints)
   make build-docker
   # for integration test (includes test endpoints)
   make build-docker-test
   ```

   **Note:** The `test-endpoints` feature flag controls test-only RPC methods like `omni_submitUserOpTest`. 
   Production builds exclude these by default. Use `cargo build --release --features test-endpoints` to include them.

3. Start omni-executor:
   ```bash
   # for local
   make start-local
   # for integration test
   make start-test
   ```

4. Stop omni-executor:
   ```bash
   # for local
   make stop-local
   # for integration test
   make stop-test
   ```


First service run will generate substrate account, it needs to set as omni executor in `omniAccount` pallet. 
