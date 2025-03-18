## Description

ts-tests of omni-executor

## Environment setup

-   Install [nvm](https://github.com/nvm-sh/nvm)
-   Inside the repository, run `nvm use` to set the correct Node version.
    -   If the version is not installed, run `nvm install`.

## Prerequisite

Before running the ts-tests, the client-api types generation needs to be completed.

See client-api [README.md](https://github.com/litentry/heima/blob/dev/tee-worker/client-api/README.md)

## Installation

Note: Whenever the `client-api` is updated, it must be reinstalled, and it's best to clean node_modules(`pnpm run clean`) to prevent caching issues.

```
cd tee-worker/omni-executor/ts-tests
nvm use
corepack enable pnpm
pnpm install
```

## Local

[Start parachain && omni-executor worker](https://github.com/litentry/heima/blob/dev/README.md)

## Usage(ts-tests folder)

```
pnpm --filter integration-tests run test your-testfile.test.ts
```
