# Privacy-Preserving Invoice - Testing Checklist

Complete testing guide for the privacy invoice demo implementation.

---

## Pre-Test Setup

### Environment Setup

- [ ] TEE Worker running on `ws://localhost:2004`
- [ ] Frontend server running on `http://localhost:8080`
- [ ] MetaMask installed in browser
- [ ] Test wallet with Arbitrum Sepolia ETH (for future payment integration)
- [ ] Browser console open (F12) for debugging

### Verification Commands

```bash
# Check TEE Worker is running
curl http://localhost:2004/health
# Expected: {"status":"ok"}

# Check frontend server
curl http://localhost:8080/
# Expected: Directory listing or 404 (both OK)

# Check RocksDB storage exists
ls -la /tmp/rocksdb_test/confidential_invoices/
# Expected: Directory exists
```

---

## Phase 1: Backend RPC Methods Testing

### Test 1: Invoice Creation

**Method**: `omni_createConfidentialInvoice`

```bash
# Using wscat or websocat
wscat -c ws://localhost:2004

# Send:
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "omni_createConfidentialInvoice",
  "params": [{
    "seller_account": "seller@test.com",
    "buyer_identifier": "buyer@test.com",
    "amount": "100.50",
    "currency": "USDC",
    "chain_id": 421614,
    "description": "Test Invoice"
  }]
}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "invoice_id": "inv_...",
    "invoice_url": "https://demo.heima.network/invoice/inv_...",
    "commitment": "0x..."
  }
}
```

**Checklist**:
- [ ] Invoice ID starts with `inv_` and is UUID v4 format
- [ ] Invoice URL contains the invoice ID
- [ ] Commitment is 64-character hex string (0x + 32 bytes)
- [ ] Response time < 2 seconds
- [ ] No errors in TEE Worker logs

---

### Test 2: Invoice Retrieval

**Method**: `omni_getInvoiceDetails`

```bash
# Use invoice_id from Test 1
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "omni_getInvoiceDetails",
  "params": [{
    "invoice_id": "inv_...",
    "auth_token": null
  }]
}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "invoice_id": "inv_...",
    "amount": "100.50",
    "currency": "USDC",
    "description": "Test Invoice",
    "status": "Pending",
    "created_at": 1738368000,
    "tx_hash": null,
    "seller_account": "seller@test.com"
  }
}
```

**Checklist**:
- [ ] Amount matches original (100.50)
- [ ] Currency is USDC
- [ ] Status is "Pending"
- [ ] created_at is Unix timestamp
- [ ] tx_hash is null (not yet paid)
- [ ] Decryption successful (no errors)

---

### Test 3: Payment Preparation

**Method**: `omni_payConfidentialInvoice`

```bash
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "omni_payConfidentialInvoice",
  "params": [{
    "invoice_id": "inv_...",
    "buyer_account": "buyer@test.com"
  }]
}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "status": "ready_to_pay",
    "message": "Invoice amount: 100.50 USDC. Please proceed with payment.",
    "amount": "100.50",
    "commitment": "0x..."
  }
}
```

**Checklist**:
- [ ] Status is "ready_to_pay"
- [ ] Amount decrypted correctly
- [ ] Commitment matches creation response
- [ ] Message is human-readable

---

### Test 4: Error Handling

**Test 4.1: Invalid Amount**
```bash
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "omni_createConfidentialInvoice",
  "params": [{
    "seller_account": "seller@test.com",
    "buyer_identifier": "buyer@test.com",
    "amount": "-100.00",  # Negative amount
    "currency": "USDC",
    "chain_id": 421614,
    "description": "Test"
  }]
}
```

**Expected**: Error with "Amount must be positive"

**Checklist**:
- [ ] Returns JSON-RPC error
- [ ] Error message is clear
- [ ] No invoice created in storage

---

**Test 4.2: Unsupported Currency**
```bash
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "omni_createConfidentialInvoice",
  "params": [{
    "seller_account": "seller@test.com",
    "buyer_identifier": "buyer@test.com",
    "amount": "100.00",
    "currency": "BTC",  # Not supported
    "chain_id": 421614,
    "description": "Test"
  }]
}
```

**Expected**: Error with "Only USDC is supported currently"

**Checklist**:
- [ ] Returns JSON-RPC error
- [ ] Error message mentions USDC only
- [ ] No invoice created

---

**Test 4.3: Non-Existent Invoice**
```bash
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "omni_getInvoiceDetails",
  "params": [{
    "invoice_id": "inv_nonexistent",
    "auth_token": null
  }]
}
```

**Expected**: Error with "Invoice not found"

**Checklist**:
- [ ] Returns JSON-RPC error
- [ ] Error message is clear
- [ ] No decryption attempt

