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

Update metadata: `pnpm load:metadata` or `pnpm load:metadata:identity` for specific directory

Generate types:`pnpm generate`

## Usage

```
pnpm run load:metadata
pnpm run generate
pnpm run build
```

- `import '@heima-network/api-augment/identity'` - applies Identity types and endpoint augmentation
- `import '@heima-network/api-augment/omni'` - applies Omni types and endpoint augmentation
- `import '@heima-network/api-augment/sidechain'` - applies Sidechain types and endpoint augmentation

## Publish

```
pnpm run load:metadata
pnpm run generate
pnpm run build-publish
```
