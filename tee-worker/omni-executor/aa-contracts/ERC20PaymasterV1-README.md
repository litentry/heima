# ERC20PaymasterV1

A secure, feature-rich paymaster contract that allows users to pay gas fees with ERC20 tokens while reimbursing bundlers with ETH. This paymaster supports both native token payments and arbitrary ERC20 token payments with configurable exchange rates.

## Design Concept

### Core Functionality
- **Dual Payment Support**: Accepts both native tokens (ETH, BNB, etc.) and ERC20 tokens (USDC, USDT, etc.)
- **Bundler Authorization**: Only processes operations from pre-authorized bundlers for security
- **Dynamic Exchange Rates**: Supports time-bounded exchange rate configurations
- **Configurable Beneficiary**: ERC20 tokens can be collected by the paymaster or a designated address
- **Automatic Refunds**: Calculates and refunds unused tokens to users after operation execution

### Security Features
- **Access Control**: Inherits from `BasePaymaster` with owner-only administrative functions
- **Reentrancy Protection**: Uses `ReentrancyGuard` to prevent reentrancy attacks
- **Safe Token Transfers**: Utilizes OpenZeppelin's `SafeERC20` for secure token operations
- **Bundler Validation**: Only authorized bundlers can submit operations
- **Balance & Allowance Checks**: Validates user's token balance and approval before accepting operations

## Architecture

```
ERC20PaymasterV1
├── Inherits from BasePaymaster (stake management, EntryPoint integration)
├── Uses SafeERC20 (secure token transfers)
├── Uses ReentrancyGuard (reentrancy protection)
└── Implements IPaymaster interface
```

### Key Components

#### PaymasterData Structure
```solidity
struct PaymasterData {
    address token;          // ERC20 token address (address(0) for native)
    uint256 exchangeRate;   // Wei of token per 1 wei of ETH (scaled by 1e18)
    uint256 validUntil;     // Timestamp until when rate is valid
    uint256 validAfter;     // Timestamp after which rate is valid
}
```

#### PostOpContext Structure
```solidity
struct PostOpContext {
    address sender;         // User's account address
    address token;          // Token address used
    uint256 exchangeRate;   // Exchange rate applied
    uint256 maxCost;        // Maximum cost in wei
    uint256 prefundAmount;  // Amount prefunded in tokens
}
```

## Practical Usage

### 1. Deploying the Contract

```solidity
// Deploy with initial bundler authorization
ERC20PaymasterV1 paymaster = new ERC20PaymasterV1(
    entryPointAddress,
    initialBundlerAddress
);
```

### 2. Understanding Token Decimals and Exchange Rates

**Critical**: Exchange rates must account for token decimals to work correctly.

#### Token Decimals Impact
- **ETH**: 18 decimals (1 ETH = 1e18 wei)
- **USDC**: 6 decimals (1 USDC = 1e6 units)
- **USDT**: 6 decimals (1 USDT = 1e6 units)
- **DAI**: 18 decimals (1 DAI = 1e18 units)

#### Exchange Rate Calculation
The `exchangeRate` represents **how many token units per 1 wei of ETH**.

**Formula**: `exchangeRate = (ETH_price_in_USD / token_price_in_USD) * (10 ^ token_decimals)`

**Examples**:

1. **USDC (6 decimals) at $2000 ETH, $1 USDC**:
   ```
   exchangeRate = (2000 / 1) * 10^6 = 2,000,000,000 USDC units per 1 ETH
   ```

2. **DAI (18 decimals) at $2000 ETH, $1 DAI**:
   ```
   exchangeRate = (2000 / 1) * 10^18 = 2,000,000,000,000,000,000,000 DAI units per 1 ETH
   ```

3. **Custom 8-decimal token at $2000 ETH, $0.50 per token**:
   ```
   exchangeRate = (2000 / 0.50) * 10^8 = 400,000,000,000 token units per 1 ETH
   ```

#### Helper Functions
The paymaster provides helper functions for exchange rate calculation:

```javascript
// Get token decimals
const decimals = await paymaster.getTokenDecimals(tokenAddress);

// Calculate exchange rate
const exchangeRate = await paymaster.calculateExchangeRate(
    decimals,
    tokenPriceInWei // Price of 1 token in wei (e.g., 0.0005e18 for $1 USDC when ETH is $2000)
);
```

