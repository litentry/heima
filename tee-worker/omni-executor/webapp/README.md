# Webapp - Frontend Applications

This directory contains all frontend web applications for the Heima Network.

## Structure

```
webapp/
├── aa-demo/              # Account Abstraction demo (Next.js)
├── x402-demo/            # X402 protocol demo (Express + vanilla HTML)
└── privacy-invoice-demo/ # Privacy-preserving invoice demo (vanilla HTML)
```

## Applications

### 1. aa-demo (Account Abstraction Demo)

**Technology:** Next.js 16, TypeScript, TailwindCSS
**Location:** `webapp/aa-demo/`
**Purpose:** Full-featured demo of ERC-4337 Account Abstraction with smart wallets

**Features:**
- Passkey authentication
- Smart wallet creation and management
- Gasless transactions via paymaster
- ERC-20 token transfers
- Integration with EntryPoint and OmniAccountFactory

**Setup:**
```bash
cd webapp/aa-demo
pnpm install
pnpm dev
```

**Contract Sync:**
```bash
# From contracts/aa directory
./sync-abis-from-deployment.sh
./update-demo-addresses.sh
```

### 2. x402-demo (X402 Protocol Demo)

**Technology:** Express, vanilla HTML/CSS, Ethers.js
**Location:** `webapp/x402-demo/`
**Purpose:** Demo of X402 protocol for decentralized authentication and payments

**Features:**
- MetaMask integration
- Arbitrum Sepolia testnet support
- Account Abstraction (ERC-4337)
- User operation submission
- Direct TEE Worker RPC communication

**Setup:**
```bash
cd webapp/x402-demo
npm install
npm start
```

**Access:** http://localhost:3001

### 3. privacy-invoice-demo (Confidential Invoice Demo)

**Technology:** Vanilla HTML/CSS, Ethers.js, TailwindCSS
**Location:** `webapp/privacy-invoice-demo/`
**Purpose:** Privacy-preserving invoice payments using TEE encryption

**Features:**
- TEE-encrypted invoice amounts (AES-256-GCM)
- Commitment scheme for privacy verification
- MetaMask wallet integration
- Direct WebSocket RPC calls to TEE Worker
- Only buyer and seller can see actual amounts

**Files:**
- `create-invoice.html` - Seller interface for creating encrypted invoices
- `pay-invoice.html` - Buyer interface for viewing and paying invoices

**Setup:**
- No build step required (vanilla HTML)
- Serve via any static file server or open directly in browser
- Configure TEE Worker RPC URL in HTML files (default: ws://localhost:2004)

**TEE Worker RPC Methods:**
- `omni_createConfidentialInvoice` - Create encrypted invoice
- `omni_getInvoiceDetails` - View invoice (decrypt for authorized users)
- `omni_payConfidentialInvoice` - Prepare payment information

## Related Documentation

- **Contracts:** `tee-worker/omni-executor/contracts/aa/README.md`
- **ABI Sync:** `tee-worker/omni-executor/contracts/aa/ABI-SYNC.md`
- **Deployment:** `tee-worker/omni-executor/contracts/aa/DEPLOYMENT.md`
- **Privacy Invoice Progress:** `.claude/progress-5.md`

## Development Workflow

### For aa-demo:

1. Deploy contracts:
   ```bash
   cd contracts/aa
   ./deploy-local.sh
   ```

2. Sync ABIs and addresses:
   ```bash
   ./update-demo-addresses.sh  # Automatically runs sync-abis-from-deployment.sh
   ```

3. Start frontend:
   ```bash
   cd ../../webapp/aa-demo
   pnpm dev
   ```

### For x402-demo:

1. Ensure TEE worker is running
2. Start Express server:
   ```bash
   cd webapp/x402-demo
   npm start
   ```

### For privacy-invoice-demo:

1. Ensure TEE worker is running with invoice RPC methods
2. Open HTML files directly in browser or serve via static server
3. Test workflow:
   - Seller: Open `create-invoice.html`
   - Buyer: Open `pay-invoice.html?id=<invoice_id>`

## Architecture Notes

- **Frontend → TEE Worker:** Direct WebSocket/HTTP RPC calls for privacy-critical operations
- **Frontend → Blockchain:** Direct calls via Ethers.js/Alloy for on-chain operations
- **Smart Contracts:** Located in `contracts/aa/src/`
- **Backend Logic:** TEE Worker handles encryption, signing, and sensitive operations

## Port Mapping

- **aa-demo:** http://localhost:3000
- **x402-demo:** http://localhost:3001
- **TEE Worker RPC:** ws://localhost:2004 (WebSocket) or http://localhost:2004 (HTTP)
- **Anvil (local chain):** http://localhost:8545
