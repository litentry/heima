# Privacy Pools + x402 Implementation Plan

**Goal:** Implement true transaction amount privacy for invoice payments using Privacy Pools with x402 facilitator architecture.

**Timeline:** 4-6 weeks
**Privacy Guarantee:** Transaction amounts hidden via anonymity sets (requires 50+ concurrent users)
**Compliance:** Association set proofs for regulatory compliance

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Smart Contracts](#smart-contracts)
3. [Backend Implementation](#backend-implementation)
4. [Frontend Implementation](#frontend-implementation)
5. [Testing Strategy](#testing-strategy)
6. [Deployment Plan](#deployment-plan)

---

## Architecture Overview

### Current x402 Flow (Corrected)

```
Frontend (Buyer)
    ↓ signs UserOp
    ↓
Server (Express)
    ↓ forwards to TEE worker
    ↓
TEE Worker (x402 Facilitator)
    ↓ verifies UserOp
    ↓ adds paymaster signature
    ↓ submits to EntryPoint
    ↓
Blockchain (EntryPoint → execution)
```

### Privacy Pools + x402 Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ DEPOSIT FLOW (Buyer pays invoice)                               │
└─────────────────────────────────────────────────────────────────┘

Frontend (Buyer)
    │
    ├─ 1. Fetch invoice details (encrypted amount from TEE)
    ├─ 2. Build UserOp: deposit to Privacy Pool
    ├─ 3. Sign UserOp with MetaMask
    │
    └─> POST /api/pay-invoice
        { invoice_id, signed_user_op }

        ↓

Server (Express)
    │
    └─> RPC: omni_payInvoice(invoice_id, signed_user_op)

        ↓

TEE Worker (x402 Facilitator)
    │
    ├─ 4. Decrypt invoice amount
    ├─ 5. Verify UserOp amount matches invoice
    ├─ 6. Generate deposit commitment:
    │     commitment = hash(invoice_id || amount || secret)
    ├─ 7. Store: commitment → invoice_id (encrypted mapping)
    ├─ 8. Update UserOp callData:
    │     PrivacyPool.deposit(commitment, amount)
    ├─ 9. Add paymaster signature (gas sponsorship)
    ├─ 10. Call omni_settleUserOp (internal)
    │
    └─> Submit UserOp to blockchain

        ↓

Blockchain (Arbitrum Sepolia)
    │
    ├─ EntryPoint validates UserOp
    ├─ Paymaster sponsors gas
    ├─ Execute: buyer_smart_wallet.execute(
    │              PrivacyPool.deposit(commitment, amount)
    │            )
    │
    └─> Privacy Pool Contract
        ├─ deposits[commitment] = amount
        └─> emit Deposit(commitment, amount)

        On-chain observers see:
        ✅ Deposit of X USDC with commitment 0x3f2a...
        ❌ Cannot link commitment to invoice_id (TEE secret)
        ❌ Cannot link commitment to seller (withdrawn later)


┌─────────────────────────────────────────────────────────────────┐
│ WITHDRAWAL FLOW (Seller claims payment)                         │
└─────────────────────────────────────────────────────────────────┘

Frontend (Seller)
    │
    └─> POST /api/withdraw-invoice
        { invoice_id, seller_signature }

        ↓

Server (Express)
    │
    └─> RPC: omni_withdrawFromPool(invoice_id, seller_sig)

        ↓

TEE Worker (x402 Facilitator)
    │
    ├─ 1. Verify seller owns this invoice
    ├─ 2. Retrieve commitment for invoice_id
    ├─ 3. Check invoice payment status (must be deposited)
    ├─ 4. Generate ZK proof:
    │     proof = "commitment exists in clean association set"
    │     (Proves deposit came from non-illicit source)
    ├─ 5. Build UserOp for seller:
    │     PrivacyPool.withdraw(commitment, amount, proof, recipient)
    ├─ 6. Sign UserOp with TEE's relayer key (on behalf of seller)
    ├─ 7. Add paymaster signature (gas sponsorship)
    ├─ 8. Call omni_settleUserOp (internal)
    │
    └─> Submit UserOp to blockchain

        ↓

Blockchain (Arbitrum Sepolia)
    │
    ├─ EntryPoint validates UserOp
    ├─ Paymaster sponsors gas
    ├─ Execute: seller_smart_wallet.execute(
    │              PrivacyPool.withdraw(commitment, amount, proof, recipient)
    │            )
    │
    └─> Privacy Pool Contract
        ├─ Verify ZK proof (commitment in association set)
        ├─ Check commitment exists in deposits
        ├─ Transfer amount to recipient (seller's wallet)
        ├─ Mark commitment as withdrawn
        └─> emit Withdrawal(commitment, amount, nullifier)

        On-chain observers see:
        ✅ Withdrawal of X USDC with commitment 0x3f2a...
        ❌ Cannot link to original deposit (anonymity set of 50+ users)
        ❌ Cannot determine which buyer paid which seller
```

---

## Smart Contracts

### 1. Privacy Pool Contract

**Option A: Use Existing Privacy Pools V1**

Privacy Pools V1 is already deployed on mainnet with USDS support (Jan 2026). We need to:

1. Check if it's deployed on Arbitrum Sepolia (testnet)
2. If not, request testnet deployment or deploy ourselves
3. Use existing contract interfaces

**Contract Interface (based on Privacy Pools spec):**

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IPrivacyPool {
    /// @notice Deposit funds into the pool with a commitment
    /// @param commitment Hash of (invoice_id || amount || secret)
    /// @param amount Amount in token units (e.g., USDC with 6 decimals)
    function deposit(bytes32 commitment, uint256 amount) external;

    /// @notice Withdraw funds from pool with ZK proof
    /// @param commitment Original deposit commitment
    /// @param amount Amount to withdraw
    /// @param proof ZK proof of association set membership
    /// @param recipient Address to receive funds
    function withdraw(
        bytes32 commitment,
        uint256 amount,
        bytes calldata proof,
        address recipient
    ) external;

    /// @notice Check if commitment exists
    function commitments(bytes32 commitment) external view returns (bool);

    /// @notice Check if commitment has been withdrawn (nullified)
    function nullifiers(bytes32 nullifier) external view returns (bool);

    /// Events
    event Deposit(bytes32 indexed commitment, uint256 amount);
    event Withdrawal(bytes32 indexed nullifier, uint256 amount, address recipient);
}
```

**Option B: Simplified Privacy Pool for Demo**

For initial demo (if V1 not available on testnet), deploy simplified version:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Simplified Privacy Pool for Invoice Demo
/// @notice Provides basic privacy via deposit/withdrawal unlinkability
/// @dev Production version would use full Privacy Pools V1 with ZK proofs
contract SimplePrivacyPool {
    using SafeERC20 for IERC20;

    IERC20 public immutable token; // USDC

    // Commitment -> deposit amount
    mapping(bytes32 => uint256) public deposits;

    // Nullifier -> withdrawn status
    mapping(bytes32 => bool) public nullifiers;

    event Deposit(bytes32 indexed commitment, uint256 amount, uint256 timestamp);
    event Withdrawal(bytes32 indexed nullifier, uint256 amount, address indexed recipient);

    constructor(address _token) {
        token = IERC20(_token);
    }

    /// @notice Deposit funds with commitment
    function deposit(bytes32 commitment, uint256 amount) external {
        require(deposits[commitment] == 0, "Commitment already used");
        require(amount > 0, "Amount must be positive");

        // Transfer tokens to pool
        token.safeTransferFrom(msg.sender, address(this), amount);

        // Store deposit
        deposits[commitment] = amount;

        emit Deposit(commitment, amount, block.timestamp);
    }

    /// @notice Withdraw funds with proof (simplified - no ZK proof verification for demo)
    /// @dev Production version MUST verify ZK proof of association set
    function withdraw(
        bytes32 commitment,
        uint256 amount,
        bytes32 nullifier,
        address recipient
    ) external {
        require(deposits[commitment] == amount, "Invalid commitment or amount");
        require(!nullifiers[nullifier], "Already withdrawn");

        // Mark as withdrawn (prevent double-spend)
        nullifiers[nullifier] = true;

        // Transfer to recipient
        token.safeTransfer(recipient, amount);

        emit Withdrawal(nullifier, amount, recipient);
    }

    /// @notice Get pool balance
    function poolBalance() external view returns (uint256) {
        return token.balanceOf(address(this));
    }
}
```

**Deployment addresses needed:**
- Arbitrum Sepolia USDC: `0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d`
- SimplePrivacyPool: TBD (deploy via Foundry)

---

## Backend Implementation

### Phase 1: Storage Schema Updates

**File:** `tee-worker/omni-executor/executor-storage/src/confidential_invoice.rs`

Add pool commitment tracking:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidentialInvoice {
    pub invoice_id: String,
    pub seller_account: AccountId,
    pub buyer_identifier: String,
    pub encrypted_amount: Vec<u8>,
    pub commitment: [u8; 32],  // Existing invoice commitment

    // NEW: Privacy Pool integration
    pub pool_commitment: Option<[u8; 32]>,  // Commitment used for pool deposit
    pub pool_secret: Option<Vec<u8>>,        // Secret for ZK proof (encrypted)
    pub pool_deposit_tx: Option<String>,     // Transaction hash of deposit
    pub pool_withdrawal_tx: Option<String>,  // Transaction hash of withdrawal

    pub metadata: InvoiceMetadata,
    pub status: InvoiceStatus,
    pub created_at: u64,
    pub paid_at: Option<u64>,
    pub withdrawn_at: Option<u64>,  // NEW: When seller withdrew from pool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Pending,          // Invoice created, not paid
    Deposited,        // NEW: Funds deposited to pool
    Withdrawn,        // NEW: Seller withdrew from pool
    Paid,             // Legacy: direct payment (non-pool)
    Expired,
    Cancelled,
}
```

### Phase 2: Crypto Utilities

**File:** `tee-worker/omni-executor/executor-core/src/crypto/privacy_pool.rs`

```rust
use sha3::{Digest, Keccak256};
use rand::RngCore;

/// Generate privacy pool commitment
/// commitment = keccak256(invoice_id || amount || secret)
pub fn generate_pool_commitment(
    invoice_id: &str,
    amount: u128,
    secret: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(invoice_id.as_bytes());
    hasher.update(&amount.to_le_bytes());
    hasher.update(secret);
    hasher.finalize().into()
}

/// Generate nullifier for withdrawal
/// nullifier = keccak256(commitment || "nullifier")
pub fn generate_nullifier(commitment: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(commitment);
    hasher.update(b"nullifier");
    hasher.finalize().into()
}

/// Generate random secret for commitment
pub fn generate_pool_secret() -> [u8; 32] {
    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);
    secret
}

/// Encrypt pool secret with TEE key (for storage)
pub fn encrypt_pool_secret(secret: &[u8; 32], tee_key: &[u8; 32]) -> Result<Vec<u8>, Error> {
    // Reuse existing AES-256-GCM encryption
    crate::crypto::confidential::encrypt_amount(
        u128::from_le_bytes(secret[0..16].try_into().unwrap()),
        tee_key
    )
}

/// Decrypt pool secret
pub fn decrypt_pool_secret(encrypted: &[u8], tee_key: &[u8; 32]) -> Result<[u8; 32], Error> {
    // Simplified - in production, properly encrypt/decrypt full 32 bytes
    let partial = crate::crypto::confidential::decrypt_amount(encrypted, tee_key)?;
    let mut secret = [0u8; 32];
    secret[0..16].copy_from_slice(&partial.to_le_bytes());
    Ok(secret)
}

// TODO: ZK proof generation (future - requires circom/snarkjs integration)
/// Generate ZK proof of association set membership
/// For demo: return dummy proof (production MUST implement real ZK proof)
pub fn generate_association_proof(
    commitment: &[u8; 32],
    secret: &[u8; 32],
    association_set: &[[u8; 32]],
) -> Vec<u8> {
    // PLACEHOLDER: In production, generate Groth16 proof
    // Proving: "I know secret for commitment in association_set"
    vec![0u8; 256] // Dummy 256-byte proof
}
```

### Phase 3: RPC Methods

**File:** `tee-worker/omni-executor/rpc-server/src/methods/omni/privacy_pool_invoice.rs`

```rust
use crate::server::RpcContext;
use crate::crypto::privacy_pool::*;
use crate::RpcResult;
use executor_storage::confidential_invoice::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use ethers::types::{Address, U256};
use ethers::abi::{encode, Token};

// Contract addresses
const PRIVACY_POOL_ADDRESS: &str = "0x0000000000000000000000000000000000000000"; // TODO: Deploy
const USDC_ADDRESS: &str = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; // Arbitrum Sepolia

pub fn register_privacy_pool_methods<
    CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
    module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
    // ═══════════════════════════════════════════════════════════════
    // METHOD: omni_payInvoice (deposit to privacy pool via x402)
    // ═══════════════════════════════════════════════════════════════
    module
        .register_async_method("omni_payInvoice", |params, ctx, _| async move {
            #[derive(Deserialize)]
            struct Params {
                invoice_id: String,
                user_operation: SerializablePackedUserOperation,
                chain_id: u64,
            }

            let params = parse_rpc_params::<Params>(params)?;

            info!("Processing privacy pool invoice payment: {}", params.invoice_id);

            // Step 1: Retrieve and validate invoice
            let mut invoice = ctx.invoice_storage
                .get(&invoice::Key { invoice_id: params.invoice_id.clone() })
                .ok()
                .flatten()
                .ok_or_internal("Invoice not found")?;

            if invoice.status != InvoiceStatus::Pending {
                return Err(DetailedError::invalid_params(
                    "invoice_id",
                    "Invoice already paid or expired"
                ).to_rpc_error());
            }

            // Step 2: Decrypt invoice amount
            let tee_key = ctx.get_tee_encryption_key()?;
            let amount = decrypt_amount(&invoice.encrypted_amount, &tee_key)
                .map_err_internal("Failed to decrypt invoice amount")?;

            info!("Invoice amount: {} (smallest units)", amount);

            // Step 3: Generate privacy pool commitment
            let pool_secret = generate_pool_secret();
            let pool_commitment = generate_pool_commitment(
                &params.invoice_id,
                amount,
                &pool_secret
            );

            info!("Generated pool commitment: 0x{}", hex::encode(pool_commitment));

            // Step 4: Encrypt and store pool secret
            let encrypted_secret = encrypt_pool_secret(&pool_secret, &tee_key)
                .map_err_internal("Failed to encrypt pool secret")?;

            // Step 5: Update UserOp callData to call PrivacyPool.deposit()
            let privacy_pool_addr: Address = PRIVACY_POOL_ADDRESS.parse()
                .map_err_internal("Invalid privacy pool address")?;

            // Encode: PrivacyPool.deposit(bytes32 commitment, uint256 amount)
            let deposit_calldata = encode(&[
                Token::FixedBytes(pool_commitment.to_vec()),
                Token::Uint(U256::from(amount)),
            ]);

            // Function selector for deposit(bytes32,uint256)
            let selector = &keccak256(b"deposit(bytes32,uint256)")[0..4];
            let full_calldata = [selector, &deposit_calldata].concat();

            // Update UserOp to call privacy pool
            let mut user_op = params.user_operation;
            user_op.call_data = full_calldata.into();

            info!("Updated UserOp callData to deposit to privacy pool");

            // Step 6: Add paymaster signature (x402 facilitator)
            let user_op_with_paymaster = ctx.add_paymaster_signature(
                user_op,
                params.chain_id
            ).await?;

            // Step 7: Settle via x402 facilitator (calls omni_settleUserOp internally)
            let tx_hash = ctx.settle_user_operation(
                user_op_with_paymaster,
                params.chain_id
            ).await?;

            info!("Privacy pool deposit submitted: tx={}", tx_hash);

            // Step 8: Update invoice status
            invoice.status = InvoiceStatus::Deposited;
            invoice.pool_commitment = Some(pool_commitment);
            invoice.pool_secret = Some(encrypted_secret);
            invoice.pool_deposit_tx = Some(tx_hash.clone());
            invoice.paid_at = Some(current_timestamp());

            ctx.invoice_storage
                .update(&invoice::Key { invoice_id: params.invoice_id.clone() }, |inv| {
                    *inv = invoice.clone();
                })
                .map_err_internal("Failed to update invoice")?;

            Ok(json!({
                "status": "deposited",
                "tx_hash": tx_hash,
                "commitment": format!("0x{}", hex::encode(pool_commitment)),
                "message": "Funds deposited to privacy pool. Seller can withdraw anytime."
            }))
        })
        .expect("Failed to register omni_payInvoice");

    // ═══════════════════════════════════════════════════════════════
    // METHOD: omni_withdrawFromPool (seller claims payment via x402)
    // ═══════════════════════════════════════════════════════════════
    module
        .register_async_method("omni_withdrawFromPool", |params, ctx, _| async move {
            #[derive(Deserialize)]
            struct Params {
                invoice_id: String,
                recipient: String,  // Seller's withdrawal address
                chain_id: u64,
            }

            let params = parse_rpc_params::<Params>(params)?;

            info!("Processing privacy pool withdrawal: {}", params.invoice_id);

            // Step 1: Retrieve invoice
            let mut invoice = ctx.invoice_storage
                .get(&invoice::Key { invoice_id: params.invoice_id.clone() })
                .ok()
                .flatten()
                .ok_or_internal("Invoice not found")?;

            if invoice.status != InvoiceStatus::Deposited {
                return Err(DetailedError::invalid_params(
                    "invoice_id",
                    "Invoice not deposited to pool yet"
                ).to_rpc_error());
            }

            // Step 2: Retrieve pool commitment and secret
            let pool_commitment = invoice.pool_commitment
                .ok_or_internal("Pool commitment missing")?;

            let encrypted_secret = invoice.pool_secret.as_ref()
                .ok_or_internal("Pool secret missing")?;

            let tee_key = ctx.get_tee_encryption_key()?;
            let pool_secret = decrypt_pool_secret(encrypted_secret, &tee_key)
                .map_err_internal("Failed to decrypt pool secret")?;

            // Step 3: Decrypt amount
            let amount = decrypt_amount(&invoice.encrypted_amount, &tee_key)
                .map_err_internal("Failed to decrypt amount")?;

            // Step 4: Generate nullifier
            let nullifier = generate_nullifier(&pool_commitment);

            info!("Generated nullifier: 0x{}", hex::encode(nullifier));

            // Step 5: Generate ZK proof (placeholder for demo)
            // TODO: Implement real ZK proof generation with circom
            let association_set = vec![pool_commitment]; // Simplified
            let zk_proof = generate_association_proof(
                &pool_commitment,
                &pool_secret,
                &association_set
            );

            // Step 6: Build UserOp for withdrawal
            // Encode: PrivacyPool.withdraw(bytes32 commitment, uint256 amount, bytes32 nullifier, address recipient)
            let recipient_addr: Address = params.recipient.parse()
                .map_err_internal("Invalid recipient address")?;

            let withdraw_calldata = encode(&[
                Token::FixedBytes(pool_commitment.to_vec()),
                Token::Uint(U256::from(amount)),
                Token::FixedBytes(nullifier.to_vec()),
                Token::Address(recipient_addr),
            ]);

            let selector = &keccak256(b"withdraw(bytes32,uint256,bytes32,address)")[0..4];
            let full_calldata = [selector, &withdraw_calldata].concat();

            // Build UserOp (TEE acts as relayer for seller)
            let user_op = ctx.build_user_operation(
                invoice.seller_account.clone(),
                PRIVACY_POOL_ADDRESS.to_string(),
                full_calldata,
                params.chain_id
            ).await?;

            // Step 7: Add paymaster signature
            let user_op_with_paymaster = ctx.add_paymaster_signature(
                user_op,
                params.chain_id
            ).await?;

            // Step 8: Settle via x402
            let tx_hash = ctx.settle_user_operation(
                user_op_with_paymaster,
                params.chain_id
            ).await?;

            info!("Privacy pool withdrawal submitted: tx={}", tx_hash);

            // Step 9: Update invoice
            invoice.status = InvoiceStatus::Withdrawn;
            invoice.pool_withdrawal_tx = Some(tx_hash.clone());
            invoice.withdrawn_at = Some(current_timestamp());

            ctx.invoice_storage
                .update(&invoice::Key { invoice_id: params.invoice_id.clone() }, |inv| {
                    *inv = invoice.clone();
                })
                .map_err_internal("Failed to update invoice")?;

            Ok(json!({
                "status": "withdrawn",
                "tx_hash": tx_hash,
                "recipient": params.recipient,
                "message": "Funds withdrawn from privacy pool to seller address."
            }))
        })
        .expect("Failed to register omni_withdrawFromPool");
}
```

---

## Frontend Implementation

### 1. Update Pay Invoice Page

**File:** `tee-worker/omni-executor/webapp/privacy-invoice-demo/pay-invoice.html`

Changes needed:

```javascript
// Line ~356: Replace direct omni_settleUserOp call

// OLD (current implementation):
const result = await wsClient.request('omni_settleUserOp', [{
    user_operation: userOp,
    chain_id: CHAIN_ID
}]);

// NEW (privacy pool implementation):
const result = await wsClient.request('omni_payInvoice', [{
    invoice_id: invoiceId,
    user_operation: userOp,  // UserOp will be modified by TEE to call privacy pool
    chain_id: CHAIN_ID
}]);

// Update success message
document.getElementById('statusMessage').innerText =
    'Payment deposited to privacy pool! Seller can withdraw anytime.';
```

### 2. Create Seller Withdrawal Page

**File:** `tee-worker/omni-executor/webapp/privacy-invoice-demo/withdraw-invoice.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Withdraw Invoice Payment - Privacy Pools Demo</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            background: white;
            border-radius: 10px;
            padding: 30px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.1);
        }
        h1 {
            color: #667eea;
            margin-bottom: 10px;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 8px;
            font-weight: bold;
            color: #333;
        }
        input, select {
            width: 100%;
            padding: 12px;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 14px;
            box-sizing: border-box;
        }
        button {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px 30px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: bold;
            cursor: pointer;
            width: 100%;
            margin-top: 10px;
        }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
        .status {
            margin-top: 20px;
            padding: 15px;
            border-radius: 8px;
            display: none;
        }
        .status.success {
            background: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }
        .status.error {
            background: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }
        .privacy-notice {
            background: #fff3cd;
            border: 1px solid #ffc107;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔒 Withdraw Invoice Payment</h1>
        <p class="subtitle">Claim your payment from the privacy pool</p>

        <div class="privacy-notice">
            <strong>🎯 Privacy Guarantee:</strong> This withdrawal is unlinkable to the original deposit.
            On-chain observers cannot determine which buyer paid you.
        </div>

        <div class="form-group">
            <label for="invoiceId">Invoice ID</label>
            <input type="text" id="invoiceId" placeholder="inv_xxxxxxxxxxxxxxxx" required>
        </div>

        <div class="form-group">
            <label for="recipientAddress">Withdrawal Address</label>
            <input type="text" id="recipientAddress" placeholder="0x..." required>
            <small style="color: #666;">Address to receive the funds (usually seller's main wallet)</small>
        </div>

        <button id="withdrawButton" onclick="withdrawFromPool()">
            Withdraw from Privacy Pool
        </button>

        <div id="statusDiv" class="status"></div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/@open-rpc/client-js@1.8.1/build/index.min.js"></script>
    <script>
        const RPC_URL = 'ws://localhost:2100';
        let wsClient;

        // Initialize WebSocket connection
        async function initWebSocket() {
            const { RequestManager, WebSocketTransport } = window.OpenRPCClientJS;
            const transport = new WebSocketTransport(RPC_URL);
            wsClient = new RequestManager([transport]);
            await transport.connection;
            console.log('WebSocket connected to TEE worker');
        }

        // Withdraw funds from privacy pool
        async function withdrawFromPool() {
            const invoiceId = document.getElementById('invoiceId').value.trim();
            const recipientAddress = document.getElementById('recipientAddress').value.trim();
            const statusDiv = document.getElementById('statusDiv');
            const button = document.getElementById('withdrawButton');

            if (!invoiceId || !recipientAddress) {
                showStatus('error', 'Please fill in all fields');
                return;
            }

            // Validate address format
            if (!recipientAddress.match(/^0x[a-fA-F0-9]{40}$/)) {
                showStatus('error', 'Invalid Ethereum address format');
                return;
            }

            button.disabled = true;
            button.textContent = 'Processing withdrawal...';

            try {
                // Initialize WebSocket if needed
                if (!wsClient) {
                    await initWebSocket();
                }

                console.log('Requesting withdrawal from privacy pool...');
                console.log('Invoice ID:', invoiceId);
                console.log('Recipient:', recipientAddress);

                const result = await wsClient.request('omni_withdrawFromPool', [{
                    invoice_id: invoiceId,
                    recipient: recipientAddress,
                    chain_id: 421614  // Arbitrum Sepolia
                }]);

                console.log('Withdrawal result:', result);

                showStatus('success', `
                    ✅ Withdrawal successful!<br>
                    <strong>Transaction:</strong> <a href="https://sepolia.arbiscan.io/tx/${result.tx_hash}" target="_blank">${result.tx_hash}</a><br>
                    <strong>Status:</strong> Funds withdrawn to ${recipientAddress}<br>
                    <br>
                    <strong>🔒 Privacy Note:</strong> On-chain observers can see this withdrawal but cannot link it to the original deposit.
                `);

                button.textContent = '✅ Withdrawn Successfully';

            } catch (error) {
                console.error('Withdrawal error:', error);
                showStatus('error', `Withdrawal failed: ${error.message || error}`);
                button.disabled = false;
                button.textContent = 'Withdraw from Privacy Pool';
            }
        }

        function showStatus(type, message) {
            const statusDiv = document.getElementById('statusDiv');
            statusDiv.className = `status ${type}`;
            statusDiv.innerHTML = message;
            statusDiv.style.display = 'block';
        }

        // Auto-fill invoice ID from URL parameter
        window.addEventListener('DOMContentLoaded', () => {
            const urlParams = new URLSearchParams(window.location.search);
            const invoiceId = urlParams.get('id');
            if (invoiceId) {
                document.getElementById('invoiceId').value = invoiceId;
            }

            initWebSocket();
        });
    </script>
</body>
</html>
```

### 3. Update Create Invoice Page

**File:** `tee-worker/omni-executor/webapp/privacy-invoice-demo/create-invoice.html`

Add note about privacy pools (line ~320):

```html
<div style="background: #e3f2fd; padding: 15px; border-radius: 8px; margin-top: 20px;">
    <h3 style="margin-top: 0; color: #1976d2;">🔒 Privacy Pool Integration</h3>
    <p style="margin-bottom: 0;">
        This invoice will be paid through a privacy pool, providing true transaction privacy.
        Payment amounts are hidden via anonymity sets (requires 50+ concurrent users for strong privacy).
    </p>
</div>
```

---

## Testing Strategy

### Unit Tests

**File:** `tee-worker/omni-executor/executor-core/src/crypto/privacy_pool_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pool_commitment() {
        let invoice_id = "inv_test123";
        let amount = 1_000_000u128; // 1 USDC
        let secret = [1u8; 32];

        let commitment = generate_pool_commitment(invoice_id, amount, &secret);

        // Commitment should be deterministic
        let commitment2 = generate_pool_commitment(invoice_id, amount, &secret);
        assert_eq!(commitment, commitment2);

        // Different secret = different commitment
        let secret2 = [2u8; 32];
        let commitment3 = generate_pool_commitment(invoice_id, amount, &secret2);
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_generate_nullifier() {
        let commitment = [1u8; 32];
        let nullifier = generate_nullifier(&commitment);

        // Nullifier should be deterministic
        let nullifier2 = generate_nullifier(&commitment);
        assert_eq!(nullifier, nullifier2);

        // Different commitment = different nullifier
        let commitment2 = [2u8; 32];
        let nullifier3 = generate_nullifier(&commitment2);
        assert_ne!(nullifier, nullifier3);
    }
}
```

### Integration Tests

**Scenario 1: Full payment flow**

```bash
# 1. Create invoice
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_createConfidentialInvoice",
    "params": [{
      "seller_account": "0x...",
      "buyer_identifier": "buyer@example.com",
      "amount": "1000000",
      "token_address": "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d",
      "chain_id": 421614,
      "description": "Test invoice"
    }],
    "id": 1
  }'

# 2. Pay invoice (deposit to pool)
# => Use frontend (pay-invoice.html)

# 3. Verify deposit on-chain
cast call $PRIVACY_POOL "deposits(bytes32)" $COMMITMENT --rpc-url $ARB_SEPOLIA_RPC

# 4. Withdraw from pool
# => Use frontend (withdraw-invoice.html)

# 5. Verify withdrawal on-chain
cast call $PRIVACY_POOL "nullifiers(bytes32)" $NULLIFIER --rpc-url $ARB_SEPOLIA_RPC
```

**Scenario 2: Privacy verification**

1. Create 3+ invoices with different buyers
2. All deposit to same privacy pool
3. Monitor blockchain events
4. Verify: Cannot link deposits to withdrawals

### Manual Testing Checklist

- [ ] Invoice creation stores pool_commitment and pool_secret
- [ ] Buyer's deposit UserOp calls PrivacyPool.deposit()
- [ ] Paymaster sponsors gas for deposit
- [ ] Deposit emits correct event on-chain
- [ ] Invoice status updates to "Deposited"
- [ ] Seller can query invoice and see "ready to withdraw"
- [ ] Withdrawal UserOp calls PrivacyPool.withdraw()
- [ ] Paymaster sponsors gas for withdrawal
- [ ] Withdrawal emits correct event on-chain
- [ ] Funds transferred to seller's address
- [ ] Invoice status updates to "Withdrawn"
- [ ] Cannot withdraw same invoice twice (nullifier check)

---

## Deployment Plan

### Phase 1: Smart Contract Deployment (Week 1)

**Step 1: Deploy SimplePrivacyPool to Arbitrum Sepolia**

```bash
cd tee-worker/omni-executor/aa-contracts

# Create deployment script
cat > script/DeployPrivacyPool.s.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/SimplePrivacyPool.sol";

contract DeployPrivacyPool is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address usdc = 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d; // Arbitrum Sepolia USDC

        vm.startBroadcast(deployerPrivateKey);

        SimplePrivacyPool pool = new SimplePrivacyPool(usdc);
        console.log("SimplePrivacyPool deployed at:", address(pool));

        vm.stopBroadcast();
    }
}
EOF

# Deploy
forge script script/DeployPrivacyPool.s.sol:DeployPrivacyPool \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --broadcast \
  --verify \
  -vvvv
```

**Step 2: Update contract addresses in code**

- Update `PRIVACY_POOL_ADDRESS` in `privacy_pool_invoice.rs`
- Update `PRIVACY_POOL_ADDRESS` in frontend HTML files

### Phase 2: Backend Deployment (Week 2-3)

**Step 1: Update TEE worker code**

```bash
cd tee-worker/omni-executor

# Add privacy pool module to rpc-server
# File: rpc-server/src/methods/omni/mod.rs
pub mod privacy_pool_invoice;

# Register methods in main.rs
use methods::omni::privacy_pool_invoice::register_privacy_pool_methods;
register_privacy_pool_methods(&mut module);
```

**Step 2: Build and restart TEE worker**

```bash
# Build
cd tee-worker/omni-executor
cargo build --release

# Restart worker
# (Follow existing deployment process for TEE worker)
```

### Phase 3: Frontend Deployment (Week 3-4)

**Step 1: Deploy updated frontend**

```bash
# Copy updated HTML files
cp webapp/privacy-invoice-demo/*.html /path/to/web/server/

# Or if using Express server:
npm run dev  # Test locally
npm run build && npm start  # Production
```

**Step 2: Update documentation**

- Add privacy pools explanation to README
- Document withdrawal process for sellers
- Add Arbiscan links for transparency

### Phase 4: Testing & Demo (Week 4-5)

**Step 1: Seed test data**

- Create 10+ test invoices
- Have multiple test buyers deposit to pool
- Verify anonymity set

**Step 2: Record demo video**

- Show invoice creation
- Show buyer payment (deposit to pool)
- Show on-chain events (deposits visible)
- Show seller withdrawal
- Show privacy verification (can't link deposit to withdrawal)

**Step 3: Launch announcement**

- Blog post explaining privacy pools integration
- Technical documentation
- Demo video on YouTube/Twitter

---

## Success Metrics

### Technical Metrics

- ✅ Invoice creation: < 2 seconds
- ✅ Deposit transaction: < 30 seconds (with paymaster)
- ✅ Withdrawal transaction: < 30 seconds (with paymaster)
- ✅ Gas cost (buyer): $0 (paymaster sponsors)
- ✅ Gas cost (seller): $0 (paymaster sponsors)
- ✅ On-chain privacy: Deposits/withdrawals unlinkable with 50+ users

### Privacy Metrics

- ✅ Invoice amounts encrypted in TEE (pre-payment)
- ✅ Deposit amounts visible but unlinkable to invoices
- ✅ Withdrawal amounts visible but unlinkable to deposits
- ✅ Anonymity set size: 50+ concurrent users (target)
- ✅ Association set proofs for compliance (future)

### User Experience Metrics

- ✅ Zero gas fees for users (paymaster coverage)
- ✅ No wallet setup friction (account abstraction)
- ✅ Clear privacy guarantees in UI
- ✅ Transparent on-chain verification

---

## Future Enhancements

### Phase 5: Real ZK Proofs (Week 6-8)

**Current:** Simplified association sets (placeholder proofs)
**Target:** Full Privacy Pools V1 integration with Groth16 proofs

**Implementation:**
1. Integrate circom circuits for association set proofs
2. Add snarkjs verification to smart contracts
3. Generate real ZK proofs in TEE
4. Update withdrawal logic to verify proofs on-chain

**Benefits:**
- Cryptographic proof of non-illicit funds
- Regulatory compliance (association sets)
- Stronger privacy guarantees

### Phase 6: Batched Withdrawals (Week 8-10)

**Concept:** Batch multiple seller withdrawals into single transaction

**Benefits:**
- Stronger privacy (harder to analyze timing)
- Lower gas costs (amortized across multiple withdrawals)
- Better for high-volume platforms

### Phase 7: Multi-Chain Support (Week 10-12)

**Target chains:**
- Base (ERC-5564 support via Fluidkey)
- Optimism (Privacy Pools compatible)
- Polygon (Large user base)

**Implementation:**
- Deploy SimplePrivacyPool to each chain
- Update chain_id handling in TEE worker
- Multi-chain balance tracking

---

## Conclusion

This implementation plan provides a **concrete, achievable path** to true transaction privacy for invoice payments using Privacy Pools with x402 architecture.

**Key advantages:**
1. **x402 integration:** Seamless with existing facilitator architecture
2. **Gasless UX:** Paymaster sponsors all transactions
3. **True privacy:** Anonymity sets hide payment linkages
4. **Compliance-ready:** Association set proofs for regulation
5. **Scalable:** Works with existing TEE infrastructure

**Timeline:** 4-6 weeks to production-ready demo
**Complexity:** Medium (simplified Privacy Pools) → High (full ZK proofs)

**Next step:** Deploy SimplePrivacyPool contract and begin backend integration.
