# Hyperliquid RPC Methods Usage Examples

This document demonstrates how to use both the **separate** and **unified** approaches for Hyperliquid signature generation.

## Approach 1: Separate RPC Methods

### 1. Approve Agent Wallet

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getApproveAgentWalletData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
      "agent_name": "My Trading Bot",
      "chain_id": 42161
    },
    "id": 1
  }'
```

### 2. Initiate Withdrawal

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getInitiateWithdrawalData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "amount": "100.0",
      "destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
      "chain_id": 42161
    },
    "id": 1
  }'
```

### 3. Approve Builder Fee

```bash
curl -X POST http://localhost:2100 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "omni_getApproveBuilderFeeData",
    "params": {
      "user_id": {"type": "email", "value": "user@example.com"},
      "user_auth": {"type": "email", "value": "123456"},
      "client_id": "heima",
      "max_fee_rate": "0.01",
      "builder": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
      "chain_id": 42161
    },
    "id": 1
  }'
```

## Approach 2: Unified RPC Method

### 1. Approve Agent Wallet

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

### 2. Initiate Withdrawal

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
        "type": "withdraw",
        "amount": "100.0",
        "destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
      },
      "chain_id": 42161
    },
    "id": 1
  }'
```

### 3. Approve Builder Fee

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

## Response Format Comparison

### Separate Method Response
```json
{
  "jsonrpc": "2.0",
  "result": {
    "main_address": "0x1234567890123456789012345678901234567890",
    "approve_agent_data": {
      "action": {
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

### Unified Method Response
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

## Pros and Cons

### Separate Methods Approach

**Pros:**
- ✅ Type-safe parameters (no optional fields)
- ✅ Clear method names for each action
- ✅ Simpler parameter validation
- ✅ Easier to document and understand
- ✅ Better IDE autocompletion support

**Cons:**
- ❌ Code duplication across methods
- ❌ More RPC endpoints to maintain
- ❌ Requires separate registration for each method

### Unified Method Approach

**Pros:**
- ✅ Single endpoint for all Hyperliquid operations
- ✅ Shared code and logic reduces duplication
- ✅ Easier to add new action types
- ✅ More consistent error handling
- ✅ Trait-based implementation for extensibility

**Cons:**
- ❌ More complex parameter structure
- ❌ Optional fields based on action type
- ❌ Requires pattern matching for different actions
- ❌ Potentially confusing for API consumers

## Recommendation

For **production use**, consider:
- **Separate methods** if you prioritize API clarity and type safety
- **Unified method** if you prioritize code maintainability and extensibility

Both approaches maintain the same security model with **TEE-based signing** and **EIP-712 standards compliance**.