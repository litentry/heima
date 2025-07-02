# AA Demo App - Account Abstraction Demo Application

A Next.js application demonstrating Account Abstraction (ERC-4337) functionality with OmniAccount smart contracts. This demo showcases how to create and manage smart contract wallets with root key authorization for non-custodial operations.

## Overview

This application demonstrates:
- **Account Abstraction (AA)**: Create smart contract wallets (OmniAccounts) that can be controlled by multiple signers
- **Root Key Authorization**: Add authorized signers to control the smart account
- **Non-Custodial Flow**: Users maintain full control of their accounts while enabling delegated operations
- **ERC-4337 Integration**: Implements the ERC-4337 standard for account abstraction

## Prerequisites

- Node.js 18+ and npm
- [Foundry](https://book.getfoundry.sh/getting-started/installation) installed
- A web3 wallet (MetaMask or similar)
- Git

## Quick Start

Follow these steps to run the demo application locally:

### 1. Deploy Smart Contracts

First, navigate to the parent directory and deploy the contracts:

```bash
# From the aa-contracts directory (parent of aa-demo-app)
cd /path/to/aa-contracts

# Start Anvil (local Ethereum node) and deploy contracts
./local-deploy.sh

# Keep this terminal open - Anvil needs to keep running
```

You should see output like:
```
🎉 All contracts deployed successfully!

Contract Addresses:
===================
EntryPoint:         0x5fbdb2315678afecb367f032d93f642f64180aa3
OmniAccountFactory: 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512
SimplePaymaster:    0x...
```

### 2. Update Demo App Configuration

In a new terminal, update the demo app with the deployed contract addresses:

```bash
# Still in the aa-contracts directory
./update-demo-addresses.sh
```

This creates/updates the `.env.local` file in the demo app with the correct contract addresses.

### 3. Run the Demo App

```bash
# Navigate to the demo app directory
cd aa-demo-app

# Install dependencies (first time only)
npm install

# Start the development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### 4. Use the Application

1. **Connect Wallet**: Click "Connect Wallet" and select your wallet
   - Make sure you're connected to the Anvil network (Chain ID: 1337)
   - If not, add a custom network in MetaMask:
     - Network Name: Anvil Local
     - RPC URL: http://localhost:8545
     - Chain ID: 1337
     - Currency Symbol: ETH

2. **View Your OmniAccount**: After connecting, you'll see your pre-calculated OmniAccount address

3. **Fund Your Account**: Send some ETH to your OmniAccount address
   - Use MetaMask to send ETH to the displayed address
   - Or use Anvil's funded accounts

4. **Authorize Root Key**: Once funded, authorize your wallet as a root signer
   - This deploys your OmniAccount contract
   - Adds your wallet as an authorized signer

## Verifying On-Chain

To verify your OmniAccount was created and the root signer was added:

```bash
# From the aa-contracts directory
cd /path/to/aa-contracts

# Set your OmniAccount address (copy from the UI)
ACCOUNT=0xYOUR_OMNI_ACCOUNT_ADDRESS

# Check if the contract exists
cast code $ACCOUNT --rpc-url http://localhost:8545
# Should return bytecode (not "0x")

# Check if your wallet is a root signer
cast call $ACCOUNT "isRootSigner(address)(bool)" 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://localhost:8545
# Should return "true"

# Check the owner
cast call $ACCOUNT "owner()(bytes32)" --rpc-url http://localhost:8545

# Check the client ID
cast call $ACCOUNT "clientId()(bytes)" --rpc-url http://localhost:8545
```

## Project Structure

```
aa-demo-app/
├── src/
│   ├── app/              # Next.js app router pages
│   ├── components/       # React components
│   │   ├── AAWalletInfo.tsx         # Displays OmniAccount information
│   │   ├── FundingGuide.tsx         # Guide for funding the account
│   │   ├── RootKeyAuthorization.tsx # Root key authorization flow
│   │   └── WalletConnect.tsx        # Wallet connection component
│   ├── contracts/        # Contract ABIs
│   └── lib/             # Utilities and configuration
│       ├── aa-utils.ts  # Account abstraction utilities
│       ├── constants.ts # Contract addresses and ABIs
│       └── wagmi.ts     # Web3 configuration
└── public/              # Static assets
```

## Troubleshooting

### "AA Wallet Not Ready"
- Ensure your wallet is connected
- Check you're on the correct network (Chain ID: 1337)
- Verify contracts are deployed (check Anvil terminal)

### Transaction Failures
- Ensure your OmniAccount is funded with ETH
- Check Anvil is still running
- Verify you're using the correct network

### Anvil Errors
You might see errors like `execution reverted` for `symbol()` or `decimals()` calls. These are harmless - they're from wallets trying to detect if addresses are ERC20 tokens.

## Environment Variables

The app uses these environment variables (set automatically by `update-demo-addresses.sh`):

- `NEXT_PUBLIC_CHAIN_ID`: Network chain ID (1337 for Anvil)
- `NEXT_PUBLIC_ENTRYPOINT_ADDRESS`: EntryPoint contract address
- `NEXT_PUBLIC_FACTORY_ADDRESS`: OmniAccountFactory contract address
- `NEXT_PUBLIC_RPC_URL`: Ethereum RPC URL (http://localhost:8545)

## Development

To modify the app:

1. Smart contract changes: Update contracts in parent `src/` directory
2. Re-deploy: Run `./local-deploy.sh` again
3. Update addresses: Run `./update-demo-addresses.sh`
4. The app hot-reloads automatically

## Learn More

- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337)
- [Foundry Book](https://book.getfoundry.sh/)