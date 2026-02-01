# Privacy-Preserving Invoice Demo Script

**Target Time**: 60 seconds
**Objective**: Demonstrate TEE-encrypted invoice creation, viewing, and privacy verification

---

## Pre-Demo Setup (5 minutes)

### 1. Start TEE Worker

```bash
cd /home/kai/workspace/heima/tee-worker/omni-executor
make

# Wait for:
# ✅ RPC server started on ws://0.0.0.0:2004
# ✅ Confidential invoice storage initialized
```

### 2. Start Frontend Server

```bash
cd /home/kai/workspace/heima/tee-worker/omni-executor/webapp/privacy-invoice-demo
python3 -m http.server 8080

# Or use npm start
```

### 3. Prepare Browser Tabs

Open two browser windows/tabs:
- **Tab 1 (Seller)**: http://localhost:8080/create-invoice.html
- **Tab 2 (Buyer)**: http://localhost:8080/pay-invoice.html (ready to paste invoice ID)

### 4. MetaMask Setup

- Install MetaMask browser extension
- Have test wallet ready (any address works for demo)
- Arbitrum Sepolia will be auto-added by the UI

---

## Demo Script (60 seconds)

### Act 1: The Problem (10 seconds)

**Narration**:
> "Traditional blockchain transactions are completely transparent. Every payment amount is visible to anyone with a block explorer. This is a problem for B2B invoicing where confidentiality is critical."

**Show**:
- Open Arbiscan
- Point to any transaction showing visible ETH/token amounts
- Highlight that competitors, customers, everyone can see business transactions

---

### Act 2: Creating a Confidential Invoice (15 seconds)

**Narration**:
> "With Heima's privacy-preserving invoices, we solve this using TEE encryption. Watch as I create a confidential invoice."

**Actions** (Seller Tab):

1. **Fill Form** (5 sec):
   ```
   Amount: 50000.00
   Buyer Email: buyer@acmecorp.com
   Description: Q4 2025 Enterprise License
   Seller: Heima Labs
   ```

2. **Click "Generate Confidential Invoice"** (2 sec)
   - Show loading state: "Encrypting..."

3. **Result Appears** (3 sec):
   - Invoice ID: `inv_550e8400-...`
   - Payment Link: `http://localhost:8080/pay-invoice.html?id=inv_...`
   - Commitment: `0x1234abcd...` (hash for verification)
   - **Highlight**: "Amount Encrypted" badge with lock icon

4. **Copy Link** (2 sec):
   - Click "Copy Link" button
   - Show "Copied!" confirmation

5. **Behind the Scenes** (3 sec):
   - Show terminal/logs (optional):
     ```
     INFO Created confidential invoice: inv_550e8400-...
     Amount: 50000.00 USDC → Encrypted with AES-256-GCM
     Commitment: 0x1234abcd... (SHA256 hash)
     Stored in RocksDB: ✓
     ```

---

### Act 3: Buyer Views Invoice (20 seconds)

**Narration**:
> "The buyer receives this link and can view the invoice. Only they can see the actual amount - it's decrypted in the TEE."

**Actions** (Buyer Tab):

1. **Paste Link & Load** (2 sec):
   - Paste copied URL or manually add `?id=inv_...`
   - Page shows "Connect MetaMask" button

2. **Connect MetaMask** (8 sec):
   - Click "Connect MetaMask"
   - MetaMask popup appears
   - Select account and approve
   - Auto-switch to Arbitrum Sepolia (if needed):
     ```
     Network: Arbitrum Sepolia
     Chain ID: 421614
     ```

3. **Invoice Details Revealed** (5 sec):
   - Show loading: "Loading invoice details..."
   - Amount appears: **$50,000.00**
   - Privacy indicator: "🔒 Only you and the seller can see this amount"
   - Other details:
     ```
     Description: Q4 2025 Enterprise License
     Currency: USDC
     Status: Pending
     Created: Feb 1, 2026
     Network: Arbitrum Sepolia
     ```

4. **Highlight Privacy** (3 sec):
   - Point to purple privacy box:
     > "Only you and the seller can see this amount"
   - Show commitment hash matches

5. **Payment Ready** (2 sec):
   - "Pay with Smart Wallet" button ready
   - (Note: Full payment integration coming in next phase)

---

### Act 4: Privacy Verification (15 seconds)

**Narration**:
> "The magic happens when you verify the privacy on the blockchain. Let me show you what the blockchain sees."

**Actions**:

1. **Simulate On-Chain Storage** (5 sec):
   - Show terminal with commitment storage:
     ```bash
     # What's stored on-chain (after payment):
     Commitment: 0x1234abcd...
     Status: Paid
     Timestamp: 1738368000

     # What's NOT on-chain:
     ❌ Amount (encrypted in TEE)
     ❌ Actual payment value
     ```

2. **Compare with Traditional Blockchain** (5 sec):
   - Split screen:
     ```
     Traditional Transaction:      Privacy Invoice:
     ✓ Amount: 50000 USDC         ✓ Commitment: 0x1234...
     ✓ Sender: 0xabc...            ✓ Status: Paid
     ✓ Receiver: 0xdef...          ❌ Amount: HIDDEN
     ❌ Everyone can see            ✓ Only parties can see
     ```

3. **Key Benefits** (5 sec):
   - Bullet points appear:
     ✅ Privacy from competitors
     ✅ Regulatory compliance (confidential B2B)
     ✅ Reduced market manipulation risk
     ✅ Same blockchain security guarantees

---

## Quick Demo Version (30 seconds)

