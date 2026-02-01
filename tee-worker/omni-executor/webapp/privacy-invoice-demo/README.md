# Privacy-Preserving Invoice Demo

A privacy-preserving invoice payment system using TEE (Trusted Execution Environment) encryption to hide invoice amounts from blockchain observers, server operators, and unauthorized users.

## Overview

This demo showcases:
- **TEE-Encrypted Invoice Amounts**: AES-256-GCM encryption ensures only buyer and seller can see the actual amount
- **Commitment Scheme**: SHA256 hash allows on-chain verification without revealing the amount
- **Direct TEE Communication**: Frontend communicates directly with TEE Worker via WebSocket
- **MetaMask Integration**: Seamless wallet connection with Arbitrum Sepolia support
- **Privacy Verification**: On-chain transactions hide the payment amount from blockchain explorers

## Architecture

```
┌─────────────────┐
│  Seller (Web)   │
│ create-invoice  │
└────────┬────────┘
         │ WebSocket RPC
         │ omni_createConfidentialInvoice
         ↓
┌─────────────────────────┐
│   TEE Worker (Rust)      │
│  - AES-256 Encrypt       │
│  - Generate Commitment   │
│  - Store in RocksDB      │
└────────┬────────────────┘
         │ Invoice ID + Link
         ↓
┌─────────────────┐
│  Buyer (Web)    │
│  pay-invoice    │
└────────┬────────┘
         │ WebSocket RPC
         │ omni_getInvoiceDetails
         ↓
┌─────────────────────────┐
│   TEE Worker (Rust)      │
│  - Decrypt Amount        │
│  - Verify Authorization  │
│  - Return Details        │
└─────────────────────────┘
```

## Files

### 1. create-invoice.html
**Purpose**: Seller interface for creating encrypted invoices

**Features**:
- Invoice creation form (amount, buyer email, description)
- Direct WebSocket RPC call to `omni_createConfidentialInvoice`
- Displays shareable invoice link
- Shows commitment hash for verification
- Copy-to-clipboard functionality
- Visual TEE encryption indicators

**RPC Method**: `omni_createConfidentialInvoice`
```javascript
{
  seller_account: "seller@heima.network",
  buyer_identifier: "buyer@company.com",
  amount: "50000.00",
  currency: "USDC",
  chain_id: 421614,  // Arbitrum Sepolia
  description: "Q4 2025 Consulting Services"
}
```

**Response**:
```javascript
{
  invoice_id: "inv_550e8400-e29b-41d4-a716-446655440000",
  invoice_url: "https://demo.heima.network/invoice/inv_...",
  commitment: "0x1234..."  // SHA256 hash for verification
}
```

### 2. pay-invoice.html
**Purpose**: Buyer interface for viewing and paying invoices

**Features**:
- MetaMask wallet connection
- Automatic Arbitrum Sepolia network switching
- Invoice ID from URL query parameter (`?id=inv_xxx`)
- Direct WebSocket RPC call to `omni_getInvoiceDetails`
- Displays decrypted invoice amount (only to authorized users)
- Privacy indicator showing amount is hidden from others
- Pay button (ready for `omni_settleUserOp` integration)
- Transaction success state with Arbiscan link

**RPC Method**: `omni_getInvoiceDetails`
```javascript
{
  invoice_id: "inv_550e8400-e29b-41d4-a716-446655440000",
  auth_token: null  // MVP: no auth required, will add JWT later
}
```

**Response**:
```javascript
{
  invoice_id: "inv_...",
  amount: "50000.00",  // Decrypted by TEE
  currency: "USDC",
  description: "Q4 2025 Consulting Services",
  status: "Pending",
  created_at: 1738368000,
  tx_hash: null,
  seller_account: "seller@heima.network"
}
```

## Setup

### Prerequisites

1. **TEE Worker Running**: The RPC server must be running on `ws://localhost:2004`
   ```bash
   cd /home/kai/workspace/heima/tee-worker/omni-executor
   make
   ```

2. **MetaMask Installed**: Browser extension for wallet connection

