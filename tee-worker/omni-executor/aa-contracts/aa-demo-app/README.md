# AA Demo App - Account Abstraction Demo Application

A Next.js application demonstrating Account Abstraction (ERC-4337) functionality with OmniAccount smart contracts. This demo showcases how to create and manage smart contract wallets with root key authorization for non-custodial operations.

## Overview

This application demonstrates:
- **Account Abstraction (AA)**: Create smart contract wallets (OmniAccounts) that can be controlled by multiple signers
- **Multi-Signer Management**: Add and remove authorized signers to control the smart account
- **ERC20 Token Support**: Fund accounts with both ETH and ERC20 tokens (USDC, USDT)
- **Test Token Minting**: Mint test tokens for easy demonstration
- **Non-Custodial Flow**: Users maintain full control of their accounts while enabling delegated operations
- **ERC-4337 Integration**: Implements the ERC-4337 standard for account abstraction

## Key Features

- **Separated Funding Flow**: Fund with ETH first for gas, then optionally add ERC20 tokens
- **Multi-Token Support**: Send and receive ETH, USDC, and USDT to your OmniAccount
- **Signer Management**: View, add, and remove authorized signers through the UI
- **Token Balance Display**: Monitor all token balances in real-time
- **Test Token Faucet**: Mint test tokens directly from the UI
- **Root Key Delegation**: Authorize multiple signers to control your smart account

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

Test Token Addresses:
====================
Test USDC:          0x...
Test USDT:          0x...
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
   - After connecting, your Omni Account details will be displayed automatically

2. **Fund with ETH**: Send ETH to your OmniAccount address for gas fees
   - Copy the displayed address or scan the QR code
   - Send at least 0.01 ETH (recommended)
   - The app will automatically detect when funded

3. **Create Omni Account**: Once ETH is received, create your smart account
   - This deploys your OmniAccount contract
   - Your wallet automatically becomes the initial root signer

4. **Add ERC20 Tokens** (Optional): Add USDC or USDT for token swaps
   - Select the token type (USDC or USDT)
   - Use the "Mint Test Tokens" button to get test tokens
   - Send tokens to your OmniAccount address
   - Monitor all token balances in real-time

5. **Manage Signers**: After deployment, manage authorized signers
   - View all current authorized signers
   - Add new signers by entering their address
   - Remove existing signers (except yourself while connected)

## Verifying On-Chain

To verify your OmniAccount was created and signers were added:

```bash
# From the aa-contracts directory
cd /path/to/aa-contracts

# Set your OmniAccount address (copy from the UI)
ACCOUNT=0xYOUR_OMNI_ACCOUNT_ADDRESS

# Check if the contract exists
cast code $ACCOUNT --rpc-url http://localhost:8545
# Should return bytecode (not "0x")

# Check if your wallet is a root signer
cast call $ACCOUNT "isRootSigner(address)(bool)" YOUR_WALLET_ADDRESS --rpc-url http://localhost:8545
# Should return "true"

# Check ERC20 token balance
cast call 0xTOKEN_ADDRESS "balanceOf(address)(uint256)" $ACCOUNT --rpc-url http://localhost:8545

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
│   │   ├── OmniAccountWalletInfo.tsx # Displays OmniAccount information
│   │   ├── AuthorizedSigners.tsx    # Manage authorized signers
│   │   ├── FundingGuide.tsx         # ETH funding guide
│   │   ├── ERC20FundingGuide.tsx    # ERC20 token funding guide
│   │   ├── CreateOmniAccount.tsx    # Smart account creation flow
│   │   └── WalletConnect.tsx        # Wallet connection component
│   ├── contracts/        # Contract ABIs including TestToken
│   └── lib/             # Utilities and configuration
│       ├── aa-utils.ts  # Account abstraction utilities
│       ├── constants.ts # Contract addresses and token configs
│       └── wagmi.ts     # Web3 configuration
└── public/              # Static assets
```

## Troubleshooting

### "AA Wallet Not Ready" or Balance Not Updating
- Ensure your wallet is connected
- Check you're on the correct network (Chain ID: 1337)
- Verify contracts are deployed (check Anvil terminal)
- If balance doesn't update after sending funds, try clicking "Check Balance" or refresh the page
- Clear MetaMask activity data if transactions are being rejected

### Transaction Failures
- Ensure your OmniAccount is funded with ETH (for gas)
- Check Anvil is still running
- Verify you're using the correct network

### Token Operations
- Make sure test tokens are deployed (check deploy output)
- Ensure you have ETH for gas fees before adding ERC20 tokens
- ERC20 token funding is only available after creating your Omni Account
- Check token addresses in .env.local match deployment

### Anvil Errors
You might see errors like `execution reverted` for `symbol()` or `decimals()` calls. These are harmless - they're from wallets trying to detect if addresses are ERC20 tokens.

## Environment Variables

The app uses these environment variables (set automatically by `update-demo-addresses.sh`):

- `NEXT_PUBLIC_CHAIN_ID`: Network chain ID (1337 for Anvil)
- `NEXT_PUBLIC_ENTRYPOINT_ADDRESS`: EntryPoint contract address
- `NEXT_PUBLIC_FACTORY_ADDRESS`: OmniAccountFactory contract address
- `NEXT_PUBLIC_TEST_USDC_ADDRESS`: Test USDC token address
- `NEXT_PUBLIC_TEST_USDT_ADDRESS`: Test USDT token address
- `NEXT_PUBLIC_RPC_URL`: Ethereum RPC URL (http://localhost:8545)

You can also copy `.env.local.example` to `.env.local` and update manually.

## Development

To modify the app:

1. Smart contract changes: Update contracts in parent `src/` directory
2. Re-deploy: Run `./local-deploy.sh` again
3. Update addresses: Run `./update-demo-addresses.sh`
4. The app hot-reloads automatically

### Adding New ERC20 Tokens

1. Deploy your token contract
2. Update `src/lib/constants.ts` with token details
3. Add to `SUPPORTED_TOKENS` array
4. The ERC20FundingGuide component will automatically include it

### Testing Signer Management

1. Deploy and fund an OmniAccount
2. Use the AuthorizedSigners component to add signers
3. Test from different wallets to verify permissions

## Learn More

- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337)
- [Foundry Book](https://book.getfoundry.sh/)
- [ERC20 Token Standard](https://eips.ethereum.org/EIPS/eip-20)