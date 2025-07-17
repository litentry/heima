# Omni-Executor CLI

A command-line interface for testing omni-executor RPC methods, including `omni_submitUserOp` and `omni_requestJwt`.

## Building

```bash
cargo build --package omni-cli
```

## Usage

The CLI provides three main commands:
- `request-email-verification-code`: Request an email verification code
- `request-jwt`: Request a JWT token for authentication (requires email verification code)
- `submit-user-op`: Submit a UserOperation (requires JWT authentication)

### Basic Usage

#### Step 1: Request Email Verification Code

```bash
./target/debug/omni-cli request-email-verification-code \
  --client-id "test-client" \
  --user-email "user@example.com"
```

This will send a verification code to the specified email address.

#### Step 2: Request JWT Token

```bash
./target/debug/omni-cli request-jwt \
  --client-id "test-client" \
  --user-email "user@example.com" \
  --google-code "4/0AY0e-g7..." \
  --email-code "123456"  # Use the code from Step 1
```

This will return an access token that you can use for authenticated requests.

#### Step 3: Submit UserOperation

```bash
./target/debug/omni-cli \
  --token "your-jwt-token" \
  submit-user-op \
  --sender "0x1234567890123456789012345678901234567890" \
  --chain-id 1 \
  --wallet-index 0
```

### Command-line Options

Global options:
- `--endpoint <URL>`: RPC endpoint URL (default: `http://localhost:2100/`)
- `--token <TOKEN>`: Authentication token for protected methods

`request-email-verification-code` options:
- `--client-id <ID>`: OAuth client ID (required)
- `--user-email <EMAIL>`: User email address (required)

`request-jwt` options:
- `--client-id <ID>`: OAuth client ID (required)
- `--user-email <EMAIL>`: User email address (required)
- `--google-code <CODE>`: Google OAuth authorization code (required)
- `--email-code <CODE>`: Email verification code from Step 1 (required)
- `--invite-code <CODE>`: Invitation code (optional)
- `--language <LANG>`: Language preference (optional)

`submit-user-op` options:
- `--chain-id <ID>`: Chain ID (default: 1 for Ethereum mainnet)
- `--wallet-index <INDEX>`: Wallet index (default: 0)
- `--sender <ADDRESS>`: Sender address (hex string)
- `--nonce <VALUE>`: Nonce value (default: 0)
- `--init-code <DATA>`: Init code (hex bytes)
- `--call-data <DATA>`: Call data (hex bytes)  
- `--account-gas-limits <VALUE>`: Account gas limits (hex string, default: 0)
- `--pre-verification-gas <VALUE>`: Pre-verification gas (default: 0)
- `--gas-fees <VALUE>`: Gas fees (hex string, default: 0)
- `--paymaster-and-data <DATA>`: Paymaster and data (hex bytes)
- `--signature <DATA>`: Signature (hex bytes)

### Examples

#### Complete Authentication Flow

```bash
# Step 1: Request email verification code
./target/debug/omni-cli request-email-verification-code \
  --client-id "my-client-id" \
  --user-email "john.doe@example.com"

# Step 2: Request JWT token (after receiving email code)
./target/debug/omni-cli request-jwt \
  --client-id "my-client-id" \
  --user-email "john.doe@example.com" \
  --google-code "4/0AY0e-g7..." \
  --email-code "123456"

# Step 2 with optional parameters
./target/debug/omni-cli request-jwt \
  --client-id "my-client-id" \
  --user-email "john.doe@example.com" \
  --google-code "4/0AY0e-g7..." \
  --email-code "123456" \
  --invite-code "INVITE123" \
  --language "en"
```

#### Submit UserOperation with Authentication

```bash
./target/debug/omni-cli \
  --endpoint "http://localhost:2100/" \
  --token "your-jwt-token" \
  submit-user-op \
  --sender "0x1234567890123456789012345678901234567890" \
  --chain-id 1 \
  --wallet-index 0 \
  --nonce "0x0000000000000000000000000000000000000000000000000000000000000001" \
  --call-data "0x12345678" \
  --signature "0xabcdef"
```

#### Submit UserOperation with Custom Gas Parameters

```bash
./target/debug/omni-cli \
  --token "your-jwt-token" \
  submit-user-op \
  --sender "0x1234567890123456789012345678901234567890" \
  --chain-id 1 \
  --account-gas-limits "0x00000000000000000000000000000000000000000000000000000000000186a0" \
  --pre-verification-gas "0x5208" \
  --gas-fees "0x0000000000000000000000000000000000000000000000000000000000000000"
```

## Input Format

- **Addresses**: Hex strings (with or without 0x prefix)
- **Numeric values**: Can be provided as decimal numbers or hex strings
- **Variable-length data**: Hex strings of any length (with or without 0x prefix)

For `submit-user-op`, numeric values (nonce, pre_verification_gas) are converted from hex strings to u128 integers.

## Output

### JWT Request Output

Upon successful JWT request, the CLI will display:
- Access Token (use this with `--token` for authenticated requests)
- ID Token
- User ID (if available)
- Google Auth Check status

### UserOperation Output

The CLI will print the transaction hash if the operation succeeds:

```
Transaction Hash: 0x1234567890abcdef...
```

Or indicate if no transaction hash was returned:

```
No transaction hash returned
```

## Error Handling

The CLI will display detailed error messages including:
- HTTP errors (network issues, server errors)
- JSON-RPC errors (authentication failures, validation errors)
- Parsing errors (invalid hex values, etc.)

## Requirements

- The omni-executor must be running and accessible at the specified endpoint
- For protected methods, you must provide a valid authentication token
- The authenticated user must own the smart contract accounts referenced in the UserOperations

## Authentication Flow

1. **Request Email Verification**: First request a verification code to be sent to your email
2. **Get Google OAuth Code**: Obtain a Google OAuth authorization code (through Google OAuth2 flow)
3. **Request JWT**: Use both the email verification code and Google OAuth code to get a JWT token
4. **Use JWT for Protected Methods**: Include the JWT token with `--token` flag for authenticated requests

## Notes

- This CLI is primarily for testing and development purposes
- All hex values are case-insensitive
- Chain IDs are now represented as u64 values (e.g., 1 for Ethereum mainnet)
- Multiple UserOperations can be submitted in a single batch (currently limited to one in CLI)
- JWT tokens are required for all protected methods (like submit-user-op)
- Email verification codes expire after a certain time period
- Google OAuth codes must be fresh and unused