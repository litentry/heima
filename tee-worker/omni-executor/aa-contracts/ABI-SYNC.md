# ABI Synchronization Documentation

## Overview

This document explains the automated ABI synchronization system that keeps the demo app in sync with the smart contracts, even when contract names change (e.g., V1 naming).

## Automated Workflow

### 1. During Deployment (`deploy-local.sh`)
- Deploys V1 contracts: `EntryPointV1`, `OmniAccountFactoryV1`, etc.
- Saves deployment artifacts to `deployments/local/local.json` with:
  - Contract addresses
  - Full ABIs
  - Metadata (for tokens)
- Automatically runs `sync-abis-from-deployment.sh` if available

### 2. ABI Synchronization (`sync-abis-from-deployment.sh`)
- Reads deployment artifacts from `deployments/local/local.json`
- Maps V1 contract names to demo app expected names:
  - `EntryPointV1` → `EntryPoint.json`
  - `OmniAccountFactoryV1` → `OmniAccountFactory.json`
- Extracts `OmniAccountV1` ABI from compiled artifacts
- Saves all ABIs to `aa-demo-app/src/contracts/abis/`

### 3. Address Update (`update-demo-addresses.sh`)
- Primary source: deployment artifacts (`deployments/local/local.json`)
- Fallback: broadcast files
- Automatically runs ABI sync after updating addresses
- Updates `.env.local` with contract addresses

## Manual Usage

### Sync ABIs Only
```bash
./sync-abis-from-deployment.sh
```

### Update Addresses and ABIs
```bash
./update-demo-addresses.sh
```

### For Different Environments
```bash
DEPLOYMENT_ENV=staging ./sync-abis-from-deployment.sh
DEPLOYMENT_ENV=staging ./update-demo-addresses.sh
```

## Benefits

1. **Single Source of Truth**: Deployment artifacts contain everything
2. **Automatic V1 Mapping**: No need to update demo app for V1 naming
3. **Version Agnostic**: Works with any contract version naming
4. **Environment Support**: Easy switching between local/staging/production
5. **No Manual Compilation**: Reuses existing artifacts

## Troubleshooting

### Missing OmniAccount ABI
The script will automatically run `forge build` if needed to generate the OmniAccountV1 artifact.

### Multiple Test Tokens
The script handles multiple TestToken instances by checking metadata for USDC/USDT symbols.

### Custom Networks
Set `DEPLOYMENT_ENV` to use different deployment files:
```bash
DEPLOYMENT_ENV=arbitrum-sepolia ./sync-abis-from-deployment.sh
```