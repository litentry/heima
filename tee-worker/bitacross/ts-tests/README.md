## Description

ts-tests of bitacross-worker

## Environment setup

-   Install [nvm](https://github.com/nvm-sh/nvm)
-   Inside the repository, run `nvm use` to set the correct Node version.
    -   If the version is not installed, run `nvm install`.

## Prerequisite

Before running the ts-tests, the client-api types generation needs to be completed.

See client-api [README.md](https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/client-api/README.md)

## Installation

```
nvm use
corepack enable pnpm
pnpm install
```