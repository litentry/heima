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

### 2. Encoding paymasterAndData

The `paymasterAndData` field must be encoded with the paymaster address and payment metadata:

```javascript
function encodePaymasterData(
    paymasterAddress,
    tokenAddress,      // address(0) for native token
    exchangeRate,      // e.g., 1500000000000000000000 for 1500 USDC per 1 ETH
    validUntil,        // Unix timestamp
    validAfter         // Unix timestamp
) {
    return ethers.utils.solidityPack(
        ['address', 'address', 'uint256', 'uint256', 'uint256'],
        [
            paymasterAddress,
            tokenAddress.padStart(64, '0'), // Pad to 32 bytes
            exchangeRate,
            validUntil,
            validAfter
        ]
    );
}
```

### 3. Example Usage Scenarios

#### Native Token Payment (ETH/BNB)
```javascript
const paymasterAndData = encodePaymasterData(
    '0x1234...paymaster',
    '0x0000000000000000000000000000000000000000', // Native token
    ethers.utils.parseEther('1'),  // 1:1 rate
    Math.floor(Date.now() / 1000) + 3600, // Valid for 1 hour
    Math.floor(Date.now() / 1000)         // Valid from now
);
```

#### ERC20 Token Payment (USDC)
```javascript
const paymasterAndData = encodePaymasterData(
    '0x1234...paymaster',
    '0xA0b86a33E6441E8fd796Fa2b0E8024F5e28D0C70', // USDC address
    ethers.utils.parseUnits('1500', 18),  // 1500 USDC per 1 ETH
    Math.floor(Date.now() / 1000) + 3600, // Valid for 1 hour
    Math.floor(Date.now() / 1000)         // Valid from now
);
```

### 4. User Operation Flow

#### For ERC20 Payments:
1. **User Preparation**: User must have sufficient ERC20 tokens and approve the paymaster
2. **Operation Submission**: Bundler submits UserOp with encoded paymaster data
3. **Validation**: Paymaster validates bundler, checks balances/allowances, prefunds tokens
4. **Execution**: EntryPoint executes the user operation
5. **Post-Operation**: Paymaster calculates actual cost and refunds excess tokens

#### For Native Token Payments:
1. **User Preparation**: User account should have native tokens (validation happens during execution)
2. **Operation Submission**: Bundler submits UserOp with native token paymaster data
3. **Validation**: Paymaster validates bundler and exchange rate
4. **Execution**: EntryPoint executes operation, charges user's account directly
5. **Post-Operation**: Paymaster logs the sponsored operation

### 5. Administrative Functions

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

### For Integrators
- Always validate exchange rates from trusted sources
- Set reasonable validity timeframes for exchange rates
- Monitor for sudden rate changes or manipulation attempts
- Ensure proper allowance management in user interfaces

### For Users
- Only approve the minimum necessary token amounts
- Verify paymaster addresses before approving tokens
- Monitor transaction fees and exchange rates
- Be aware that refunds depend on the beneficiary configuration

### For Bundlers
- Only submit operations from authorized addresses
- Validate paymaster data encoding before submission
- Monitor for failed operations due to insufficient allowances
- Ensure adequate paymaster deposits for gas reimbursement

## Version Information

- **Contract Version**: 1.0.0
- **Solidity Version**: ^0.8.28
- **ERC4337 Compatibility**: v0.7 (PackedUserOperation)

## Events

- `UserOpSponsored`: Emitted when an operation is sponsored
- `AuthorizedBundlerUpdated`: Emitted when bundler authorization changes  
- `BeneficiaryUpdated`: Emitted when beneficiary address changes
- `TokensWithdrawn`: Emitted when tokens are withdrawn by owner

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