For time-constrained demos, focus on:

1. **Problem** (5s): "Blockchain is transparent, B2B invoices need privacy"
2. **Create Invoice** (10s): Fill form → Generate → Show encrypted badge
3. **Buyer View** (10s): Connect wallet → Amount revealed → Privacy indicator
4. **Privacy Proof** (5s): "Amount hidden on-chain, only commitment visible"

---

## Demo Talking Points

### Technical Highlights

- **TEE Encryption**: AES-256-GCM in trusted execution environment
- **Zero-Knowledge Commitment**: SHA256 hash proves amount without revealing it
- **Direct Communication**: Frontend → TEE Worker (no server middleman)
- **ERC-4337**: Account abstraction for gasless payments (coming soon)

### Business Value

- **B2B Confidentiality**: Enterprise customers demand privacy
- **Regulatory Compliance**: GDPR, financial privacy requirements
- **Competitive Advantage**: Hide deal sizes from competitors
- **Trust**: Blockchain security + privacy

### Privacy Guarantees

- ✅ Amount encrypted at rest (RocksDB)
- ✅ Amount encrypted in transit (WebSocket → will use TLS/WSS)
- ✅ Amount never touches blockchain (only commitment)
- ✅ Server cannot decrypt (TEE-only key)
- ❌ Invoice metadata public (ID, description, currency)
- 🔄 Access control via JWT (coming in production)

---

## Troubleshooting During Demo

### TEE Worker Not Responding
```bash
# Quick restart
cd /home/kai/workspace/heima/tee-worker/omni-executor
make clean && make
```

### MetaMask Not Connecting
- Check browser console (F12)
- Refresh page
- Try different MetaMask account

### Invoice Not Found
- Verify invoice ID in URL
- Check TEE Worker logs
- Ensure storage directory exists

### Network Issues
- Manually add Arbitrum Sepolia:
  - Chain ID: 421614
  - RPC: https://sepolia-rollup.arbitrum.io/rpc

---

## Post-Demo Q&A Preparation

### Expected Questions

**Q: Can the server see the amount?**
> A: No. The encryption happens inside the TEE (trusted execution environment). The server only stores encrypted bytes and cannot access the decryption key.

**Q: What if the TEE is compromised?**
> A: TEE (Intel SGX) provides hardware-level isolation with remote attestation. Even if the server is compromised, the TEE remains secure. For production, we verify attestation quotes.

**Q: How does the buyer prove they paid the right amount?**
> A: The commitment hash on-chain matches the hash of (invoice_id || amount). The buyer can prove to auditors they paid correctly without revealing the amount publicly.

**Q: What about gas fees?**
> A: We use ERC-4337 Account Abstraction with paymasters for gasless transactions. The demo focuses on privacy; payment integration comes next.

**Q: Is this compliant with regulations?**
> A: Yes. Privacy doesn't mean opacity. Authorized parties (buyer, seller, auditors with proper authorization) can access amounts. It's transparent to participants, private to the public.

**Q: Can I use this on mainnet?**
> A: Currently Arbitrum Sepolia testnet. Production deployment requires:
> - Mainnet TEE deployment with attestation
> - TLS/WSS for encrypted connections
> - JWT authentication for access control
> - Mainnet contract deployment

---

## Success Metrics

Demo is successful if audience understands:

1. ✅ **Privacy Problem**: Blockchain transparency is bad for B2B
2. ✅ **TEE Solution**: Encryption without trusted third party
3. ✅ **Workflow**: Simple UI for creating and viewing invoices
4. ✅ **Verification**: Amount hidden on-chain, only commitment visible
5. ✅ **Business Value**: Real-world use case for enterprise adoption

---

## Next Steps to Show

After demo, mention:

1. **Phase 3 Completion**:
   - Payment integration with `omni_settleUserOp`
   - USDC token transfers
   - Transaction confirmations

2. **Production Roadmap**:
   - SGX attestation for mainnet
   - JWT authentication
   - Email notifications
   - Invoice expiration
   - Multi-currency support

3. **Developer Adoption**:
   - Open-source SDK
   - API documentation
   - Integration examples
   - Developer sandbox

---

## Recording Tips

For video demos:

1. **Use Screen Recording**:
   - QuickTime (Mac): Cmd+Shift+5
   - OBS Studio (Cross-platform)
   - Resolution: 1920x1080 minimum

2. **Split Screen Layout**:
   - Left: Seller (create-invoice.html)
   - Right: Buyer (pay-invoice.html)
   - Bottom: Terminal (TEE Worker logs)

3. **Zoom In on Important Parts**:
   - "Amount Encrypted" badge
   - Privacy indicator
   - Commitment hash
   - MetaMask connection

4. **Use Annotations**:
   - Highlight key features with circles/arrows
   - Add text overlays for technical details
   - Use slow-motion for encryption process

5. **Background Music**:
   - Subtle, non-distracting
   - Pause during narration
   - Upbeat for demo, serious for problem intro

---

## Demo Variations

### Technical Audience (90 seconds)
- Show code: confidential_invoice.rs
- Explain AES-256-GCM vs other schemes
- Demonstrate commitment verification
- Discuss TEE attestation

### Business Audience (45 seconds)
- Focus on use case (B2B invoicing)
- Emphasize privacy value
- Show simple UI only
- Skip technical details

### Investor Pitch (30 seconds)
- Problem: $X billion B2B market needs privacy
- Solution: TEE-encrypted invoices
- Demo: Quick create → view → verify
- Traction: Ready for enterprise pilots
