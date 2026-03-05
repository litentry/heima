# Privacy Invoice Demo

A confidential multi-seller invoice system on Arbitrum Sepolia. Invoice amounts are TEE-encrypted; payments go into a privacy pool so buyer→seller links are unlinkable on-chain.

## Deployed Contracts (Arbitrum Sepolia)

| Contract | Address |
|---|---|
| EntryPoint | `0xe6042188857a822DDfcFE5fd9E17118049Ab539a` |
| OmniAccountFactory | `0xC099F3Cc3cA145546B502A8d8B2866283Aaf054e` |
| SimplePrivacyPool | `0x3921f08067D3316f1AF35a87937b6E719Fc98c8E` |
| PoolVerifier (Groth16) | `0x3706D75511518deaD16581a245e52b37212B743e` |
| SimplePaymaster | `0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912` |
| DemoUSDC | `0xc76aab3623dCac3939ae5D771598d51A560381c8` |

## Running Locally

### 1. Build and start the TEE binary

```bash
cd tee-worker/omni-executor
cargo build --release --features mock-server

# Wipe invoice state if needed (preserves bundler key)
rm -rf storage_db/

source .env
export OE_PRIVACY_POOL_ADDRESS=0x3921f08067D3316f1AF35a87937b6E719Fc98c8E
export OE_ETH_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
export OE_CIRCUIT_WASM_PATH=/path/to/contracts/privacy-pool/circuits/build/withdraw_js/withdraw.wasm
export OE_CIRCUIT_ZKEY_PATH=/path/to/contracts/privacy-pool/circuits/build/withdraw_final.zkey
./target/release/omni-executor run \
  --local-directory-path ./local \
  --enable-mock-server \
  --mock-server-port 3456
```

The RPC server starts on `ws://localhost:2100`.

### 2. Serve the frontend

```bash
cd webapp/privacy-invoice-demo
python3 -m http.server 8080
```

### 3. Mint DemoUSDC for the buyer's smart wallet

The buyer's smart wallet address is shown on the pay-invoice page after MetaMask connects. Fund it:

```bash
cast send 0xc76aab3623dCac3939ae5D771598d51A560381c8 \
  "mintFor(address,uint256)" <smart-wallet-address> 1000000000000000000000 \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --private-key <any-key>
```

The smart wallet needs no ETH — the SimplePaymaster covers gas.

## Demo Flow

### Create Invoice (Seller / billing admin)

Open `http://localhost:8080/create-invoice.html`

- Enter token address: `0xc76aab3623dCac3939ae5D771598d51A560381c8` (DemoUSDC)
- Enter your Ethereum address as **Invoice Creator**
- Add recipients: one row per payee, each with their address and amount
  - Example: Alice `0xAlice...` → 100 USDC, Bob `0xBob...` → 50 USDC
- Enter buyer email and description
- Click **Generate Confidential Invoice**

Result: invoice ID + pay link (for buyer) + withdraw link (for recipients).

### Pay Invoice (Buyer)

Open `http://localhost:8080/pay-invoice.html?id=<invoice_id>`

- Connect MetaMask (auto-switches to Arbitrum Sepolia)
- Page shows total amount due (decrypted by TEE)
- Click **Pay into Privacy Pool**
  - MetaMask: sign one UserOp containing `approve(total) + deposit_0 + deposit_1 + ...`
  - One pool commitment is created per recipient — no amounts visible on-chain

### Withdraw (Each Recipient)

Open `http://localhost:8080/withdraw-invoice.html?id=<invoice_id>`

- Connect MetaMask with the recipient's wallet
- Click **Withdraw** — the TEE matches the connected address to the correct sub-commitment and issues a nullifier
- MetaMask: sign a plain transaction calling `pool.withdraw(nullifier, amount, recipient, ...)`
- Funds arrive at the recipient's EOA

Each recipient withdraws independently. There is no on-chain link between the buyer's deposit and any recipient's withdrawal.

## Privacy Model

**What is hidden:** The link between buyer and individual recipients. Each deposit shows only a commitment hash (no amount, no recipient). Each withdrawal shows only a nullifier (no link to the deposit).

**What is visible:** Token transfers to/from the pool contract, and their amounts. Privacy is meaningful only when the pool has enough traffic that timing and amount correlation is hard. In this single-user demo, correlation is trivially possible — the design is correct, the anonymity set is just small.

**ZK proofs:** Currently using MockVerifier (accepts any proof). Real Groth16 proofs (Poseidon-based circuit) are planned for the next phase, which will require redeploying the pool contract.

## RPC Methods

| Method | Caller | Purpose |
|---|---|---|
| `omni_createConfidentialInvoice` | Seller | Encrypt amounts, create invoice with N recipients |
| `omni_getInvoiceDetails` | Buyer | Decrypt and show total amount |
| `omni_payInvoice` | Buyer (via pay page) | TEE generates N commitments + deposit calldatas |
| `omni_withdrawFromPool` | Recipient | TEE issues withdrawal calldata for their sub-commitment |

## Storage

- **RocksDB**: `tee-worker/omni-executor/storage_db/` — invoice state only. Safe to wipe.
- **Keystore**: `tee-worker/omni-executor/local/keystore/` — bundler key, AES key. Do NOT wipe.
