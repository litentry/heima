# AA Demo App - Account Abstraction Demo Application

A Next.js application demonstrating Account Abstraction (ERC-4337) functionality with OmniAccount smart contracts. This demo showcases how to create and manage smart contract wallets with root key authorization for non-custodial operations.

## Overview

This application demonstrates:
- **Account Abstraction (AA)**: Create smart contract wallets (OmniAccounts) that can be controlled by multiple signers
- **Multi-Signer Management**: Add and remove authorized signers to control the smart account
- **ERC20 Token Transfers**: Send USDC and USDT through the TEE worker using UserOperations
- **Real Token Support**: Use actual deployed tokens instead of test tokens
- **Non-Custodial Flow**: Users maintain full control of their accounts while enabling delegated operations
- **ERC-4337 Integration**: Implements the ERC-4337 standard for account abstraction
- **TEE Worker Integration**: Authorize trusted execution environment workers to execute transactions securely
- **Paymaster Support**: Optional gas sponsorship through integrated paymaster contracts

## Key Features

- **Separated Funding Flow**: Fund with ETH first for gas
- **Multi-Token Support**: Send USDC and USDT transfers through TEE worker
- **Signer Management**: View, add, and remove authorized signers through the UI
- **Token Balance Display**: Monitor all token balances in real-time
- **TEE Worker Integration**: Execute token transfers securely through TEE worker
- **Root Key Delegation**: Authorize multiple signers to control your smart account
- **TEE Worker Authorization**: Delegate transaction execution to secure TEE workers
- **Gas Sponsorship**: Enable paymaster to cover transaction fees for users

## Prerequisites

- Node.js 18+ and pnpm
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

**Note**: By default, this deploys with SimplePaymaster (requires authorized bundlers). To deploy with DemoPaymaster (no bundler restrictions, for testing only):

```bash
PAYMASTER_TYPE=demo ./local-deploy.sh
```

