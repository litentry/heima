# Unified Hyperliquid RPC Method

This document demonstrates the unified approach for Hyperliquid signature generation using a single RPC endpoint.

## RPC Method: `omni_getHyperliquidSignatureData`

A single endpoint that handles all Hyperliquid operations through discriminated union parameters.

### Usage Examples

#### 1a. Approve Agent Wallet (Email Authentication)

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getHyperliquidSignatureData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "action_type": {
        "type": "approve_agent",
        "agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
        "agent_name": "My Trading Bot"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

#### 1b. Approve Agent Wallet (WildMeta Client Authentication)

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getHyperliquidSignatureData",
    "params": {
      "user_id": {"type": "evm", "value": "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e"},
      "client_id": "wildmeta",
      "client_auth": {
        "type": "wildmeta_hl",
        "value": {
          "agent_address": "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd",
          "business_json": "{\"action\":\"approve_agent\",\"agent_address\":\"0x742d35Cc6634C0532925a3b844Bc9e7595f02A10\",\"timestamp\":1752573555}",
          "main_address": "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e",
          "signature": "0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c",
          "login_type": 0
        }
      },
      "action_type": {
        "type": "approve_agent",
        "agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
        "agent_name": "My Trading Bot"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

#### 1c. Approve Agent Wallet (EVM Web3 Authentication)

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getHyperliquidSignatureData",
    "params": {
      "user_id": {"type": "evm", "value": "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e"},
      "user_auth": {"type": "evm", "value": "0x1234567890abcdef..."},
      "client_id": "heima",
      "action_type": {
        "type": "approve_agent",
        "agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
        "agent_name": "My Trading Bot"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

#### 2. Initiate Withdrawal

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getHyperliquidSignatureData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "action_type": {
        "type": "withdraw3",
        "amount": "100.0",
        "destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

#### 3. Approve Builder Fee

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getHyperliquidSignatureData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "action_type": {
        "type": "approve_builder_fee",
        "max_fee_rate": "0.01",
        "builder": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "result": {
    "main_address": "0x1234567890123456789012345678901234567890",
    "hyperliquid_signature_data": {
      "action": {
        "type": "approve_agent",
        "signatureChainId": "0xa4b1",
        "hyperliquidChain": "Mainnet",
        "agentAddress": "0x742d35cc6634c0532925a3b844bc9e7595f02a10",
        "agentName": "My Trading Bot",
        "nonce": 1700000000000
      },
      "nonce": 1700000000000,
      "signature": "0x1234567890abcdef..."
    }
  },
  "id": 1
}
```

## Parameters

### Common Parameters
- `user_id`: User identification (email, EVM address, etc.)
- `user_auth`: **Optional** - User authentication (email verification, Web3 signature, auth token, OAuth2, passkey)
- `client_id`: Client identifier (`heima`, `pumpx`, `wildmeta`)
- `client_auth`: **Optional** - Client authentication (WildMeta signature-based auth)
- `chain_id`: Target blockchain network ID
- `action_type`: Discriminated union for different operations

### Authentication Flow
The RPC method now supports **dual authentication modes**:

1. **User Authentication** (`user_auth` provided):
   - Email verification codes
   - Web3 signatures (EVM, Substrate, Solana, Bitcoin)
   - Auth tokens (JWT)
   - OAuth2 (Google)
   - Passkey authentication

2. **Client Authentication** (`client_auth` provided):
   - WildMeta signature-based authentication
   - Agent-to-main address linking verification
   - Timestamp-based replay protection

**Note**: At least one of `user_auth` OR `client_auth` must be provided. `user_auth` takes a privileged role.

### Action Types

#### `approve_agent`
- `agent_address`: Agent's Ethereum address (required)
- `agent_name`: Human-readable agent name (optional)

#### `withdraw3`
- `amount`: Withdrawal amount as string (required)
- `destination`: Destination wallet address (required)

#### `approve_builder_fee`
- `max_fee_rate`: Maximum fee rate as string (required)
- `builder`: Builder's wallet address (required)

## Features

### Security
- **TEE-Based Signing**: Private keys never leave trusted execution environment
- **Email Authentication**: Verification codes required before signing
- **EIP-712 Compliance**: Proper domain separation and type hashing
- **Replay Protection**: Timestamp-based nonces

### Technical
- **Chain Detection**: Automatic testnet vs mainnet detection
- **Address Validation**: Proper Ethereum address parsing
- **Error Handling**: Comprehensive error types and logging
- **Extensibility**: Easy to add new Hyperliquid action types

### Supported Chains
- **Mainnet**: Ethereum (1), BSC (56), Arbitrum (42161), HyperEVM (999)
- **Testnet**: Sepolia (11155111), BSC Testnet (97), Arbitrum Sepolia (421614), HyperEVM Testnet (998)

## EIP-712 Signatures

### Type Hashes
- **ApproveAgent**: `HyperliquidTransaction:ApproveAgent(string hyperliquidChain,address agentAddress,string agentName,uint64 nonce)`
- **Withdraw3**: `HyperliquidTransaction:Withdraw3(string hyperliquidChain,string amount,uint64 time,address destination)`
- **ApproveBuilderFee**: `HyperliquidTransaction:ApproveBuilderFee(string hyperliquidChain,string maxFeeRate,address builder,uint64 nonce)`

### Domain
- **Name**: `"HyperliquidSignTransaction"`
- **Version**: `"1"`
- **Chain ID**: Matches target network
- **Verifying Contract**: `Address::ZERO`

## Integration with Hyperliquid

The generated signatures are ready for direct use with Hyperliquid's API endpoints:
- **Approve Agent**: POST to `/exchange` with type `approveBuilderFee`
- **Withdraw3**: POST to `/exchange` with type `withdraw3`
- **Builder Fee**: POST to `/exchange` with type `approveBuilderFee`

## Advantages

✅ **Single Endpoint**: One RPC method for all Hyperliquid operations
✅ **Code Reuse**: Shared logic reduces duplication
✅ **Extensibility**: Trait-based design for easy additions
✅ **Type Safety**: Discriminated unions ensure correct parameters
✅ **Maintainability**: Centralized error handling and validation
✅ **Security**: Same TEE-based signing model throughout