# @heima-network/client-sdk

This package provides helpers for dApps to interact with the Heima Protocol.

The Enclave is the Heima's Trusted Execution Environment (TEE), that provides the hightest security and privacy for users to store their identity.

This is a browser package, it may not work as-is on Node.js due to Crypto Subtle and WebSocket differences, but the exposed RPC logic is the same.

## Installation

1. Install from NPM

    ```bash
    npm install @heima-network/parachain-api @heima-network/sidechain-api @heima-network/client-sdk
    ```
   
   If you are using this sdk in react-native, you need to install the following packages:

    ```bash
    npm install react-native-quick-crypto
    ```

    Then using below clode to install it in your `index.js` file:

    ```js
    import { install } from 'react-native-quick-crypto';

    install();
    ```

2. Set the right environment

    Heima's Protocol is currently available in three main stages: local (development), `tee-dev` (staging), and `tee-prod` (production).

    You can set what stage to use by setting the `HEIMA_NETWORK` environment variable. Valid values are:

    - `heima-local`: will point to a local enclave `ws://localhost:2100`
    - `heima-dev` (default): will point to `tee-dev`'s Enclave.
    - `heima-prod`: will point to `tee-prod`'s Enclave.
    - `ws://<your-enclave>` or `wss://<your-enclave>`: will point to a custom Enclave.

    `NX_*` prefixed env variables (NX projects) will work too.

    For non-nodejs environments (browser or React Native), you can use the Babel plugin to inject environment variables:

    1) Install the Babel plugin:

        ```bash
        npm install -D babel-plugin-transform-inline-environment-variables
        ```

    2) Configure `babel.config.js`:

        ```js
        module.exports = {
            plugins: [
                [
                'transform-inline-environment-variables',
                {
                    include: ['HEIMA_NETWORK'],
                },
                ],
            ],
        };
        ```
    Then after you set `HEIMA_NETWORK=some-value` in your env, all `process.env.HEIMA_NETWORK` will be statically replaced with `some-value` during build.

### Versions

This package is distributed under two main tags: `next` and `latest`.

Versions in the pattern of `x.x.x-next.x` feature the most recent code version to work with `tee-dev`. E.g., `1.0.0-next.0`. Once stable and once the Heima Protocol is upgraded, the version will be tagged as `latest` and should be used against `tee-prod`. E.g., `1.0.0`. You can find all versions on https://www.npmjs.com/package/@heima-network/client-sdk?activeTab=versions

## Examples & API documentation

Please refer to the `examples` folder in this repository to learn more about all the available operations. The `docs` folder includes detailed API information about.

## Development

### Quick start

These are the steps for publishing the package locally for development purposes.

1. Install dependencies

    ```bash
    pnpm install
    ```

2. Spin up an local NPM registry

    ```bash
    pnpm nx local-registry
    ```

3. Publish locally

    Follow the steps of [Publish new versions](#publish-new-versions). The step 1 can be skipped.

    As long as the local registry is up, any publishing will happen locally.

4. Run test and lint checks

    ```bash
    pnpm nx run client-sdk:lint

    pnpm nx run client-sdk:test
    ```

    > Before running the tests, make sure you have a local omni-executor running, see [omni-executor README](https://github.com/litentry/heima/tree/dev/tee-worker/omni-executor/README.md) for more information.

### Publish new versions

1. Bump the version on package.json to for instance `1.0.0`.

2. Update the latest documentation

    ```bash
    pnpm nx run client-sdk:generate-doc
    ```

3. Build the project

    ```bash
    pnpm nx run client-sdk:build
    ```

4. Publish the distribution files

    ```
    pnpm nx run client-sdk:publish --tag latest
    ```
