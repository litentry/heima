# Privacy Invoice Demo

A confidential multi-seller invoice system on Arbitrum Sepolia. Invoice amounts are TEE-encrypted; payments go into a privacy pool so buyer→seller links are unlinkable on-chain.

## Deployed Contracts (Arbitrum Sepolia)

| Contract | Address |
|---|---|
| EntryPoint | `0xe6042188857a822DDfcFE5fd9E17118049Ab539a` |
| OmniAccountFactory | `0xC099F3Cc3cA145546B502A8d8B2866283Aaf054e` |
| SimplePrivacyPool | `0xcdBc5c8C4A8c475712627cB5476edbede5cFf860` |
| PoolVerifier (Groth16) | `0x9cfa2C1f4c84136a76823CC1886AE373174A8E75` |
| SimplePaymaster | `0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912` |
| DemoUSDC | `0xE7ECbb383987E374ceE676c89C81F41f6EEDC9fA` |

## Building from Source

### ZK Artifacts (circuit + proving key)

Pre-built artifacts are committed at `contracts/privacy-pool/circuits/build/`. To regenerate from scratch (e.g. after changing the circuit):

**Prerequisites:** [circom 2.x](https://docs.circom.io/getting-started/installation/) (install from source via `cargo install`), [snarkjs](https://github.com/iden3/snarkjs) (`npm install -g snarkjs`).

```bash
cd contracts/privacy-pool

# Compile circuit
circom circuits/withdraw.circom --r1cs --wasm --sym -o circuits/build/

# Trusted setup — phase 1 (BN128, 2^14 = 16k constraints sufficient for this circuit)
snarkjs powersoftau new bn128 14 circuits/build/pot14_0000.ptau -v
snarkjs powersoftau contribute circuits/build/pot14_0000.ptau circuits/build/pot14_0001.ptau --name="contributor"
snarkjs powersoftau prepare phase2 circuits/build/pot14_0001.ptau circuits/build/pot14_final.ptau -v

# Trusted setup — phase 2
snarkjs groth16 setup circuits/build/withdraw.r1cs circuits/build/pot14_final.ptau circuits/build/withdraw_0000.zkey
snarkjs zkey contribute circuits/build/withdraw_0000.zkey circuits/build/withdraw_final.zkey --name="contributor"
snarkjs zkey export verificationkey circuits/build/withdraw_final.zkey circuits/build/verification_key.json

# Export Solidity verifier (overwrites src/Verifier.sol — redeploy after)
snarkjs zkey export solidityverifier circuits/build/withdraw_final.zkey src/Verifier.sol
```

### Contract Deployment

**Prerequisites:** [Foundry](https://book.getfoundry.sh/getting-started/installation).

```bash
cd contracts/privacy-pool

# Install Solidity dependencies
forge install

# Run tests
forge test

# Deploy to Arbitrum Sepolia
forge script script/Deploy.s.sol \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --private-key <deployer-key> \
  --broadcast
```

The script deploys `DemoUSDC`, `PoolVerifier` (wraps the Groth16 verifier), and `SimplePrivacyPool`. Update the addresses in this README and in `create-invoice.html` after redeployment.

### TEE Binary

**Prerequisites:** Rust stable toolchain.

```bash
cd tee-worker/omni-executor
cargo build --release --features mock-server
```

## Running Locally

### 1. Start the TEE binary

```bash
cd tee-worker/omni-executor

# Wipe invoice state if needed (preserves bundler key and AES key)
rm -rf storage_db/

source .env
export OE_PRIVACY_POOL_ADDRESS=0xcdBc5c8C4A8c475712627cB5476edbede5cFf860
export OE_ETH_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
export OE_CIRCUIT_WASM_PATH=$(pwd)/contracts/privacy-pool/circuits/build/withdraw_js/withdraw.wasm
export OE_CIRCUIT_ZKEY_PATH=$(pwd)/contracts/privacy-pool/circuits/build/withdraw_final.zkey
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

**ZK proofs:** Real Groth16 proofs using a Poseidon-based circom circuit (depth-20 Merkle tree). The TEE generates the proof at withdrawal time by querying the on-chain Merkle state and running `ark-groth16` over the `withdraw_final.zkey` proving key. The pool contract verifies via a snarkjs-generated `Groth16Verifier`.

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
