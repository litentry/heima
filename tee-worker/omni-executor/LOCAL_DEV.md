# Local Development Guide

This guide provides instructions for running the omni-executor locally with fast development cycles.

## Quick Start

### 1. Build the Binary Locally

Build the omni-executor binary with the mock-server feature enabled:

```bash
cargo build --release --features mock-server
```

### 2. Run with Docker Compose

Start the full development environment:

```bash
cd docker/
docker compose build --no-cache omni-executor
docker compose up
```

The services will be available at:
- **Omni-Executor RPC**: http://localhost:2100
- **Mock Server**: http://localhost:3456
- **Ethereum Node (Anvil)**: http://localhost:8545
- **Heima Node**: ws://localhost:9944
- **Metrics**: http://localhost:9090

### Automatic AA Contract Deployment

The Docker setup now automatically deploys Account Abstraction contracts to the local Anvil node:

1. **aa-contracts-deploy** service compiles and deploys the AA contracts
2. Contract addresses are saved to a shared volume
3. **omni-executor** service loads these addresses automatically

Deployed contracts:
- **EntryPoint**: EIP-4337 EntryPoint contract
- **OmniAccountFactory**: Factory for creating OmniAccount instances
- **SimplePaymaster**: Paymaster contract for sponsoring transactions
- **TestToken**: USDC and USDT test tokens for development

## Mock Server

The mock server is automatically enabled and provides mocked responses for:
- PumpX API endpoints (`/pumpx/v3/account/*`)
- SendGrid email API
- Binance API
- Solana RPC
- EVM RPC

### Testing the Mock Server

Test the PumpX mock endpoints:

```bash
# Test get_account_user_id
curl "http://localhost:3456/pumpx/v3/account/get_account_user_id?email=test@example.com"

# Test user_connect (requires POST with JSON body)
curl -X POST "http://localhost:3456/pumpx/v3/account/user_connect" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","google_code":"123456","user_id":"test_user"}'
```

## Email Verification Flow

### 1. Request Email Verification Code

```bash
./target/release/omni-cli request-email-verification-code \
  --client-id "heima" \
  --user-email "user@example.com"
```

The verification code will be printed in the console logs (look for `VERIFICATION CODE: XXXXXX`).

### 2. Request JWT Token

Use the verification code from step 1. **Important**: Use a supported client ID (`heima`, `pumpx`, or `wildmeta`):

```bash
./target/release/omni-cli request-jwt \
  --client-id "heima" \
  --user-email "user@example.com" \
  --google-code "dummy_google_code" \
  --email-code "XXXXXX"
```

### 3. Submit User Operation

Use the **ID Token** (not access token) from step 2:

```bash
./target/release/omni-cli --token "YOUR.ID.TOKEN" submit-user-op \
  --sender "0xfb9c2a00066c1e5f259d9df1532475688e2f10e1" \
  --call-data "0x" \
  --init-code "9fe46736679d2d9a65f0992f2272de9f3c7fa6e00db21afe90622cdbc4aa82304e92d828ccc887e3ebfc03414d1a35026819c1b1ed726afd0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000a0ee7a142d267c1f36714e4a8f75612f20a7972000000000000000000000000000000000000000000000000000000000000000056865696d61000000000000000000000000000000000000000000000000000000" \
  --account-gas-limits "0x0000000000000000000000000003d0900000000000000000000000000000c350" \
  --pre-verification-gas "0x0000000000000000000000000000000000000000000000000000000000005208" \
  --gas-fees "0x00000000000000000000000003b9aca0000000000000000000000000b2d05e00" \
  --nonce 0x00 \
  --chain-id 31337
```

**Note**: The `submit-user-op` command requires:
- A valid **ID Token** (not access token) from step 2
- Supported client ID: `heima`, `pumpx`, or `wildmeta`
- Proper sender address format (0x prefixed, 40 hex characters)
- Chain ID that matches your target network (1 for Ethereum mainnet, 31337 for local dev)

**Expected Behavior**: With the automatic contract deployment, the command should now work properly. The system will:

1. Authenticate the user using the JWT token
2. Validate the user operation ownership against the deployed contracts
3. Process the user operation through the EIP-4337 EntryPoint

If you see "Failed to query OmniWallet owner", it means the wallet hasn't been created yet. You need to create an OmniAccount first using the deployed OmniAccountFactory contract.

### Viewing Deployed Contract Addresses

To see the deployed contract addresses, check the aa-contracts-deploy service logs:

```bash
docker compose logs aa-contracts-deploy
```

The addresses are also loaded into the omni-executor service environment variables:
- `OE_ENTRY_POINT_ADDRESS`: EntryPoint contract
- `OE_OMNI_FACTORY_ADDRESS`: OmniAccountFactory contract
- `OE_PAYMASTER_ADDRESS`: SimplePaymaster contract
- `OE_TEST_USDC_ADDRESS`: Test USDC token
- `OE_TEST_USDT_ADDRESS`: Test USDT token