You should see output like:
```
🎉 All contracts deployed successfully!

Contract Addresses:
===================
EntryPoint:         0x5fbdb2315678afecb367f032d93f642f64180aa3
OmniAccountFactory: 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512
Paymaster:          0x...

Token Addresses:
================
USDC:               <Set via NEXT_PUBLIC_TEST_USDC_ADDRESS>
USDT:               <Set via NEXT_PUBLIC_TEST_USDT_ADDRESS>
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
pnpm install

# Start the development server
pnpm dev
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
   - **Optional**: If a paymaster is deployed and funded, you can enable gas sponsorship instead

3. **Create Omni Account**: Once ETH is received, create your smart account
   - This deploys your OmniAccount contract
   - Your wallet automatically becomes the initial root signer
   - **Optional**: Toggle "Use Paymaster" to have gas fees sponsored

4. **Authorize TEE Worker**: Authorize the TEE worker to execute transactions
   - Click "Authorize TEE Worker" to authenticate with the TEE service
   - The worker will be added as an authorized signer
   - Enables secure delegated transaction execution
   - The worker address will be tagged in the signers list

5. **Send Token Transfer**: Transfer USDC or USDT through the TEE worker
   - Select the token you want to transfer (USDC or USDT)
   - Enter recipient address and amount
   - The transfer is executed through a UserOperation signed by the TEE worker
   - Monitor transaction status in real-time

6. **Manage Signers**: After deployment, manage authorized signers
   - View all current authorized signers
   - Add new signers by entering their address
   - Remove existing signers (except yourself while connected)
   - **Optional**: Enable paymaster for signer management operations

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
│   │   ├── AccountsDashboard.tsx    # Displays wallet and OmniAccount balances
│   │   ├── AuthorizedSigners.tsx    # Manage authorized signers
│   │   ├── AuthorizeTEEWorker.tsx   # TEE worker authorization flow
│   │   ├── FundingGuide.tsx         # ETH funding guide
│   │   ├── TEETokenTransfer.tsx     # Token transfer through TEE worker
│   │   ├── CreateOmniAccount.tsx    # Smart account creation flow with paymaster option
│   │   └── WalletConnect.tsx        # Wallet connection component
│   ├── contracts/        # Contract ABIs
│   │   ├── ...existing ABIs
│   │   └── SimplePaymaster.json     # Paymaster contract ABI
│   └── lib/             # Utilities and configuration
│       ├── aa-utils.ts  # AA utilities including UserOp construction
│       ├── constants.ts # Contract addresses and token configs
│       ├── tee-worker-client.ts # TEE Worker RPC client with submitUserOpTest
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
- Make sure USDC and USDT addresses are set in .env.local
- Ensure you have ETH for gas fees before sending token transfers
- Token transfers require TEE worker to be authorized
- Check that your Omni Account has sufficient token balance

### Anvil Errors
You might see errors like `execution reverted` for `symbol()` or `decimals()` calls. These are harmless - they're from wallets trying to detect if addresses are ERC20 tokens.

### TEE Worker Authorization Issues
- **Parse errors**: Ensure you're using the correct client ID ("wildmeta" for staging)
- **Authentication failures**: The TEE worker uses Web3 message signing for authentication
- **Server errors**: Check that the TEE Worker RPC URL is accessible
- **Worker not added**: Ensure your Omni Account is deployed before authorizing

### Paymaster Issues
- **"Paymaster not available"**: The paymaster might not be deployed or funded
- **"Insufficient balance"**: The paymaster needs ETH deposited at the EntryPoint
- **Transaction fails with paymaster**: Ensure the paymaster is properly configured
- **DemoPaymaster vs SimplePaymaster**: DemoPaymaster accepts all operations (testing only), SimplePaymaster requires authorized bundlers

## Environment Variables

The app uses these environment variables (set automatically by `update-demo-addresses.sh`):

- `NEXT_PUBLIC_CHAIN_ID`: Network chain ID (1337 for Anvil)
- `NEXT_PUBLIC_ENTRYPOINT_ADDRESS`: EntryPoint contract address
- `NEXT_PUBLIC_FACTORY_ADDRESS`: OmniAccountFactory contract address
- `NEXT_PUBLIC_TEST_USDC_ADDRESS`: USDC token address (real deployed token)
- `NEXT_PUBLIC_TEST_USDT_ADDRESS`: USDT token address (real deployed token)
- `NEXT_PUBLIC_RPC_URL`: Ethereum RPC URL (http://localhost:8545)
- `NEXT_PUBLIC_TEE_WORKER_RPC_URL`: TEE Worker RPC endpoint (default: https://staging-dex-worker.heima.network)
- `NEXT_PUBLIC_PAYMASTER_ADDRESS`: Paymaster contract address (0x0 if not deployed)

You can also copy `.env.local.example` to `.env.local` and update manually.

## Development

To modify the app:

1. Smart contract changes: Update contracts in parent `src/` directory
2. Re-deploy: Run `./local-deploy.sh` again (or `PAYMASTER_TYPE=demo ./local-deploy.sh` for DemoPaymaster)
3. Update addresses: Run `./update-demo-addresses.sh`
4. The app hot-reloads automatically

### Paymaster Configuration

The app supports two paymaster types:

1. **SimplePaymaster** (default): Production-ready, requires authorized bundlers
2. **DemoPaymaster**: For testing, accepts any operation (not for production)

To configure paymaster behavior, edit `src/lib/constants.ts`:
```typescript
export const PAYMASTER_CONFIG = {
  defaultValidationGasLimit: BigInt(100000),
  defaultPostOpGasLimit: BigInt(50000),
  enabledByDefault: false, // Set to true to enable by default
};
```

### Adding New ERC20 Tokens

1. Deploy your token contract on the target chain
2. Update `src/lib/constants.ts` with token details
3. Add to `TEST_TOKENS` object with address, symbol, decimals, and ABI
4. The TEETokenTransfer component will automatically include it

### Testing Signer Management

1. Deploy and fund an OmniAccount
2. Use the AuthorizedSigners component to add signers
3. Test from different wallets to verify permissions

## Technical Details

### Paymaster Integration

The app integrates paymaster functionality for gas sponsorship:

- **UserOperation Enhancement**: Automatically encodes `paymasterAndData` field when paymaster is enabled
- **Balance Checking**: Verifies paymaster has sufficient deposit at EntryPoint
- **Dynamic Toggle**: Users can enable/disable paymaster per transaction

### New Utility Functions

- `buildTokenTransferUserOp()`: Creates UserOperation for ERC20 token transfers
- `buildERC20TransferCallData()`: Encodes ERC20 transfer function call
- `toSerializablePackedUserOperation()`: Converts PackedUserOperation to serializable format
- `submitUserOpTest()`: Submits UserOperations through TEE worker RPC

### RPC Method Updates

The app now uses the new `omni_getSmartWalletRootSigner` RPC method for improved TEE worker integration.

## Learn More

- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337)
- [Foundry Book](https://book.getfoundry.sh/)
- [ERC20 Token Standard](https://eips.ethereum.org/EIPS/eip-20)
- [Account Abstraction Paymasters](https://eips.ethereum.org/EIPS/eip-4337#paymaster-1)