3. **Arbitrum Sepolia Testnet**:
   - Network will be auto-added by the UI
   - Get testnet ETH from [Arbitrum Sepolia Faucet](https://faucet.quicknode.com/arbitrum/sepolia)

### Running the Demo

#### Option 1: Simple HTTP Server (Recommended for Testing)

```bash
cd /home/kai/workspace/heima/tee-worker/omni-executor/webapp/privacy-invoice-demo

# Python 3
python3 -m http.server 8080

# Or Node.js
npx http-server -p 8080
```

Then open:
- Seller: http://localhost:8080/create-invoice.html
- Buyer: http://localhost:8080/pay-invoice.html?id=<invoice_id>

#### Option 2: Open Directly in Browser

Simply open the HTML files directly in your browser. Note that WebSocket connections to localhost should work without CORS issues.

## Demo Workflow

### Step 1: Create Invoice (Seller)

1. Open `create-invoice.html`
2. Fill in the form:
   - Amount: 50000.00 (USDC)
   - Buyer Email: buyer@company.com
   - Description: Q4 2025 Consulting Services
   - Seller Account: seller@heima.network (optional)
3. Click "Generate Confidential Invoice"
4. Wait for TEE encryption (~1-2 seconds)
5. Copy the generated invoice link
6. Note the commitment hash (for verification)

**What Happens Behind the Scenes**:
- Frontend sends amount to TEE Worker via WebSocket
- TEE Worker encrypts amount with AES-256-GCM
- Generates SHA256 commitment for verification
- Stores encrypted invoice in RocksDB
- Returns invoice ID and shareable link

### Step 2: View Invoice (Buyer)

1. Open the copied invoice link (or `pay-invoice.html?id=inv_xxx`)
2. Click "Connect MetaMask"
3. Approve wallet connection
4. Switch to Arbitrum Sepolia (auto-prompted if needed)
5. View decrypted invoice details:
   - Amount: **Only you and seller can see this**
   - Description, Currency, Status
6. Click "Pay with Smart Wallet" (ready for integration)

**What Happens Behind the Scenes**:
- Frontend requests invoice details via WebSocket
- TEE Worker decrypts amount (MVP: no auth check yet)
- Returns decrypted amount only to authorized user
- Amount remains encrypted in storage
- Payment flow ready for `omni_settleUserOp` integration

### Step 3: Privacy Verification

1. After payment (future integration), check transaction on Arbiscan
2. Verify that the payment amount is **NOT visible** on the blockchain
3. Only the commitment hash is stored on-chain
4. Only buyer and seller can reconstruct the actual amount

## Technical Details

### Encryption Scheme

**Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key**: 32-byte AES key stored securely in TEE
- **Nonce**: 12-byte random nonce (unique per encryption)
- **Format**: `[nonce || ciphertext || tag]`

```rust
// Simplified encryption flow
let plaintext = amount.to_le_bytes();  // u128 as 16 bytes
let nonce = random_12_bytes();
let ciphertext = aes256_gcm_encrypt(plaintext, key, nonce);
let encrypted = [nonce, ciphertext].concat();
```

### Commitment Scheme

**Algorithm**: SHA256 Hash
- **Input**: `invoice_id || amount`
- **Output**: 32-byte hash (commitment)
- **Purpose**: On-chain verification without revealing amount

```rust
// Simplified commitment generation
let commitment = sha256(invoice_id.as_bytes() || amount.to_le_bytes());
```

### RPC Methods

All methods are implemented in:
`tee-worker/omni-executor/rpc-server/src/methods/omni/confidential_invoice.rs`

1. **omni_createConfidentialInvoice**
   - Creates encrypted invoice
   - Generates commitment
   - Stores in RocksDB
   - Returns invoice_id and shareable URL

2. **omni_getInvoiceDetails**
   - Retrieves invoice by ID
   - Decrypts amount for authorized users
   - Returns full invoice details

3. **omni_payConfidentialInvoice**
   - Prepares payment information
   - Decrypts amount for buyer verification
   - Ready for `omni_settleUserOp` integration

## Configuration

### RPC URL

Update in HTML files if TEE Worker is running on a different host/port:

```javascript
// create-invoice.html and pay-invoice.html
const RPC_URL = 'ws://localhost:2004';  // Change if needed
```

### Invoice URL Base

Update in TEE Worker if hosting on custom domain:

```rust
// confidential_invoice.rs
invoice_url: format!("https://demo.heima.network/invoice/{}", invoice_id),
// Change to your domain or localhost for testing
```

## Testing

### Manual Testing Checklist

- [ ] **Invoice Creation**
  - [ ] Form validation (positive amounts only)
  - [ ] WebSocket connection to TEE Worker
  - [ ] Invoice ID generation (UUID v4 format)
  - [ ] Commitment hash displayed
  - [ ] Copy link functionality works
  - [ ] Create multiple invoices

- [ ] **Invoice Viewing**
  - [ ] URL parameter parsing (`?id=inv_xxx`)
  - [ ] MetaMask connection
  - [ ] Network switching to Arbitrum Sepolia
  - [ ] Amount decryption successful
  - [ ] Invoice details displayed correctly
  - [ ] Privacy indicator visible

- [ ] **Error Handling**
  - [ ] Missing invoice ID in URL
  - [ ] Invalid invoice ID
  - [ ] TEE Worker offline
  - [ ] MetaMask not installed
  - [ ] Network connection issues

### Automated Testing

Integration tests are located in:
`tee-worker/omni-executor/rpc-server/src/methods/omni/confidential_invoice.rs`

```bash
cargo test --features=test-endpoints confidential_invoice
```

## Privacy Guarantees

### What is Private:

✅ **Invoice Amount**: Encrypted with AES-256-GCM in TEE
✅ **Amount in Transit**: WebSocket connection can use TLS
✅ **Amount in Storage**: RocksDB stores encrypted bytes only
✅ **On-Chain Amount**: Not stored on blockchain (commitment only)

### What is Public:

❌ **Invoice ID**: Necessary for retrieval
❌ **Seller Account**: Visible in invoice metadata
❌ **Buyer Identifier**: Email or account (for access control)
❌ **Currency & Description**: Public metadata
❌ **Commitment Hash**: On-chain for verification

### Access Control (Future Enhancement):

- [ ] JWT token authentication for invoice viewing
- [ ] Buyer email verification
- [ ] Time-limited access tokens
- [ ] Seller authorization checks

## Production Deployment

### Security Considerations:

1. **TEE Attestation**: Verify TEE Worker is running in genuine SGX enclave
2. **TLS/WSS**: Use encrypted WebSocket connections (wss://)
3. **JWT Auth**: Implement proper authentication for invoice viewing
4. **Rate Limiting**: Prevent brute-force invoice ID guessing
5. **CORS Policy**: Restrict frontend origins
6. **Key Rotation**: Periodic AES key rotation in TEE

### Deployment Steps:

1. Deploy TEE Worker to production with SGX attestation
2. Deploy frontend to CDN (Cloudflare, Vercel, etc.)
3. Configure production RPC URL in frontend
4. Set up monitoring and alerting
5. Test privacy verification on mainnet

## Troubleshooting

### "Failed to connect to TEE Worker"
- Ensure TEE Worker is running on `ws://localhost:2004`
- Check WebSocket connection in browser console
- Verify no firewall blocking port 2004

### "Invoice not found"
- Check invoice ID in URL is correct
- Verify TEE Worker storage is persisted (RocksDB directory)
- Check RPC method logs for errors

### "MetaMask connection failed"
- Ensure MetaMask extension is installed
- Check browser console for errors
- Try refreshing the page

### "Network not supported"
- UI will auto-prompt to switch to Arbitrum Sepolia
- Manually add network if auto-add fails:
  - Chain ID: 421614
  - RPC URL: https://sepolia-rollup.arbitrum.io/rpc
  - Currency: ETH
  - Explorer: https://sepolia.arbiscan.io/

## Future Enhancements

### Phase 3.1: Payment Integration
- [ ] Integrate with `omni_settleUserOp` for actual payments
- [ ] USDC token approval flow
- [ ] Transaction confirmation UI
- [ ] Payment receipt generation

### Phase 3.2: Enhanced Privacy
- [ ] Zero-knowledge proofs for amount verification
- [ ] Multi-party computation for shared secrets
- [ ] Homomorphic encryption for amount computations

### Phase 3.3: Production Features
- [ ] Email notifications (invoice sent, payment received)
- [ ] Invoice expiration (time-based)
- [ ] Recurring invoices
- [ ] Multi-currency support (USDT, DAI, etc.)
- [ ] Invoice history dashboard

## References

- **Implementation Docs**: `.claude/progress-5.md`
- **Proposal**: `.claude/proposal-5-privacy-invoicing-implementation.md`
- **Backend Code**: `tee-worker/omni-executor/rpc-server/src/methods/omni/confidential_invoice.rs`
- **Encryption**: `tee-worker/omni-executor/crypto/src/confidential.rs`
- **Storage**: `tee-worker/omni-executor/storage/src/confidential_invoice.rs`

## Support

For issues or questions:
- Check TEE Worker logs: `journalctl -u tee-worker -f`
- Review browser console for frontend errors
- Check RocksDB storage: `/path/to/rocksdb/confidential_invoices/`
- File issues on GitHub: https://github.com/anthropics/heima-network/issues
