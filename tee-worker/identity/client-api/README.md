## Description

Client-api of tee-worker

## Purpose

In order to enable the use of our parachain and sidechain types in client and other projects.

## Environment setup

-   Install [nvm](https://github.com/nvm-sh/nvm)
-   Inside the repository, run `nvm use` to set the correct Node version.
    -   If the version is not installed, run `nvm install`.

## Installation

```cd tee-worker/ts-tests
cd tee-worker/client-api
nvm use
corepack enable pnpm
pnpm install
```

## Type Generated

Update parachain metadata: `pnpm --filter parachain-api run update-metadata` (requires the parachain is running)

Update sidechain metadata: `pnpm --filter sidechain-api run update-metadata` (requires the worker is running)

Generate parachain type: `pnpm --filter parachain-api run build`

Generate sidechain type: `pnpm --filter sidechain-api run build`

Alternatively, you can run `pnpm --run update-build` to do all things above in one go.

## Test

Once update the `client-api/parachain-api` or `client-api/sidechain-api`, all you need to do in the tee-worker/ts-tests is to re-install(run `pnpm install` in tee-worker/ts-tests) and then you can test directly.

## Publish

1. [parachain-api](https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/client-api/parachain-api/README.md#publish-new-versions)
2. [sidechain-api](https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/client-api/sidechain-api/README.md#publish-new-versions)