---

## Phase 2: Frontend Testing

### Test 5: Create Invoice Page

**URL**: http://localhost:8080/create-invoice.html

**Test 5.1: Page Load**
- [ ] Page loads without errors
- [ ] TailwindCSS styles applied correctly
- [ ] Header displays "Create Confidential Invoice"
- [ ] TEE encryption badge visible
- [ ] All form fields rendered

**Test 5.2: Form Validation**
- [ ] Amount field requires positive number
- [ ] Amount accepts decimal values (50000.00)
- [ ] Buyer email requires valid email format
- [ ] Description is required
- [ ] Seller account is optional (defaults to seller@heima.network)

**Test 5.3: Invoice Creation**
1. Fill form:
   - Amount: 50000.00
   - Buyer Email: buyer@company.com
   - Description: Q4 2025 Consulting Services
   - Seller: Heima Labs

2. Click "Generate Confidential Invoice"

**Checklist**:
- [ ] Button text changes to "Encrypting..."
- [ ] Loading state visible
- [ ] Form hides after successful creation
- [ ] Result section appears with:
  - [ ] Invoice ID displayed
  - [ ] Invoice URL displayed
  - [ ] Commitment hash displayed (truncated with ...)
  - [ ] "Amount Encrypted" indicator visible
  - [ ] "Copy Link" button works
- [ ] Success animation/state shown
- [ ] No console errors

**Test 5.4: Copy Functionality**
- [ ] Click "Copy Link" button
- [ ] Button shows "Copied!" feedback
- [ ] Pasted URL is correct format
- [ ] Feedback reverts after 2 seconds

**Test 5.5: Create Another Invoice**
- [ ] Click "Create Another Invoice" button
- [ ] Form reappears
- [ ] Result section hides
- [ ] Form is reset (empty fields)

---

### Test 6: Pay Invoice Page

**URL**: http://localhost:8080/pay-invoice.html

**Test 6.1: Missing Invoice ID**
- [ ] Open without `?id=` parameter
- [ ] Error message appears: "No invoice ID provided in URL"
- [ ] No wallet connection prompt

**Test 6.2: Invalid Invoice ID**
- [ ] Open with `?id=invalid_id`
- [ ] Connect wallet (should work)
- [ ] After connection, error: "Failed to load invoice"
- [ ] Error UI shows clearly

**Test 6.3: Valid Invoice - Before Wallet Connection**
- [ ] Open with valid `?id=inv_...`
- [ ] Login state displayed:
  - [ ] Invoice icon visible
  - [ ] "Confidential Invoice Payment" header
  - [ ] "Connect MetaMask" button prominent
  - [ ] Blue gradient button style

**Test 6.4: MetaMask Connection**
1. Click "Connect MetaMask"

**Checklist**:
- [ ] MetaMask popup appears
- [ ] Account selection shown
- [ ] After approval, network check happens
- [ ] If not on Arbitrum Sepolia:
  - [ ] Auto-switch prompt appears
  - [ ] Network info correct (Chain ID: 421614)
  - [ ] After switch, invoice loads

**Test 6.5: Invoice Display**
After successful wallet connection:

- [ ] Login state hides
- [ ] Invoice state appears
- [ ] Header shows "Invoice from [Seller Name]"
- [ ] Status badge shows "Pending" (yellow)
- [ ] Amount section:
  - [ ] Shows loading initially ("Loading...")
  - [ ] Amount reveals after load
  - [ ] Font size large and prominent
  - [ ] Currency displayed (USDC)
  - [ ] Privacy indicator box:
    - [ ] Purple background
    - [ ] Lock icon
    - [ ] Text: "Only you and the seller can see this amount"
- [ ] Invoice details:
  - [ ] Invoice ID (truncated)
  - [ ] Created date (human-readable)
  - [ ] Network: Arbitrum Sepolia
- [ ] Pay button:
  - [ ] "Pay with Smart Wallet" text
  - [ ] Green gradient style
  - [ ] Enabled (not disabled)
  - [ ] Hover effect works

**Test 6.6: Pay Button (Current MVP)**
- [ ] Click "Pay with Smart Wallet"
- [ ] Button disabled during click
- [ ] Info message appears: "Payment functionality will be integrated with omni_settleUserOp in the next phase"
- [ ] Error UI styled correctly (orange/info color, not red)
- [ ] Button re-enables after message

---

### Test 7: End-to-End Workflow

**Complete flow from seller to buyer**:

1. **Seller Creates Invoice**:
   - [ ] Open create-invoice.html
   - [ ] Fill form with test data
   - [ ] Submit and get invoice URL
   - [ ] Copy invoice URL

