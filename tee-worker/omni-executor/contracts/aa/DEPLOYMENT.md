# AA Contracts Deployment Guide

This guide provides comprehensive instructions for deploying Account Abstraction (AA) contracts to any EVM-compatible network using Foundry.

## 📋 Overview

The deployment script can deploy the following contracts:
- **EntryPointV1**: The main entry point for ERC-4337 user operations (always deployed)
- **OmniAccountFactoryV1**: Factory contract for creating OmniAccount smart wallets (always deployed)
- **SimplePaymaster**: Basic paymaster contract for sponsoring transactions (optional, default: enabled)
- **ERC20PaymasterV1**: Advanced paymaster that accepts ERC20 tokens as payment (optional, default: disabled)

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
| Ethereum Mainnet | 1 | `ethereum.json` |
| Ethereum Sepolia | 11155111 | `ethereum-sepolia.json` |
| Arbitrum Mainnet | 42161 | `arbitrum.json` |
| Arbitrum Sepolia | 421614 | `arbitrum-sepolia.json` |
| BSC Mainnet | 56 | `bsc.json` |
| BSC Testnet | 97 | `bsc-testnet.json` |
| Polygon Mainnet | 137 | `polygon.json` |
| Polygon Mumbai | 80001 | `polygon-mumbai.json` |
| HyperEVM Mainnet | 999 | `hyperevm.json` |
| HyperEVM Testnet | 998 | `hyperevm-testnet.json` |
| Local Anvil | 1337/31337 | `local.json` |

## ⚙️ Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# Required
PRIVATE_KEY=0x1234567890abcdef...  # Your deployer private key
RPC_URL=https://your-rpc-endpoint   # Network RPC URL

# Optional - Contract Deployment Configuration
DEPLOY_ENTRYPOINT=true                         # Deploy EntryPointV1 (default: true)
DEPLOY_FACTORY=true                            # Deploy OmniAccountFactoryV1 (default: true)
DEPLOY_SIMPLE_PAYMASTER=true                   # Deploy SimplePaymaster (default: true)
DEPLOY_ERC20_PAYMASTER=false                   # Deploy ERC20PaymasterV1 (default: false)

# Required when DEPLOY_ENTRYPOINT=false
ENTRYPOINT_ADDRESS=0x1234567890abcdef...       # Address of existing EntryPoint (required if not deploying new one)

# Optional - Configuration
PAYMASTER_INITIAL_DEPOSIT=1000000000000000000  # 1 ETH in wei (default: 1 ETH)
INITIAL_BUNDLER=0x1234567890abcdef...          # Initial bundler address (default: deployer)
SAVE_DEPLOYMENT_FILE=true                      # Save deployment file (default: false, set true for official deployments)
DEPLOYMENT_ENV=staging                         # Environment subdirectory (e.g., staging, production)
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

### Using Remote Deployment Script (Recommended)

The `deploy-remote.sh` script handles deployment and automatic ABI enrichment:

```bash
# Set environment variables in .env
DEPLOYMENT_ENV=staging  # Optional: staging, production, etc.

# Run the deployment script
./deploy-remote.sh
```

This script will:
1. Deploy all configured contracts to the specified network
2. Save enhanced artifacts with addresses and bytecode
3. Automatically enrich artifacts with ABIs from Foundry build outputs
4. Save to `deployments/[environment]/[network].json`


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

## 🎯 CREATE2 Deterministic Deployments

### Why CREATE2?

When deploying contracts across multiple EVM chains, standard CREATE deployments (using EOA nonce) require careful nonce management to maintain consistent addresses. If you deploy contracts in different orders on different chains, they'll have different addresses, making multi-chain integrations complex.

**CREATE2** solves this by making contract addresses deterministic based on:
- Factory address (not deployer EOA)
- Salt value
- Contract bytecode

This allows **identical addresses across all chains** when using the same salt and factory address.

### Benefits