## Development Workflow

### Fast Rebuild Cycle

1. Make code changes
2. Rebuild binary: `cargo build --release --features mock-server`
3. Restart Docker: `docker compose restart omni-executor`

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f omni-executor

# Filter for specific patterns
docker compose logs omni-executor | grep "mock-server"
docker compose logs omni-executor | grep "VERIFICATION CODE"
```

### Environment Variables

Key environment variables (set in `.env` file):

```bash
# Mock server configuration
OE_PUMPX_API_BASE_URL=http://omni-executor:3456/pumpx
# If you want to use console mailer type for HEIMA client id
OE_MAILER_TYPE_HEIMA=console

# Network endpoints
OE_PARENTCHAIN_URL=ws://heima-node:9944
OE_ETHEREUM_URL=http://ethereum-node:8545
OE_SOLANA_URL=https://api.devnet.solana.com

# Logging
RUST_LOG=debug
```

## Troubleshooting

### Mock Server Not Working

1. Check if the binary was built with mock-server feature:
   ```bash
   ./target/release/executor-worker --help | grep mock
   ```

2. Check if mock server is starting:
   ```bash
   docker compose logs omni-executor | grep -i mock
   ```

3. Verify mock server endpoints:
   ```bash
   curl -v http://localhost:3456/pumpx/v3/account/get_account_user_id?email=test@example.com
   ```

### Email Verification Issues

1. Check console logs for verification codes:
   ```bash
   docker compose logs omni-executor | grep "VERIFICATION CODE"
   ```

2. Verify mailer type is set to console:
   ```bash
   docker compose logs omni-executor | grep "ConsoleMailer"
   ```

### Build Issues

1. Clean and rebuild:
   ```bash
   cargo clean
   cargo build --release --features mock-server
   ```

2. Check feature flags:
   ```bash
   cargo build --release --features mock-server --verbose
   ```

## Docker-Free Development

For even faster development, you can run the omni-executor directly without Docker:

### Prerequisites

1. Start external services:
   ```bash
   # In separate terminals
   anvil --host 0.0.0.0 --block-time 1  # Ethereum node
   # Start heima-node separately
   ```

2. Set environment variables:
   ```bash
   export OE_PARENTCHAIN_URL=ws://localhost:9944
   export OE_ETHEREUM_URL=http://localhost:8545
   export OE_PUMPX_API_BASE_URL=http://localhost:3456/pumpx
   export OE_MAILER_TYPE_HEIMA=console
   export RUST_LOG=debug
   ```

3. Run omni-executor:
   ```bash
   ./target/release/executor-worker run --enable-mock-server --parentchain-sync
   ```

## Account Funding and EntryPoint Deposits

For EIP-4337 Account Abstraction to work properly, accounts need ETH for gas and deposits in the EntryPoint contract.

### 1. Fund Ethereum Accounts

Fund any Ethereum account with ETH using the default anvil private key:

```bash
# Fund an account with 10 ETH
cast send --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --value 10ether \
  --rpc-url http://localhost:8545 \
  0x5360456c51a6242B07f21C656a742081F52ffaae
```

### 2. Deposit to EntryPoint for UserOp Senders

UserOp sender accounts need deposits in the EntryPoint contract to pay for gas:

```bash
# Deposit 1 ETH to EntryPoint for a sender account
cast send --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --value 1ether \
  --rpc-url http://localhost:8545 \
  0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  "depositTo(address)" \
  0xfb9c2a00066c1e5f259d9df1532475688e2f10e1
```

### 3. Check Account Balance and Deposits

```bash
# Check ETH balance
cast balance YOUR_ACCOUNT_ADDRESS --rpc-url http://localhost:8545

# Check EntryPoint deposit balance
cast call --rpc-url http://localhost:8545 \
  0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  "balanceOf(address)" \
  YOUR_SENDER_ADDRESS
```

### Gas Limits for UserOps

When submitting UserOps with omni-cli, ensure proper gas limits for account creation:

```bash
# Correct gas limits for account creation (verificationGas: 250000, callGas: 50000)
--account-gas-limits "0x0000000000000000000000000003d0900000000000000000000000000000c350"
```

**Note**: A `verificationGasLimit` of 250,000 is required for account creation. Lower values (like 1) will cause account creation to fail.

## Additional Notes

- The mock server runs on port 3456 by default
- Console mailer will print verification codes to stdout
- All PumpX API calls are mocked and return successful responses
- The development setup uses in-memory storage that resets on restart
- EntryPoint contract address: `0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512`
- Default anvil private key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`