2. **Seller Shares Invoice**:
   - [ ] Paste URL in new browser tab/window
   - [ ] Verify URL format correct

3. **Buyer Opens Invoice**:
   - [ ] Invoice loads in new tab
   - [ ] Connect MetaMask prompt appears

4. **Buyer Connects Wallet**:
   - [ ] Click connect
   - [ ] Approve MetaMask
   - [ ] Switch to Arbitrum Sepolia (if needed)

5. **Buyer Views Invoice**:
   - [ ] Amount displayed correctly
   - [ ] Matches seller's input
   - [ ] Privacy indicator shows

6. **Buyer Verifies Privacy**:
   - [ ] Amount visible to buyer
   - [ ] Commitment hash visible
   - [ ] Can verify hash matches seller's

**Time Checklist**:
- [ ] Create invoice: < 2 seconds
- [ ] Load invoice page: < 1 second
- [ ] Connect wallet: < 5 seconds (user action)
- [ ] Decrypt and display: < 2 seconds
- [ ] **Total flow: < 30 seconds** (including user actions)

---

## Phase 3: Integration Testing

### Test 8: Multiple Invoices

**Test 8.1: Create Multiple**
- [ ] Create 5 different invoices with varying amounts
- [ ] Each gets unique invoice_id
- [ ] Each has unique commitment
- [ ] All stored correctly

**Test 8.2: Retrieve Each**
- [ ] Open each invoice in separate tabs
- [ ] All load correctly
- [ ] Amounts decrypt correctly
- [ ] No cross-contamination (invoice A doesn't show invoice B's data)

**Test 8.3: Storage Verification**
```bash
# Check RocksDB storage
ls /tmp/rocksdb_test/confidential_invoices/ | wc -l
# Expected: 5 entries
```

---

### Test 9: Concurrent Access

**Test 9.1: Same Invoice, Multiple Tabs**
- [ ] Open same invoice URL in 3 different tabs
- [ ] All tabs connect wallet
- [ ] All tabs show same amount
- [ ] No race conditions

**Test 9.2: Create Multiple Simultaneously**
- [ ] Open 3 create-invoice tabs
- [ ] Submit all at same time
- [ ] All succeed with unique IDs
- [ ] No duplicate invoice_ids

---

### Test 10: Browser Compatibility

**Test on Multiple Browsers**:

- [ ] **Chrome/Chromium**:
  - [ ] Create invoice works
  - [ ] View invoice works
  - [ ] MetaMask connects
  - [ ] TailwindCSS renders correctly

- [ ] **Firefox**:
  - [ ] Create invoice works
  - [ ] View invoice works
  - [ ] MetaMask connects
  - [ ] Styles correct

- [ ] **Safari** (if on Mac):
  - [ ] Create invoice works
  - [ ] View invoice works
  - [ ] MetaMask connects
  - [ ] WebSocket connection works

- [ ] **Edge**:
  - [ ] Create invoice works
  - [ ] View invoice works
  - [ ] MetaMask connects

---

### Test 11: Network Conditions

**Test 11.1: Slow Network**
```bash
# Simulate slow network (Linux)
sudo tc qdisc add dev lo root netem delay 500ms

# Run tests
# ...

# Remove delay
sudo tc qdisc del dev lo root
```

**Checklist**:
- [ ] Loading states appear
- [ ] Timeouts handled gracefully
- [ ] No frozen UI
- [ ] Error messages if timeout

**Test 11.2: Offline TEE Worker**
```bash
# Stop TEE Worker
pkill -f omni-executor
```

**Checklist**:
- [ ] Create invoice shows error
- [ ] View invoice shows error
- [ ] Error messages are clear
- [ ] No console errors (handled gracefully)

---

### Test 12: Edge Cases

**Test 12.1: Very Large Amount**
- [ ] Create invoice with amount: 999999999.99
- [ ] Amount encrypts successfully
- [ ] Amount decrypts correctly
- [ ] Display format correct (commas, decimals)

**Test 12.2: Very Small Amount**
- [ ] Create invoice with amount: 0.01
- [ ] Amount encrypts successfully
- [ ] Amount decrypts correctly
- [ ] Display shows "0.01" not "0.00"

**Test 12.3: Zero Amount**
- [ ] Create invoice with amount: 0
- [ ] Error: "Amount must be positive"
- [ ] No invoice created

**Test 12.4: Non-Numeric Amount**
- [ ] Create invoice with amount: "abc"
- [ ] Browser validation prevents submission
- [ ] Or backend error if bypassed

**Test 12.5: Special Characters in Description**
- [ ] Create invoice with description: `<script>alert('xss')</script>`
- [ ] HTML properly escaped
- [ ] No XSS vulnerability
- [ ] Description displays as plain text

