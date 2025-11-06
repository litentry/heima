# X402 + ERC-4337 Demo Application

A demonstration application that combines HTTP 402 (Payment Required) protocol with ERC-4337 Account Abstraction smart wallets. This showcases a novel payment flow where users pay for protected content using UserOperations from their smart wallets.

## Overview

This demo implements the following flow:

1. **User Authentication**: User connects their MetaMask wallet
2. **Smart Wallet Discovery**: Application displays the counterfactual ERC-4337 smart wallet address
3. **Protected Content Request**: User requests access to x402-protected content
4. **Payment Challenge**: Server responds with HTTP 402 and payment requirements
5. **UserOp Construction**: Client builds a UserOperation to transfer payment from smart wallet
6. **UserOp Signing**: User signs the UserOperation via MetaMask
7. **Payment Verification**: Server verifies the UserOp through omni-executor simulation
8. **Payment Settlement**: Server settles the UserOp by submitting it to the blockchain
9. **Content Delivery**: Server returns the protected content with transaction hash

## Architecture

```
┌─────────────┐         ┌─────────────┐         ┌────────────────┐
│   Browser   │         │   Express   │         │ Omni-Executor  │
│  (MetaMask) │ ◄────► │   Server    │ ◄────► │   (JSON-RPC)   │
└─────────────┘         └─────────────┘         └────────────────┘
       │                       │                        │
       │ 1. Connect Wallet     │                        │
       ├──────────────────────►│                        │
       │                       │                        │
       │ 2. Request /x402      │                        │
       ├──────────────────────►│                        │
       │ ◄──────────────────── │ (402 + requirements)   │
       │                       │                        │
       │ 3. Sign UserOp        │                        │
       │    (via MetaMask)     │                        │
       │                       │                        │
       │ 4. Send X-Payment     │                        │
       ├──────────────────────►│                        │
       │                       │ 5. Verify UserOp       │
       │                       ├───────────────────────►│
       │                       │ ◄───────────────────── │ (simulation OK)
       │                       │                        │
       │                       │ 6. Settle UserOp       │
       │                       ├───────────────────────►│
       │                       │ ◄───────────────────── │ (tx hash)
       │                       │                        │
       │ ◄──────────────────── │ (protected content)    │
       │                       │                        │
```

## Prerequisites

- **Node.js** 18+ and npm/pnpm
- **MetaMask** browser extension
- **Omni-Executor** running locally or accessible via URL
- **Smart Wallet Contracts** deployed on Arbitrum Sepolia (or target network)
- Some **ETH on Arbitrum Sepolia** for gas and testing

## Installation

1. **Navigate to the demo directory**:
   ```bash
   cd ~/workspace/heima/tee-worker/omni-executor/aa-contracts/x402-demo-app
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Configure environment variables**:
   ```bash
   cp .env.example .env
   ```

   Edit `.env` with your configuration:
   ```bash
   # Server configuration
   PORT=3001

   # Omni-Executor configuration (JSON-RPC endpoint)
   OMNI_EXECUTOR_URL=http://localhost:8080

   # Chain configuration (Arbitrum Sepolia)
   CHAIN_ID=421614
   RPC_URL=https://sepolia-rollup.arbitrum.io/rpc

   # Contract addresses
   ENTRYPOINT_ADDRESS=0x0000000071727De22E5E9d8BAf0edAc6f37da032
   FACTORY_ADDRESS=0x... # Your deployed OmniAccountFactory address
   PAYMASTER_ADDRESS=0x0000000000000000000000000000000000000000

   # Payment configuration
   PAYMENT_AMOUNT_ETH=0.00001
   SERVER_WALLET_ADDRESS=0x... # Address to receive payments

   # Client ID
   CLIENT_ID=wildmeta
   ```

## Running the Demo

### 1. Start Omni-Executor

Make sure omni-executor is running and accessible:

```bash
cd ~/workspace/heima/tee-worker/omni-executor
cargo build --release
./target/release/omni-executor
```

The RPC server should be listening on port 8080 (or as configured).

### 2. Start the Demo Server

```bash
npm start
```

Or for development with auto-reload:
```bash
npm run dev
```

The server will start on `http://localhost:3001`.

### 3. Access the Demo

Open your browser and navigate to:
```
http://localhost:3001
```

## Usage Guide

### Step 1: Connect Wallet

1. Click the **"Connect MetaMask"** button
2. Approve the connection in MetaMask
3. The app will automatically switch to Arbitrum Sepolia network (or add it if not present)

### Step 2: View Account Information

After connecting, you'll see:
- Your **EOA (Externally Owned Account)** address from MetaMask
- Your **Smart Wallet (Counterfactual)** address - this is your ERC-4337 account

### Step 3: Request Protected Content

1. Click **"Request X402-Protected Content"**
2. The app will:
   - Make an initial request to `/x402` endpoint
   - Receive a 402 Payment Required response
   - Construct a UserOperation for payment (0.00001 ETH)
   - Prompt you to sign in MetaMask
3. **Sign the transaction** in MetaMask when prompted
4. Wait for verification and settlement
5. View the protected content and transaction hash

## Technical Details

### Client-Side (Browser)

