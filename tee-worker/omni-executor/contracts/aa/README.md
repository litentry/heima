# aa-contracts

Contains Solidity code for ERC-4337 Account Abstraction.

This project includes code from https://github.com/eth-infinitism/account-abstraction git commit sha: cc3893bcaf2272c163ce89d5eb9eadb8e6b52db7
licensed under the GNU General Public License v3.0.

## Compiling

`forge compile`

## Running tests

`forge test`

## Deploying locally

Run the deployment script to start an Anvil node and deploy all contracts:

```bash
./deploy-local.sh
```

This script will:
- Start an Anvil node (or use existing one on port 8545)
- Compile the contracts
- Deploy EntryPoint, OmniAccountFactory, and SimplePaymaster contracts
- Display the deployed contract addresses

The script keeps Anvil running after deployment. Press Ctrl+C to stop both the script and Anvil.

## Deployment Artifacts

When deploying contracts, the scripts now generate enhanced deployment artifacts in the `deployments/` directory. These artifacts include:

- **Contract Address**: The deployed address of each contract
- **ABI**: The full Application Binary Interface for contract interaction
- **Bytecode**: The deployed bytecode for verification purposes
- **Metadata**: Optional additional information (e.g., configuration parameters)

### Artifact Structure

Deployment artifacts are saved in environment-based subdirectories with network-specific JSON files:
- Local development: `deployments/local/local.json`
- Staging: `deployments/staging/ethereum-sepolia.json`, `deployments/staging/arbitrum-sepolia.json`, etc.
- Production: `deployments/production/ethereum.json`, `deployments/production/arbitrum.json`, etc.

```json
{
  "network": "Local Network",
  "chainId": 31337,
  "timestamp": 1234567890,
  "blockNumber": 1,
  "deployer": "0x...",
  "contracts": {
    "EntryPoint": {
      "address": "0x...",
      "abi": [...],
      "bytecode": "0x...",
      "metadata": {}
    },
    // ... other contracts
  }
}
```

### Enabling Artifact Generation

To enable deployment artifact generation, set the `SAVE_DEPLOYMENT_FILE` environment variable:

```bash
SAVE_DEPLOYMENT_FILE=true forge script script/Deploy.s.sol --rpc-url <RPC_URL> --broadcast
```

The local deployment script (`./local-deploy.sh`) automatically enables artifact generation.

### Using Deployment Artifacts

Once generated, deployment artifacts can be used to interact with deployed contracts:

```javascript
const deployment = require('./deployments/local/local.json');

// Get contract address and ABI
const entryPointAddress = deployment.contracts.EntryPoint.address;
const entryPointABI = deployment.contracts.EntryPoint.abi;

// Create contract instance (ethers.js example)
const entryPoint = new ethers.Contract(entryPointAddress, entryPointABI, provider);
```

### Remote Deployment

For remote deployments, use the provided script template:

```bash
# Set the environment variables
# Then run:
./deploy-remote.sh
```

The script will:
1. Deploy contracts to the specified network
2. Generate deployment artifacts with addresses and bytecode
3. Enrich artifacts with ABIs from Foundry build outputs
4. Save the complete artifacts to `deployments/<environment>/<network>.json`

### Artifact Contents

Each deployment artifact includes:

- **network**: Network name (e.g., "Ethereum Sepolia", "Local Network")
- **chainId**: Chain ID of the deployment network
- **timestamp**: Unix timestamp of deployment
- **blockNumber**: Block number at deployment time
- **deployer**: Address that deployed the contracts
- **contracts**: Object containing each deployed contract with:
  - **address**: Deployed contract address
  - **abi**: Full contract ABI for interaction
  - **bytecode**: Deployed bytecode for verification
  - **metadata**: Optional configuration parameters (e.g., paymaster type, initial bundler)

### Verification

The included bytecode can be used to verify deployed contracts on block explorers or to ensure the deployed code matches the source.

Also see [DEPLOYMENT.md](./DEPLOYMENT.md) for more information.
