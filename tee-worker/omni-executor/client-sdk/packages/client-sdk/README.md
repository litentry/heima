# @litentry/client-sdk

This package provides helpers for dApps to interact with the Litentry Protocol.

The Enclave is the Litentry's Trusted Execution Environment (TEE), that provides the hightest security and privacy for users to store their identity.

This is a browser package, it may not work as-is on Node.js due to Crypto Subtle and WebSocket differences, but the exposed RPC logic is the same.

## Installation

1. Install from NPM

    ```
    npm install @litentry/parachain-api @litentry/sidechain-api @litentry/client-sdk
    ```

2. Set the right environment

    Litentry's Protocol is currently available in three main stages: local (development), `tee-dev` (staging), and `tee-prod` (production).

    You can set what stage to use by setting the `LITENTRY_NETWORK` environment variable. Valid values are:

    - `litentry-local`: will point to a local enclave `ws://localhost:2000`
    - `litentry-dev` (default): will point to `tee-dev`'s Enclave.
    - `litentry-staging`: will point to `tee-staging`'s Enclave.
    - `litentry-prod`: will point to `tee-prod`'s Enclave.

    `NX_*` prefixed env variables (NX projects) will work too.

### Versions

This package is distributed under two main tags: `next` and `latest`.

Versions in the pattern of `x.x.x-next.x` feature the most recent code version to work with `tee-dev`. E.g., `1.0.0-next.0`. Once stable and once the Litentry Protocol is upgraded, the version will be tagged as `latest` and should be used against `tee-prod`. E.g., `1.0.0`. You can find all versions on https://www.npmjs.com/package/@litentry/enclave?activeTab=versions

## Examples & API documentation

Please refer to the `examples` folder in this repository to learn more about all the available operations. The `docs` folder includes detailed API information about.

## Development

### Quick start

These are the steps for publishing the package locally for development purposes.

1. Install dependencies

    ```
    pnpm install
    ```

2. Spin up an local NPM registry

    ```
    pnpm nx local-registry
    ```

3. Publish locally

    Follow the steps of [Publish new versions](#publish-new-versions). The step 1 can be skipped.

    As long as the local registry is up, any publishing will happen locally.

4. Run test and lint checks

    ```
    pnpm nx run client-sdk:lint

    pnpm nx run client-sdk:test
    ```

### Publish new versions

1. Bump the version on package.json to for instance `1.0.0`.

2. Update the latest documentation

    ```
    pnpm nx run client-sdk:generate-doc
    ```

3. Build the project

    ```
    pnpm nx run client-sdk:build
    ```

4. Publish the distribution files

    ```
    pnpm nx run client-sdk:publish --ver 1.0.0 --tag latest
    ```