**Test 12.6: Very Long Description**
- [ ] Create invoice with 1000-character description
- [ ] Invoice created successfully
- [ ] Display handles long text (wrapping/truncation)
- [ ] No UI breaks

---

## Phase 4: Security Testing

### Test 13: Privacy Verification

**Test 13.1: Storage Inspection**
```bash
# Check RocksDB stored data
hexdump -C /tmp/rocksdb_test/confidential_invoices/inv_... | head
```

**Checklist**:
- [ ] Amount not visible in raw storage
- [ ] Only encrypted bytes visible
- [ ] Cannot reverse-engineer amount from storage

**Test 13.2: Network Traffic**
```bash
# Use browser DevTools Network tab
# Or Wireshark to capture WebSocket traffic
```

**Checklist**:
- [ ] Amount sent to TEE Worker encrypted (future: TLS/WSS)
- [ ] Response shows decrypted amount (expected in current MVP)
- [ ] Future: Implement WSS (WebSocket Secure)

**Test 13.3: Commitment Verification**
```bash
# Manually verify commitment
# SHA256(invoice_id || amount_bytes)
echo -n "inv_...$(printf '%016x' 50000000000 | xxd -r -p)" | sha256sum
```

**Checklist**:
- [ ] Manual hash matches returned commitment
- [ ] Changing amount changes commitment
- [ ] Changing invoice_id changes commitment

---

### Test 14: XSS Protection

**Test 14.1: Malicious Input**
Try injecting in all fields:
```html
<script>alert('XSS')</script>
<img src=x onerror=alert('XSS')>
javascript:alert('XSS')
```

**Checklist**:
- [ ] No alerts triggered
- [ ] HTML rendered as plain text
- [ ] Proper escaping applied

---

### Test 15: CORS & Origin Checks

**Test 15.1: Cross-Origin Requests**
```bash
# Try accessing from different origin
# (if TEE Worker implements CORS)
```

**Checklist**:
- [ ] Same-origin works
- [ ] Cross-origin handled appropriately
- [ ] CORS headers correct (if applicable)

---

## Phase 5: Performance Testing

### Test 16: Response Times

**Measure with browser DevTools Performance tab**:

- [ ] Invoice creation: < 2000ms
  - [ ] Encryption time: < 100ms
  - [ ] RocksDB write: < 50ms
  - [ ] Network round-trip: < 1000ms

- [ ] Invoice retrieval: < 1000ms
  - [ ] RocksDB read: < 50ms
  - [ ] Decryption time: < 100ms
  - [ ] Network round-trip: < 500ms

- [ ] Page load: < 1000ms
  - [ ] HTML parsing: < 100ms
  - [ ] TailwindCSS load: < 200ms (CDN)
  - [ ] Ethers.js load: < 500ms (CDN)

---

### Test 17: Load Testing

**Test 17.1: Rapid Invoice Creation**
```bash
# Create 100 invoices rapidly
for i in {1..100}; do
  wscat -c ws://localhost:2004 <<< '{...}' &
done
```

**Checklist**:
- [ ] All succeed
- [ ] No duplicate IDs
- [ ] Storage handles concurrent writes
- [ ] No memory leaks in TEE Worker

---

## Test Results Summary

**Copy this template for test reports**:

```markdown
# Privacy Invoice Test Results

**Date**: YYYY-MM-DD
**Tester**: [Name]
**Environment**: [Local/Staging/Production]
**TEE Worker Version**: [Commit Hash]

## Summary
- Total Tests: XX
- Passed: XX
- Failed: XX
- Skipped: XX

## Critical Issues
- [ ] None found / [List issues]

## Non-Critical Issues
- [ ] [List issues]

## Recommendations
- [ ] [List recommendations]

## Sign-Off
- [ ] Ready for Phase 3 completion
- [ ] Ready for demo
- [ ] Ready for production (requires additional work)
```

---

## Automated Testing

### Integration Tests

```bash
# Run Rust integration tests
cd /home/kai/workspace/heima/tee-worker/omni-executor
cargo test --features=test-endpoints confidential_invoice

# Expected output:
# test confidential_invoice::tests::test_create_and_get_invoice ... ok
# test confidential_invoice::tests::test_invoice_not_found ... ok
# test confidential_invoice::tests::test_invalid_currency ... ok
```

---

## Next Steps After Testing

- [ ] All Phase 2 tests pass
- [ ] Document any issues found
- [ ] Create GitHub issues for bugs
- [ ] Update progress-5.md with test results
- [ ] Prepare demo script based on test findings
- [ ] Record demo video
- [ ] Deploy to staging environment (if available)
