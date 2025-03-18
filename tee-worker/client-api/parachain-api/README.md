# Parachain-api

This library contains the Litentry Network API types and types definitions.

These types were auto generated using [Polkadot.js Type Generation](https://polkadot.js.org/docs/api/examples/promise/typegen/)

## How to use it

1. Install the package from NPM

    ```typescript
    npm install @litentry/parachain-api
    ```

2. Extend and decorate the API's types with:

    ```typescript
    import { identity, vc, trusted_operations, sidechain, omniAccount } from "parachain-api";

    const types = {
        ...identity.types,
        ...vc.types,
        ...omniAccount.types,
        ...trusted_operations.types,
        ...sidechain.types,
    };

    const api = await ApiPromise.create({
        provider,
        types,
    });
    ```

3. Import type definitions as needed:

    ```typescript
    import type { LitentryIdentity } from "@litentry/parachain-api";

    function myFunction(identity: LitentryIdentity) {
        // ...
    }
    ```

## Versions

This package is distributed under two main tags: `next` and `latest`.

Versions in the pattern of `x.x.x-next.x` feature the most recent code version to work with `tee-dev`. E.g., `1.0.0-next.0`. Once stable and once the Litentry Protocol is upgraded, the version will be tagged as `latest` and should be used against `tee-prod`. E.g.`1.0.0`.

## Publish new versions

1. [Update your published package version number](https://docs.npmjs.com/updating-your-published-package-version-number)

1. Update the `CHANGELOG.md` file

1. Build the package

    ```s
    pnpm run build
    ```

1. Publish the distribution files

    Use `next` tag for preview versions. Use `latest` tag for

    ```s
    # for preview versions
    npm publish --access=public --tag next

    # for stable versions
    npm publish --access=public --tag latest
    ```

## How to regenerate types

Please read the commands of [client-api](https://github.com/litentry/heima/blob/dev/tee-worker/identity/client-api/README.md).
