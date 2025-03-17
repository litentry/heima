## Description

ts-tests of tee-worker

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
cd tee-worker/ts-tests
nvm use
corepack enable pnpm
pnpm install
```

## Local

[Start parachain && worker](https://github.com/litentry/heima/blob/dev/README.md)

## Usage(ts-tests folder)

```
pnpm --filter integration-tests run test your-testfile.test.ts
```

Direct invocation identity test: `pnpm --filter integration-tests run test di_identity.test.ts`

Direct invocation vc test: `pnpm --filter integration-tests run test vc_correctness.test.ts`

Direct requect vc test: `pnpm --filter integration-tests run test dr_vc.test.ts`

## Data-provider test

1. Start tee-worker with real endpoint and real code(Configure in `local-setup/env.dev.`).
2. Configure definitions in `ts-tests/integration-tests/common/credential-json/*.json`,like [vip3-membership-card-gold](https://github.com/litentry/heima/blob/dev/tee-worker/identity/ts-tests/integration-tests/common/credential-json/vip3.json#L3)
3. Execute test cases:
   1. Single test:  `pnpm run test-data-providers:local --id=your credential json id` 
   2. All credential tests:`pnpm run test-data-providers:local`（Run all the `credential-json/*.json` test cases, execute them in the order of [export](https://github.com/litentry/heima/blob/dev/tee-worker/identity/ts-tests/integration-tests/common/credential-json/index.ts#L21).）