✅ **Deterministic Addresses**: Same contract address on all chains
✅ **Order Independent**: Deploy contracts in any order
✅ **Predictable**: Know contract addresses before deployment
✅ **Truly Universal**: Same addresses for all deployers
✅ **Multi-Chain Ready**: Deploy to new networks without any coordination

### CREATE2 Deployment Strategy

#### Step 1: Deploy the CREATE2 Factory

The `Create2FactoryV1` contract must be deployed **once per network** using a **fresh EOA** (recommended for consistency, though not strictly required).

```bash
# Set up environment
source .env

# Deploy the factory
forge script script/DeployCreate2Factory.s.sol:DeployCreate2Factory \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv
```

**Important**:
- The script will warn if your EOA has a non-zero nonce
- For maximum consistency, use a fresh EOA (nonce 0) to deploy the factory on all chains
- Save the factory address - you'll need it for all future deployments

After deployment, add the factory address to `deployments/create2-factories.json`:

```json
{
  "ethereum": "0x...",
  "arbitrum": "0x...",
  "bsc": "0x...",
  "hyperevm": "0x..."
}
```

#### Step 2: Deploy AA Contracts via CREATE2

Once the factory is deployed, you can deploy AA contracts using two methods:

**Option A: Deploy All Contracts at Once**

```bash
# Set factory address
export CREATE2_FACTORY_ADDRESS=0x...  # From step 1

# Deploy all AA contracts via CREATE2
forge script script/DeployWithCreate2.s.sol:DeployWithCreate2 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv
```

**Option B: Deploy Individual Contract**

```bash
# Set factory address
export CREATE2_FACTORY_ADDRESS=0x...  # From step 1

# Deploy a single contract
CONTRACT_NAME=EntryPointV1 forge script script/DeployContract.s.sol:DeployContract \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv

# Deploy factory (requires EntryPoint)
CONTRACT_NAME=OmniAccountFactoryV1 ENTRYPOINT_ADDRESS=0x... \
    forge script script/DeployContract.s.sol:DeployContract \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv
```

#### Step 3: Deploy to Additional Networks

To deploy to a new network with the **same addresses**:

1. Deploy the CREATE2 factory on the new network (step 1)
2. Deploy contracts using either method from step 2
3. Contracts will deploy to **identical addresses** automatically!

```bash
# Example: Deploy to new network
export RPC_URL=https://new-network-rpc.example.com
export CREATE2_FACTORY_ADDRESS=0x...  # Factory on new network

# Addresses will be identical across all chains!
forge script script/DeployWithCreate2.s.sol:DeployWithCreate2 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    -vvv
```

### CREATE2 Configuration

**DeployWithCreate2.s.sol** - All standard environment variables from `Deploy.s.sol` are supported, plus:

```bash
# CREATE2-specific variables
CREATE2_FACTORY_ADDRESS=0x...  # Address of deployed Create2FactoryV1 (required)

# All standard variables still work
DEPLOY_ENTRYPOINT=true
DEPLOY_FACTORY=true
DEPLOY_SIMPLE_PAYMASTER=true
DEPLOY_ERC20_PAYMASTER=false
PAYMASTER_INITIAL_DEPOSIT=1000000000000000000
INITIAL_BUNDLER=0x...
SAVE_DEPLOYMENT_FILE=true
DEPLOYMENT_ENV=production
```

**DeployContract.s.sol** - For individual contract deployment:

```bash
# Required
CREATE2_FACTORY_ADDRESS=0x...  # Address of deployed Create2FactoryV1
CONTRACT_NAME=EntryPointV1      # Contract to deploy (EntryPointV1, OmniAccountFactoryV1, SimplePaymaster, ERC20PaymasterV1)

# Required for Factory/Paymaster contracts
ENTRYPOINT_ADDRESS=0x...        # Address of deployed EntryPoint

# Optional for Paymaster contracts
INITIAL_BUNDLER=0x...           # Default: deployer address
PAYMASTER_INITIAL_DEPOSIT=1000000000000000000  # Default: 1 ETH

# Optional
SAVE_DEPLOYMENT_FILE=true       # Default: false
DEPLOYMENT_ENV=production       # Default: empty
```

