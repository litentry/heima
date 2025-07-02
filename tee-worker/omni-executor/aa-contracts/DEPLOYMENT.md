# AA Contracts Deployment Guide

This guide provides comprehensive instructions for deploying Account Abstraction (AA) contracts to any EVM-compatible network using Foundry.

## 📋 Overview

The deployment script deploys three core contracts:
- **EntryPoint**: The main entry point for ERC-4337 user operations
- **OmniAccountFactory**: Factory contract for creating OmniAccount smart wallets
- **SimplePaymaster**: Paymaster contract for sponsoring transactions

## 🔧 Prerequisites

1. **Foundry installed**
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

2. **Project dependencies**
   ```bash
   forge install
   ```

3. **Private key and RPC URL** for your target network

## 🌐 Supported Networks

The script automatically detects and configures for these networks:

| Network | Chain ID |  Filename |
|---------|----------|----------|
| Ethereum Mainnet | 1 | `mainnet.json` |
| Ethereum Sepolia | 11155111 | `sepolia.json` |
| BSC Mainnet | 56 | `bsc.json` |
| BSC Testnet | 97 | `bsc-testnet.json` |
| Polygon Mainnet | 137 | `polygon.json` |
| Polygon Mumbai | 80001 | `mumbai.json` |
| Local Anvil | 1337 | `local.json` |

## ⚙️ Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# Required
PRIVATE_KEY=0x1234567890abcdef...  # Your deployer private key
RPC_URL=https://your-rpc-endpoint   # Network RPC URL

# Optional
PAYMASTER_INITIAL_DEPOSIT=1000000000000000000  # 1 ETH in wei (default: 1 ETH)
INITIALIZE_PAYMASTER=true                      # Whether to initialize paymaster (default: true)
INITIAL_BUNDLER=0x1234567890abcdef...          # Initial bundler address (default: deployer)
SAVE_DEPLOYMENT_FILE=true                      # Save deployment file (default: false, set true for official deployments)
ETHERSCAN_API_KEY=ABC123                       # For contract verification
```

## 🚀 Deployment Commands

### Basic Deployment (Any Network)

```bash
# Load environment variables
source .env

# Deploy to any network
forge script script/Deploy.s.sol:Deploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv
```


### Ethereum Mainnet with Verification
```bash
forge script script/Deploy.s.sol:Deploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --etherscan-api-key $ETHERSCAN_API_KEY \
    --chain-id 1 \
    -vvv
```

## 📁 Deployment Artifacts

After successful deployment, you'll find:

### Console Output
```
=== AA Contracts Deployment ===
Network: BSC Testnet
Chain ID: 97
Deployer address: 0x...
Deployer balance: 1.5 ETH

🚀 Deploying EntryPoint...
✅ EntryPoint deployed at: 0x...

🚀 Deploying OmniAccountFactory...
✅ OmniAccountFactory deployed at: 0x...

🚀 Deploying SimplePaymaster...
✅ SimplePaymaster deployed at: 0x...

💰 Initializing Paymaster with deposit...
✅ Paymaster initialized

=== DEPLOYMENT COMPLETE ===
📋 Contract Addresses:
EntryPoint:          0x...
OmniAccountFactory:  0x...
SimplePaymaster:     0x...
```

### JSON File
Location: `deployments/bsc-testnet.json`
```json
{
  "network": "BSC Testnet",
  "chainId": 97,
  "timestamp": 1703001234,
  "deployer": "0x...",
  "contracts": {
    "EntryPoint": "0x...",
    "OmniAccountFactory": "0x...",
    "SimplePaymaster": "0x..."
  }
}
```

### Broadcast Files
Foundry creates detailed transaction logs in:
```
broadcast/Deploy.s.sol/[CHAIN_ID]/run-latest.json
```

## 🔍 Contract Verification

### Automatic Verification
Add `--verify` flag and API key:
```bash
forge script script/Deploy.s.sol:Deploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --etherscan-api-key $ETHERSCAN_API_KEY \
    -vvv
```

### Manual Verification
If automatic verification fails:

```bash
# Verify EntryPoint
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor()") \
    [ENTRYPOINT_ADDRESS] \
    src/core/EntryPoint.sol:EntryPoint \
    --etherscan-api-key $ETHERSCAN_API_KEY

# Verify Factory
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor(address)" [ENTRYPOINT_ADDRESS]) \
    [FACTORY_ADDRESS] \
    src/accounts/OmniAccountFactory.sol:OmniAccountFactory \
    --etherscan-api-key $ETHERSCAN_API_KEY

# Verify Paymaster
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor(address)" [ENTRYPOINT_ADDRESS]) \
    [PAYMASTER_ADDRESS] \
    src/core/SimplePaymaster.sol:SimplePaymaster \
    --etherscan-api-key $ETHERSCAN_API_KEY
```

## 📋 Managing Official Deployments

### Deployment Files Management

The script can optionally create deployment files in `deployments/` folder when `SAVE_DEPLOYMENT_FILE=true`. For official deployments:

```bash
# After successful deployment, commit only official deployment files
git add deployments/mainnet.json     # For Ethereum mainnet
git add deployments/bsc.json         # For BSC mainnet  
git add deployments/bsc-testnet.json # For BSC testnet (if official)
git commit -m "Deploy AA contracts to mainnet"

# Tag official releases
git tag v1.0.0-mainnet
git push origin v1.0.0-mainnet
```

**Note**: The `broadcast/` folder is in `.gitignore` and should not be committed as it contains transaction details that change with each deployment run.