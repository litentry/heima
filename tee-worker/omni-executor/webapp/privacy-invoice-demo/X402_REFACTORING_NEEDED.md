# X402 Architecture Refactoring Needed

## Current Problem

The privacy-invoice-demo **does not follow x402 architecture**. Currently:

### Current (Wrong) Flow
```
┌─────────┐                    ┌─────────────┐
│ Buyer   │ ──(1) Sign UserOp─→│ TEE Server  │
│ (Front) │ ──(2) Submit to───→│omni_settle  │
└─────────┘     EntryPoint      │  UserOp     │
                                └─────────────┘
```

**Problems**:
1. Buyer's smart wallet must have ETH for gas ❌
2. No paymaster support ❌
3. Not following x402 pattern ❌
4. Invoice payment bypasses x402 facilitator ❌

## Correct X402 Flow

### What Should Happen
```
┌─────────┐               ┌──────────────┐              ┌───────────┐
│ Buyer   │──(1) Sign────→│ TEE Server   │──(3) Submit─→│ EntryPoint│
│ (Front) │   UserOp      │ (Facilitator)│   with       │  on-chain │
└─────────┘               └──────────────┘   Paymaster  └───────────┘
                                 │
                            (2) Add paymaster
                                 ↓
                         ┌──────────────┐
                         │  Paymaster   │
                         │ (pays gas)   │
                         └──────────────┘
```

**Benefits**:
1. Buyer's smart wallet only needs tokens (no ETH) ✅
2. Paymaster pays gas fees ✅
3. Follows x402 architecture ✅
4. Server acts as facilitator ✅

## Required Changes

### 1. Backend: New RPC Method `omni_payInvoiceWithUserOp`

Replace direct calls to `omni_settleUserOp` with invoice-specific payment handler:

```rust
#[derive(Debug, Deserialize)]
pub struct PayInvoiceWithUserOpParams {
    pub invoice_id: String,
    pub user_operation: SerializablePackedUserOperation, // Signed by buyer
    pub chain_id: ChainId,
}

#[derive(Serialize)]
pub struct PayInvoiceWithUserOpResponse {
    pub transaction_hash: String,
    pub invoice_status: InvoiceStatus,
    pub message: String,
}
```

**Implementation steps**:
1. Retrieve invoice from storage
2. Verify invoice is in `Pending` status
3. Decrypt amount and verify UserOp amount matches
4. **Add paymaster data** to UserOp:
   ```rust
   let paymaster_data = get_paymaster_signature(
       &user_op,
       paymaster_address,
       chain_id
   ).await?;
   user_op.paymasterAndData = paymaster_data;
   ```
5. Submit to EntryPoint via `handle_ops_with_retry()`
6. Update invoice status to `Paid` with tx_hash
7. Return success response

### 2. Frontend: Update Payment Flow

**Current (Wrong)**:
```javascript
// pay-invoice.html lines 356-358
const txHash = await settleUserOp(userOp);  // ❌ Direct submission
```

**Should be**:
```javascript
// New flow
async function payInvoice() {
    // Step 1: Build UserOp (no paymaster yet)
    const userOp = await buildPaymentUserOp(...);

    // Step 2: Sign with MetaMask
    const signature = await signUserOperation(userOp, userAddress);
    userOp.signature = signature;

    // Step 3: Send to facilitator (TEE server)
    const result = await payInvoiceWithUserOp(invoiceId, userOp);

    // Server adds paymaster and submits
    return result.transaction_hash;
}

async function payInvoiceWithUserOp(invoiceId, userOp) {
    const ws = new WebSocket(RPC_URL);
    await new Promise((resolve, reject) => {
        ws.onopen = resolve;
        ws.onerror = reject;
    });

    const request = {
        jsonrpc: '2.0',
        id: Math.floor(Math.random() * 1000000),
        method: 'omni_payInvoiceWithUserOp',
        params: {
            invoice_id: invoiceId,
            user_operation: userOp,
            chain_id: CHAIN_ID
        }
    };

    ws.send(JSON.stringify(request));

    const result = await new Promise((resolve, reject) => {
        ws.onmessage = (event) => {
            const response = JSON.parse(event.data);
            if (response.id === request.id) {
                if (response.error) {
                    reject(new Error(response.error.message));
                } else {
                    resolve(response.result);
                }
            }
        };
        setTimeout(() => reject(new Error('Timeout')), 30000);
    });

    ws.close();
    return result;
}
```

### 3. Paymaster Integration

Add paymaster support in the backend:

```rust
// In settle_user_op.rs or new paymaster module
async fn add_paymaster_data(
    user_op: &mut PackedUserOperation,
    paymaster_address: Address,
    chain_id: ChainId,
) -> Result<(), Error> {
    // Get paymaster to sign the UserOp
    let paymaster_and_data = paymaster_client
        .get_paymaster_and_data(user_op, chain_id)
        .await?;

    user_op.paymasterAndData = paymaster_and_data;
    Ok(())
}
```

### 4. Configuration

Add paymaster configuration to RPC server context:

```rust
pub struct RpcContext {
    // ... existing fields
    pub paymaster_address: Address,
    pub paymaster_enabled: bool,
}
```

## File Changes Summary

### Backend Files to Modify

1. **`rpc-server/src/methods/omni/confidential_invoice.rs`**
   - Add `PayInvoiceWithUserOpParams` struct
   - Add `PayInvoiceWithUserOpResponse` struct
   - Add `omni_payInvoiceWithUserOp` RPC method
   - Integrate paymaster logic
   - Update invoice status after payment

2. **`rpc-server/src/methods/omni/settle_user_op.rs`** (or create new `paymaster.rs`)
   - Add paymaster integration helper
   - Add function to get paymaster signature

3. **`rpc-server/src/server.rs`**
   - Add paymaster configuration to RpcContext

### Frontend Files to Modify

1. **`webapp/privacy-invoice-demo/pay-invoice.html`**
   - Replace `settleUserOp()` with `payInvoiceWithUserOp()`
   - Update flow to send UserOp to server instead of submitting directly
   - Remove direct EntryPoint submission logic

## Testing Checklist

After refactoring:

- [ ] Buyer can pay invoice without ETH in smart wallet
- [ ] Paymaster pays gas fees
- [ ] Invoice status updates to `Paid` after successful payment
- [ ] Transaction hash is recorded in invoice
- [ ] Frontend shows correct payment flow
- [ ] Error handling for failed payments
- [ ] Invoice cannot be paid twice

## Benefits After Refactoring

1. **True x402 Architecture**: Server acts as facilitator
2. **Gasless Payments**: Buyers don't need ETH
3. **Better UX**: Simpler for users
4. **Centralized Logic**: Payment logic in one place
5. **Invoice Tracking**: Automatic status updates

## Current Status

❌ **Not x402 compliant** - needs refactoring
❌ **Requires gas in smart wallet** - not gasless
❌ **Direct EntryPoint submission** - bypasses facilitator

## Next Steps

1. Implement `omni_payInvoiceWithUserOp` RPC method
2. Add paymaster integration
3. Update frontend to use new flow
4. Test end-to-end with paymaster
5. Update documentation