### Salt Generation Strategy

The deployment scripts use **purely deterministic salts** based only on contract name:

```solidity
salt = keccak256(abi.encode(contractName))
```

This means:
- **Same contract name** = **same address** on all chains for all deployers
- Truly universal addresses across all EVM chains
- Anyone can deploy to the predicted address (first deployment wins)

### Address Prediction

Before deployment, the script shows predicted addresses:

```
=== Predicted Addresses ===
EntryPointV1 (predicted):      0x1234...
OmniAccountFactoryV1 (predicted): 0x5678...
SimplePaymaster (predicted):   0xabcd...
```

You can also compute addresses manually:

```solidity
// In Solidity
Create2FactoryV1 factory = Create2FactoryV1(factoryAddress);
bytes32 salt = factory.generateSalt("EntryPointV1");
address predicted = factory.computeAddress(salt, type(EntryPointV1).creationCode);
```

```bash
# Using cast
cast call $FACTORY_ADDRESS "computeAddress(bytes32,bytes)(address)" \
    $SALT \
    $(cast --from-utf8 "$(cat out/EntryPointV1.sol/EntryPointV1.json | jq -r .bytecode.object)")
```

### Migration from Standard Deployments

**Current deployments are preserved** - no migration needed!

- Existing contracts on 15+ networks continue to work
- CREATE2 factory is used **only for future deployments**
- When deploying to new networks, use CREATE2 for consistency

### Deterministic Bytecode Configuration

The `foundry.toml` has been configured for deterministic builds:

```toml
solc_version = "0.8.28"
evm_version = "cancun"
bytecode_hash = "none"        # Critical for determinism
cbor_metadata = false         # Critical for determinism
optimizer = true
optimizer_runs = 1000000
```

**Important**: These settings ensure identical bytecode across builds, which is essential for CREATE2 determinism. Do not modify these settings between deployments.

### Troubleshooting

**Problem**: Addresses don't match across chains
**Solution**: Ensure the Create2FactoryV1 is deployed to the same address on all chains (use same fresh EOA)

**Problem**: Factory deployment fails
**Solution**: Make sure you have enough ETH for deployment gas

**Problem**: "AddressAlreadyDeployed" error
**Solution**: Contract was already deployed to this deterministic address. Check if it's functioning correctly or use a different contract name

**Problem**: Verification fails with "Bytecode does not match"
**Solution**: Ensure your local build uses the same compiler settings as deployment

## 📁 Deployment Artifacts

After successful deployment, you'll find:

### Console Output Examples

**Full deployment with both paymasters:**
```
=== AA Contracts Deployment ===
Network: BSC Testnet
Chain ID: 97
Deployer address: 0x...
Deployer balance: 1.5 ETH

🚀 Deploying EntryPointV1...
✅ EntryPointV1 deployed at: 0x...

🚀 Deploying OmniAccountFactoryV1...
✅ OmniAccountFactoryV1 deployed at: 0x...

🚀 Deploying SimplePaymaster...
✅ SimplePaymaster deployed at: 0x...

🚀 Deploying ERC20PaymasterV1...
✅ ERC20PaymasterV1 deployed at: 0x...

💰 Initializing SimplePaymaster with deposit...
✅ SimplePaymaster initialized

💰 Initializing ERC20PaymasterV1 with deposit...
✅ ERC20PaymasterV1 initialized

=== DEPLOYMENT COMPLETE ===
📋 Contract Addresses:
EntryPointV1:          0x...
OmniAccountFactoryV1:  0x...
SimplePaymaster:       0x...
ERC20PaymasterV1:      0x...
```

### JSON File (Enhanced Artifacts)
Location: `deployments/[environment]/[network].json` (e.g., `deployments/staging/bsc-testnet.json`)