- **Configuration Loading**: Fetches config from `/config` endpoint to get contract addresses
- **Wallet Connection**: Uses MetaMask's `eth_requestAccounts`
- **OmniAccount Calculation**: SHA256 hash of `clientId + "evm" + address`
- **Counterfactual Address**: Calls `OmniAccountFactory.getAddress()` on-chain to get actual smart wallet address
- **Nonce Fetching**: Calls `OmniAccount.getNonce()` on-chain (returns 0 if not deployed)
- **UserOp Construction**: Builds PackedUserOperation with:
  - `sender`: Smart wallet address from factory
  - `nonce`: Fetched from chain
  - `callData`: Execute function to transfer ETH
  - Gas parameters with reasonable defaults for Arbitrum Sepolia
- **EIP-712 Signing**: Signs UserOperation using `eth_signTypedData_v4`
- **X-PAYMENT Header**: Encodes signed UserOp as base64 JSON

### Server-Side (Express)

- **402 Response**: Returns payment requirements on initial request
- **JSON-RPC Calls**: Communicates with omni-executor via:
  - `omni_verifyUserOp`: Simulates UserOp without submitting
  - `omni_settleUserOp`: Submits UserOp to blockchain
- **Content Delivery**: Returns protected content with tx hash after settlement

### Omni-Executor (Rust)

Two new JSON-RPC methods added:

#### `omni_verifyUserOp`
- **Purpose**: Verify a signed UserOperation by simulating it
- **Method**: `omni_verifyUserOp`
- **Params**:
  ```json
  {
    "user_operation": SerializablePackedUserOperation,
    "chain_id": number
  }
  ```
- **Returns**:
  ```json
  {
    "valid": boolean,
    "message": string
  }
  ```
- **Implementation**: Calls `EntryPoint.simulateHandleOps()` to validate

#### `omni_settleUserOp`
- **Purpose**: Submit a verified UserOperation to the blockchain
- **Method**: `omni_settleUserOp`
- **Params**:
  ```json
  {
    "user_operation": SerializablePackedUserOperation,
    "chain_id": number
  }
  ```
- **Returns**:
  ```json
  {
    "transaction_hash": string,
    "message": string
  }
  ```
- **Implementation**:
  1. Simulates UserOp first
  2. Calls `EntryPoint.handleOps()` with retry logic
  3. Returns transaction hash

## Configuration

### Supported Networks

By default configured for **Arbitrum Sepolia** (Chain ID: 421614):
- RPC URL: `https://sepolia-rollup.arbitrum.io/rpc`
- Block Explorer: `https://sepolia.arbiscan.io/`

To use a different network, update the `.env` file with appropriate values.

### Contract Requirements

The following contracts must be deployed:
1. **EntryPoint v0.7**: Standard ERC-4337 EntryPoint
2. **OmniAccountFactory**: Factory for creating OmniAccount smart wallets
3. **Paymaster** (optional): For sponsored transactions

## Development

### Project Structure

```
x402-demo-app/
├── server.js              # Express backend server
├── package.json           # Node.js dependencies
├── .env.example          # Environment configuration template
├── README.md             # This file
└── public/
    ├── index.html        # Main HTML page
    ├── aa-utils.js       # Account Abstraction utilities
    └── app.js            # Frontend application logic
```

### Adding New Features

**Custom Payment Amounts**: Modify `PAYMENT_AMOUNT_ETH` in `.env`

**Different Payment Tokens**: Update UserOp construction in `aa-utils.js` to use ERC20 transfers

**Paymaster Integration**: Add `paymasterAndData` to UserOp construction

**Add New Contract Calls**: Place contract ABIs in `public/abis/` directory and use `eth_call` via MetaMask

## Troubleshooting

### MetaMask Issues

**"User rejected the request"**: User cancelled the signature prompt
- Solution: Try again and approve in MetaMask

**Network not switching**:
- Solution: Manually add Arbitrum Sepolia to MetaMask

### Verification Failures

**"UserOp simulation failed"**:
- Check that smart wallet has sufficient ETH balance
- Verify contract addresses in `.env`
- Check gas parameters are reasonable for the network

### Settlement Failures

**"Failed to submit UserOp"**:
- Check omni-executor has ETH for bundler operations
- Verify EntryPoint client is configured for the chain
- Check network connectivity

### Common Errors

**"No EntryPoint client configured for chain_id"**:
- Ensure omni-executor is configured with EntryPoint for the target chain

**"Signature is required"**:
- UserOp must be signed before verification/settlement
- Check that signing in MetaMask completed successfully

## Security Considerations

This is a **prototype demonstration** and not production-ready. Consider:

- ✅ UserOps are signed by the client (non-custodial)
- ✅ Server verifies before settling (no double-spend)
- ✅ Uses standard ERC-4337 EntryPoint
- ⚠️ No authentication or rate limiting on endpoints
- ⚠️ Payment amounts are not validated against actual content cost
- ⚠️ No replay protection beyond nonce management
- ⚠️ No HTTPS/TLS in demo setup

## Future Enhancements

- [ ] Support for ERC20 token payments
- [ ] Paymaster integration for gasless transactions
- [ ] Content access control and authentication
- [ ] Payment history and receipt generation
- [ ] Multi-chain support
- [ ] Batch payments for multiple content items
- [ ] Integration with actual x402 protocol libraries

## References

- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337)
- [HTTP 402 Payment Required](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/402)
- [X402 Protocol](https://github.com/anthropics/x402)
- [Arbitrum Sepolia](https://docs.arbitrum.io/for-devs/concepts/public-chains#arbitrum-sepolia)
- [MetaMask Documentation](https://docs.metamask.io/)

## License

Copyright 2020-2024 Trust Computing GmbH

Licensed under the GNU General Public License v3.0