### 3. Encoding paymasterAndData

The `paymasterAndData` field must be encoded with the paymaster address and payment metadata:

```javascript
function encodePaymasterData(
    paymasterAddress,
    tokenAddress,      // address(0) for native token
    exchangeRate,      // Token units per 1 wei of ETH (see calculation above)
    validUntil,        // Unix timestamp
    validAfter         // Unix timestamp
) {
    return ethers.utils.solidityPack(
        ['address', 'bytes12', 'address', 'uint256', 'uint256', 'uint256'],
        [
            paymasterAddress,      // 20 bytes
            '0x000000000000000000000000', // 12 bytes padding  
            tokenAddress,          // 20 bytes
            exchangeRate,          // 32 bytes
            validUntil,           // 32 bytes
            validAfter            // 32 bytes
        ]
    );
}
```

### 4. Example Usage Scenarios

#### Native Token Payment (ETH/BNB)
```javascript
const paymasterAndData = encodePaymasterData(
    '0x1234...paymaster',
    '0x0000000000000000000000000000000000000000', // Native token
    ethers.utils.parseEther('1'),  // 1:1 rate (can be any rate)
    Math.floor(Date.now() / 1000) + 3600, // Valid for 1 hour
    Math.floor(Date.now() / 1000)         // Valid from now
);
```

#### ERC20 Token Payment (USDC at $2000 ETH)
```javascript
// Method 1: Manual calculation
const usdcExchangeRate = 2000 * (10 ** 6); // 2000 USDC per ETH, 6 decimals
const paymasterAndData = encodePaymasterData(
    '0x1234...paymaster',
    '0xA0b86a33E6441E8fd796Fa2b0E8024F5e28D0C70', // USDC address
    usdcExchangeRate,  // 2,000,000,000 USDC units per 1 ETH
    Math.floor(Date.now() / 1000) + 3600, // Valid for 1 hour
    Math.floor(Date.now() / 1000)         // Valid from now
);

// Method 2: Using helper functions
const decimals = await paymaster.getTokenDecimals(usdcAddress);
const tokenPriceInEth = ethers.utils.parseEther('0.0005'); // $1 USDC at $2000 ETH
const exchangeRate = await paymaster.calculateExchangeRate(decimals, tokenPriceInEth);
const paymasterAndData = encodePaymasterData(
    paymasterAddress,
    usdcAddress,
    exchangeRate,
    validUntil,
    validAfter
);
```

#### Full Sponsorship (No Token Charge)
```javascript
const paymasterAndData = encodePaymasterData(
    '0x1234...paymaster',
    '0xA0b86a33E6441E8fd796Fa2b0E8024F5e28D0C70', // Any token address
    0,  // Zero exchange rate = full sponsorship
    Math.floor(Date.now() / 1000) + 3600,
    Math.floor(Date.now() / 1000)
);
```

### 5. User Operation Flow

#### For ERC20 Payments:
1. **User Preparation**: User must have sufficient ERC20 tokens and approve the paymaster
2. **Operation Submission**: Bundler submits UserOp with encoded paymaster data
3. **Validation**: Paymaster validates bundler, checks balances/allowances, prefunds tokens
4. **Execution**: EntryPoint executes the user operation
5. **Post-Operation**: Paymaster calculates actual cost and refunds excess tokens

#### For Native Token Payments:
1. **User Preparation**: User account should have native tokens and implement a `sendETH` function
2. **Operation Submission**: Bundler submits UserOp with native token paymaster data  
3. **Validation**: Paymaster validates bundler, exchange rate, and user's native token balance
4. **Execution**: EntryPoint executes the user operation (paymaster covers gas cost)
5. **Post-Operation**: Paymaster collects payment from user account by calling `sendETH(beneficiary, amount)`

**Why Native Token Payment Collection is Different**: Unlike ERC20 tokens which can be transferred during validation using `transferFrom`, native tokens cannot be \"pulled\" from an account. Instead, the user account must actively send the tokens. This is handled in `postOp` by calling the user account's `sendETH` function. If this call fails, the payment failure is logged but doesn't revert the user operation. Native tokens don't need refunds because payment collection happens after the actual gas cost is known.