```json
{
  "network": "BSC Testnet",
  "chainId": 97,
  "timestamp": 1703001234,
  "blockNumber": 123456,
  "deployer": "0x...",
  "contracts": {
    "EntryPointV1": {
      "address": "0x...",
      "abi": [...],  // Full contract ABI
      "bytecode": "0x608060...",  // Deployed bytecode
      "metadata": {}
    },
    "OmniAccountFactoryV1": {
      "address": "0x...",
      "abi": [...],
      "bytecode": "0x608060...",
      "metadata": {}
    },
    "SimplePaymaster": {
      "address": "0x...",
      "abi": [...],
      "bytecode": "0x608060...",
      "metadata": {
        "initialBundler": "0x..."
      }
    },
    "ERC20PaymasterV1": {
      "address": "0x...",
      "abi": [...],
      "bytecode": "0x608060...",
      "metadata": {
        "initialBundler": "0x..."
      }
    }
  }
}
```

The artifacts are saved in environment-based subdirectories:
- **Local**: `deployments/local/`
- **Staging**: `deployments/staging/`
- **Production**: `deployments/production/`
- **No environment**: `deployments/` (backward compatible)

### Broadcast Files
Foundry creates detailed transaction logs in:
```
broadcast/Deploy.s.sol/[CHAIN_ID]/run-latest.json
```

## 📝 ABI Enrichment

The deployment scripts automatically enrich artifacts with contract ABIs after deployment. If this fails or you need to manually enrich:

```bash
# Run the ABI enrichment script
node extract-abis.js
```

This script:
- Reads all deployment files from `deployments/`
- Extracts ABIs from Foundry build artifacts
- Updates deployment files with full contract ABIs
- Handles both old (address-only) and new (enhanced) formats

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
# Verify EntryPointV1
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor()") \
    [ENTRYPOINT_ADDRESS] \
    src/core/EntryPointV1.sol:EntryPointV1 \
    --etherscan-api-key $ETHERSCAN_API_KEY

# Verify Factory
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor(address)" [ENTRYPOINT_ADDRESS]) \
    [FACTORY_ADDRESS] \
    src/accounts/OmniAccountFactoryV1.sol:OmniAccountFactoryV1 \
    --etherscan-api-key $ETHERSCAN_API_KEY

# Verify SimplePaymaster
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor(address,address)" [ENTRYPOINT_ADDRESS] [INITIAL_BUNDLER]) \
    [SIMPLE_PAYMASTER_ADDRESS] \
    src/core/SimplePaymaster.sol:SimplePaymaster \
    --etherscan-api-key $ETHERSCAN_API_KEY

# Verify ERC20PaymasterV1
forge verify-contract \
    --chain-id [CHAIN_ID] \
    --constructor-args $(cast abi-encode "constructor(address,address)" [ENTRYPOINT_ADDRESS] [INITIAL_BUNDLER]) \
    [ERC20_PAYMASTER_ADDRESS] \
    src/core/ERC20PaymasterV1.sol:ERC20PaymasterV1 \
    --etherscan-api-key $ETHERSCAN_API_KEY
```

## 📋 Managing Official Deployments

### Deployment Files Management

The script creates enhanced deployment files when `SAVE_DEPLOYMENT_FILE=true`. For official deployments:

```bash
# After successful deployment, commit only official deployment files
git add deployments/production/ethereum.json     # For Ethereum mainnet
git add deployments/production/arbitrum.json     # For Arbitrum mainnet
git add deployments/staging/bsc-testnet.json     # For BSC testnet staging
git commit -m "Deploy AA contracts to mainnet"

# Tag official releases
git tag v1.0.0-mainnet
git push origin v1.0.0-mainnet
```

### Using Deployment Artifacts

```javascript
// Load deployment artifacts in your application
const deployment = require('./deployments/staging/arbitrum-sepolia.json');

// Access contract addresses and ABIs
const entryPointAddress = deployment.contracts.EntryPointV1.address;
const entryPointABI = deployment.contracts.EntryPointV1.abi;

// Create contract instance (ethers.js example)
const entryPoint = new ethers.Contract(entryPointAddress, entryPointABI, provider);
```

**Note**: The `broadcast/` folder is in `.gitignore` and should not be committed as it contains transaction details that change with each deployment run.