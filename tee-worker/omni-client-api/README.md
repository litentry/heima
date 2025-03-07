## Description

Client-api of tee-worker

## Purpose

In order to enable the use of our parachain and sidechain types in client and other projects.

## Environment setup

-   Install [nvm](https://github.com/nvm-sh/nvm)
-   Inside the repository, run `nvm use` to set the correct Node version.
    -   If the version is not installed, run `nvm install`.

## Installation

```
cd tee-worker/omni-client-api
nvm use
corepack enable pnpm
pnpm install
```

## Type Generated

Update parachain metadata: `pnpm --filter parachain-api run update-metadata` (requires the parachain is running)

Generate parachain type: `pnpm --filter parachain-api run build`

Alternatively, you can run `pnpm --run update-build` to do all things above in one go.

## Publish

1. [parachain-api](./parachain-api/README.md#publish-new-versions)
2. [sidechain-api](./sidechain-api/README.md#publish-new-versions)