#### For Full Sponsorship (exchangeRate = 0):
1. **User Preparation**: No token preparation needed
2. **Operation Submission**: Bundler submits UserOp with zero exchange rate
3. **Validation**: Paymaster validates bundler only, no token operations
4. **Execution**: EntryPoint executes the user operation
5. **Post-Operation**: Paymaster fully covers the gas cost, no charges to user

### 6. Administrative Functions

#### Managing Authorized Bundlers
```solidity
// Add authorized bundler
paymaster.setAuthorizedBundler(bundlerAddress, true);

// Remove authorized bundler
paymaster.setAuthorizedBundler(bundlerAddress, false);
```

#### Setting Beneficiary
```solidity
// Set custom beneficiary for ERC20 token collection
paymaster.setBeneficiary(treasuryAddress);

// Reset to paymaster contract (enables withdrawTokens)
paymaster.setBeneficiary(address(paymaster));
```

#### Withdrawing Tokens
```solidity
// Withdraw ERC20 tokens (only if beneficiary is the paymaster)
paymaster.withdrawTokens(tokenAddress, recipientAddress, amount);

// Withdraw ETH
paymaster.withdrawTokens(address(0), recipientAddress, amount);
```

## Security Considerations

### Arithmetic Overflow Protection
- **Uses `Math.mulDiv`**: Prevents overflow in `(maxCost * exchangeRate)` calculations
- **Safe token transfers**: Uses `SafeERC20` for all token operations
- **Rounding protection**: Uses ceiling rounding to ensure sufficient token charges

### Attack Prevention
- **No "Free Approval" attacks**: Even approval operations require sufficient token balance to cover gas
- **Bundler authorization**: Only authorized bundlers can submit operations
- **Balance validation**: Always checks user token balance before accepting operations
- **Allowance validation**: Verifies sufficient allowance for non-approval operations

### Token Decimal Handling
- **Automatic decimals detection**: Helper functions handle different token decimals
- **Precision-aware calculations**: Exchange rates account for token decimal differences
- **Overflow-safe math**: Large exchange rate calculations use protected arithmetic

### For Integrators
- Always validate exchange rates from trusted sources
- Set reasonable validity timeframes for exchange rates
- Monitor for sudden rate changes or manipulation attempts
- Ensure proper allowance management in user interfaces
- Use the helper functions for exchange rate calculations to avoid decimal errors

### For Users
- Only approve the minimum necessary token amounts
- Verify paymaster addresses before approving tokens
- Monitor transaction fees and exchange rates
- Be aware that refunds depend on the beneficiary configuration
- **For native token payments**: Ensure your account contract implements a `sendETH(address,uint256)` function\n- Understand that native token payment collection happens after operation execution

### For Bundlers
- Only submit operations from authorized addresses
- Validate paymaster data encoding before submission
- Monitor for failed operations due to insufficient allowances
- Ensure adequate paymaster deposits for gas reimbursement
- Understand that full sponsorship (exchangeRate = 0) is valid and intentional

## Version Information

- **Contract Version**: 1.0.0
- **Solidity Version**: ^0.8.28
- **ERC4337 Compatibility**: v0.7 (PackedUserOperation)

## Events

- `UserOpSponsored`: Emitted when an operation is sponsored
- `AuthorizedBundlerUpdated`: Emitted when bundler authorization changes  
- `BeneficiaryUpdated`: Emitted when beneficiary address changes
- `TokensWithdrawn`: Emitted when tokens are withdrawn by owner\n- `NativeTokenPaymentFailed`: Emitted when native token payment collection fails

## Error Handling

The contract uses custom errors for gas efficiency:
- `UnauthorizedBundler`: Bundler not authorized
- `InsufficientDeposit`: Paymaster lacks ETH deposit
- `InvalidToken`: Token address validation failed
- `InsufficientTokenBalance`: User lacks required tokens
- `InsufficientTokenAllowance`: User hasn't approved enough tokens
- `InvalidExchangeRate`: Exchange rate is zero
- `TimestampValidationFailed`: Operation outside valid time window
- `InvalidPaymasterData`: Malformed paymaster data
- `TokenTransferFailed`: ERC20 transfer failed
- `RefundFailed`: Token refund failed