## Description

Heima types augment for @polkadot/api

## Environment setup

- Install [nvm](https://github.com/nvm-sh/nvm)

- Inside the repository, run

  ```
  nvm use
  ```

  to set the correct Node version.

  - If the version is not installed, run `nvm install`.

## Installation

```
cd tee-worker/client-api
nvm use
corepack enable pnpm
pnpm install
```

## Type Generated

Update metadata: `pnpm load:metadata`

Generate types:`pnpm generate`

## Usage

```
pnpm run load:metadata
pnpm run generate
pnpm run build
```

- `import '@heima-network/api-augment/omni'` - applies the Heima parachain types and endpoint augmentation (also the package default export)

## Publish

```
pnpm run load:metadata
pnpm run generate
pnpm run build-publish
```
