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

Also see [DEPLOYMENT.md](./DEPLOYMENT.md) for more information.