// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "./BasePaymaster.sol";
import "./UserOperationLib.sol";
import "../interfaces/PackedUserOperation.sol";
import "../interfaces/IPaymaster.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

/**
 * ERC20PaymasterV1 - A paymaster that allows users to pay gas fees with ERC20 tokens
 * while reimbursing bundlers with ETH. Only supports ERC20 token payments.
 * Only accepts operations from authorized bundlers for security.
 */
contract ERC20PaymasterV1 is BasePaymaster, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using UserOperationLib for PackedUserOperation;
    using Math for uint256;

    // Minimum required postOp gas limit to prevent paymaster losses
    uint256 private constant MIN_POST_OP_GAS_LIMIT = 50000;

    // Struct to decode paymaster data
    struct PaymasterData {
        address token; // ERC20 token address (must not be address(0))
        uint256 exchangeRate; // Exchange rate: how many token units per 1 wei of ETH
            // For tokens with different decimals, this should account for the difference
            // Example: For 6-decimal USDC at $2000/ETH: rate = 2000 * 10^6 = 2000000000
            // Example: For 18-decimal token at 1500:1 ratio: rate = 1500 * 10^18
            // Set to 0 for full sponsorship (no token charge)
        uint256 validUntil; // Timestamp until when this exchange rate is valid
        uint256 validAfter; // Timestamp after which this exchange rate is valid
    }

    // Context for postOp
    struct PostOpContext {
        address sender; // User's account address
        address token; // ERC20 token address
        uint256 exchangeRate; // Exchange rate used
        uint256 maxCost; // Maximum cost in wei
        uint256 prefundAmount; // Amount prefunded in tokens
        uint256 postOpGasLimit; // PostOp gas limit from paymasterAndData
        bool isApprovalOp; // Whether this was an approval operation
    }

    // Mapping of authorized bundler addresses
    mapping(address => bool) public authorizedBundlers;

    // Beneficiary address for collected ERC20 tokens (default: this contract)
    address public beneficiary;

    // Events
    event UserOpSponsored(address indexed account, address indexed token, uint256 actualGasCost, uint256 tokenAmount);
    event AuthorizedBundlerUpdated(address indexed bundler, bool authorized);
    event BeneficiaryUpdated(address indexed oldBeneficiary, address indexed newBeneficiary);
    event TokensWithdrawn(address indexed token, address indexed to, uint256 amount);

    // Errors
    error UnauthorizedBundler();
    error InsufficientDeposit();
    error PostOpGasLimitTooLow();
    error InvalidToken();
    error InsufficientTokenBalance();
    error InsufficientTokenAllowance();
    error InvalidExchangeRate();
    error TimestampValidationFailed();
    error InvalidPaymasterData();
    error TokenTransferFailed();
    error RefundFailed();

    constructor(IEntryPoint _entryPoint, address _initialBundler) BasePaymaster(_entryPoint) {
        authorizedBundlers[_initialBundler] = true;
        beneficiary = address(this);

        emit AuthorizedBundlerUpdated(_initialBundler, true);
        emit BeneficiaryUpdated(address(0), beneficiary);
    }

    /**
     * Validate a user operation and handle ERC20 token prefunding
     */
    function _validatePaymasterUserOp(PackedUserOperation calldata userOp, bytes32, /* userOpHash */ uint256 maxCost)
        internal
        override
        returns (bytes memory context, uint256 validationData)
    {
        // Check if the transaction is being submitted by an authorized bundler
        if (!authorizedBundlers[tx.origin]) {
            revert UnauthorizedBundler();
        }

        // Check if we have enough ETH deposit to cover the cost
        uint256 ourDeposit = entryPoint.balanceOf(address(this));
        if (ourDeposit < maxCost) {
            revert InsufficientDeposit();
        }

        // Decode paymaster data first to validate structure
        PaymasterData memory data = _decodePaymasterData(userOp.paymasterAndData);

        // Extract and validate postOp gas limit after data validation
        uint256 postOpGasLimit = userOp.unpackPostOpGasLimit();
        if (postOpGasLimit < MIN_POST_OP_GAS_LIMIT) {
            revert PostOpGasLimitTooLow();
        }

        // Validate timestamps
        uint256 currentTime = block.timestamp;
        if (currentTime < data.validAfter || currentTime > data.validUntil) {
            return ("", 1); // Reject with validation failure
        }

        // Validate token address - native tokens (address(0)) are not supported
        if (data.token == address(0)) {
            revert InvalidToken();
        }

        // Handle full sponsorship case (0 exchange rate means paymaster fully sponsors)
        bool isFullSponsorship = data.exchangeRate == 0;

        // Handle ERC20 token payment
        IERC20 token = IERC20(data.token);

        // Calculate required token amount with overflow protection
        uint256 requiredTokenAmount;
        if (isFullSponsorship) {
            requiredTokenAmount = 0; // No charge for full sponsorship
        } else {
            // Use Math.mulDiv to prevent overflow: (maxCost * exchangeRate) / 1e18
            requiredTokenAmount = maxCost.mulDiv(data.exchangeRate, 1e18, Math.Rounding.Ceil);
        }

        // Check user's token balance
        uint256 userBalance = token.balanceOf(userOp.sender);
        if (userBalance < requiredTokenAmount) {
            revert InsufficientTokenBalance();
        }

        // Check if the userOp is an approval transaction for this paymaster
        bool isApprovalOp = _isApprovalOperation(userOp, data.token);

        // For approval operations, still check balance to prevent "free approval" attacks
        // but skip allowance and prefunding checks since the approval is happening in this operation
        if (isApprovalOp) {
            // Even for approval ops, user must have sufficient balance to cover the gas
            // This prevents attackers from spamming free approval operations to drain paymaster
            if (userBalance < requiredTokenAmount) {
                revert InsufficientTokenBalance();
            }
        } else if (!isFullSponsorship) {
            // For non-approval operations with token charges, check allowance and prefund
            uint256 currentAllowance = token.allowance(userOp.sender, address(this));
            if (currentAllowance < requiredTokenAmount) {
                revert InsufficientTokenAllowance();
            }

            // Prefund by transferring tokens from user to beneficiary
            token.safeTransferFrom(userOp.sender, beneficiary, requiredTokenAmount);
        }

        PostOpContext memory postOpContext = PostOpContext({
            sender: userOp.sender,
            token: data.token,
            exchangeRate: data.exchangeRate,
            maxCost: maxCost,
            prefundAmount: requiredTokenAmount,
            postOpGasLimit: postOpGasLimit,
            isApprovalOp: isApprovalOp
        });

        return (abi.encode(postOpContext), 0);
    }

    /**
     * Post-operation handler to handle refunds and logging
     */
    function _postOp(
        IPaymaster.PostOpMode mode,
        bytes calldata context,
        uint256 actualGasCost,
        uint256 actualUserOpFeePerGas
    ) internal override nonReentrant {
        PostOpContext memory postOpContext = abi.decode(context, (PostOpContext));

        // Add postOp gas overhead to actual gas cost to prevent paymaster losses
        // The actualGasCost parameter doesn't include gas consumed by postOp itself
        // Use the postOp gas limit from paymasterAndData for accurate accounting
        uint256 totalGasCost = actualGasCost + (postOpContext.postOpGasLimit * actualUserOpFeePerGas);

        // Calculate token costs and refunds for ERC20 payments
        uint256 actualTokenCost;
        if (postOpContext.exchangeRate == 0) {
            actualTokenCost = 0; // Full sponsorship
        } else {
            // Use Math.mulDiv to prevent overflow, include postOp gas overhead
            actualTokenCost = totalGasCost.mulDiv(postOpContext.exchangeRate, 1e18, Math.Rounding.Ceil);
        }

        // Handle refunds/charges based on operation type
        if (mode == IPaymaster.PostOpMode.opSucceeded) {
            if (postOpContext.isApprovalOp) {
                // For approval operations, charge the user the actual token cost
                // since no prefunding occurred during validation
                if (actualTokenCost > 0) {
                    // Use low-level call to prevent revert on failed charge
                    (bool success,) = postOpContext.token.call(
                        abi.encodeWithSelector(
                            IERC20.transferFrom.selector, postOpContext.sender, beneficiary, actualTokenCost
                        )
                    );
                    if (!success) {
                        // Charge failed, but don't revert the entire operation
                        // The approval succeeded, but gas payment failed
                        emit UserOpSponsored(postOpContext.sender, postOpContext.token, totalGasCost, 0);
                        return;
                    }
                }
            } else if (postOpContext.prefundAmount > actualTokenCost) {
                // For non-approval operations, refund excess tokens
                uint256 refundAmount = postOpContext.prefundAmount - actualTokenCost;

                // Transfer refund from beneficiary back to user
                if (beneficiary == address(this)) {
                    // If beneficiary is this contract, we can refund directly
                    // Use low-level call to prevent revert on failed refund
                    (bool success,) = postOpContext.token.call(
                        abi.encodeWithSelector(IERC20.transfer.selector, postOpContext.sender, refundAmount)
                    );
                    if (!success) {
                        // Refund failed, but don't revert the entire operation
                        emit UserOpSponsored(
                            postOpContext.sender, postOpContext.token, totalGasCost, postOpContext.prefundAmount
                        );
                        return;
                    }
                }
                // If beneficiary is external, they need to handle their own refunds
            }
        }

        emit UserOpSponsored(postOpContext.sender, postOpContext.token, totalGasCost, actualTokenCost);
    }

    /**
     * Decode paymaster data from paymasterAndData field
     */
    function _decodePaymasterData(bytes calldata paymasterAndData) internal pure returns (PaymasterData memory data) {
        // paymasterAndData format:
        // paymaster_address (20) + validation_gas_limit (16) + postop_gas_limit (16) + paymaster_data
        // Our data starting at PAYMASTER_DATA_OFFSET (52): token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)
        if (paymasterAndData.length < PAYMASTER_DATA_OFFSET + 20 + 32 + 32 + 32) {
            revert InvalidPaymasterData();
        }

        bytes calldata paymasterData = paymasterAndData[PAYMASTER_DATA_OFFSET:];

        // Read 20 bytes token address, then 32 bytes each for other fields
        data.token = address(bytes20(paymasterData[0:20]));
        data.exchangeRate = uint256(bytes32(paymasterData[20:52]));
        data.validUntil = uint256(bytes32(paymasterData[52:84]));
        data.validAfter = uint256(bytes32(paymasterData[84:116]));
    }

    /**
     * Check if the user operation is an ERC20 approval operation for this paymaster
     *
     * In Account Abstraction, users call functions directly on the account contract.
     * For ERC20 approvals, the callData will be either:
     * 1. execute(tokenAddress, 0, approve(paymaster, amount))
     * 2. executeBatch([(tokenAddress, 0, approve(paymaster, amount))])
     */
    function _isApprovalOperation(PackedUserOperation calldata userOp, address tokenAddress)
        internal
        view
        returns (bool)
    {
        if (userOp.callData.length < 4) return false;

        bytes4 selector = bytes4(userOp.callData[0:4]);

        // Check if it's an execute() call that might contain approve
        if (selector == bytes4(keccak256("execute(address,uint256,bytes)"))) {
            return _checkExecuteApproval(userOp.callData, tokenAddress);
        }

        // Check if it's a batch execution containing approve
        // Using common batch execution selector from BaseAccount
        if (selector == bytes4(keccak256("executeBatch((address,uint256,bytes)[])"))) {
            return _checkBatchApproval(userOp.callData, tokenAddress);
        }

        return false;
    }

    /**
     * Check if execute callData contains approve(address(this), amount) to the correct token
     * Decodes: execute(tokenAddress, 0, approve(paymaster, amount))
     */
    function _checkExecuteApproval(bytes calldata callData, address tokenAddress) internal view returns (bool) {
        // execute(address,uint256,bytes) = 4 + 32 + 32 + 32 (offset) + 32 (length) + data
        if (callData.length < 132) return false; // Minimum length for execute with some data

        // Decode execute parameters: target (bytes 4:36), value (bytes 36:68), data offset (bytes 68:100)
        address target = address(bytes20(callData[16:36])); // Skip 4-byte selector + 12 bytes padding
        uint256 value = uint256(bytes32(callData[36:68]));

        // Check if target is the expected token and value is 0
        if (target != tokenAddress || value != 0) {
            return false;
        }

        // Get data offset and length
        uint256 dataOffset = uint256(bytes32(callData[68:100]));
        uint256 absoluteDataOffset = 4 + dataOffset; // Add 4 for function selector

        if (callData.length < absoluteDataOffset + 32) return false; // Need at least length field

        uint256 dataLength = uint256(bytes32(callData[absoluteDataOffset:absoluteDataOffset + 32]));
        uint256 dataStart = absoluteDataOffset + 32;

        if (callData.length < dataStart + dataLength || dataLength < 68) return false; // approve needs 4+32+32 bytes

        // Check if the inner data is approve(paymaster, amount)
        bytes4 innerSelector = bytes4(callData[dataStart:dataStart + 4]);
        if (innerSelector != IERC20.approve.selector) {
            return false;
        }

        // Check if spender is this paymaster
        address spender = address(bytes20(callData[dataStart + 16:dataStart + 36])); // Skip selector + 12 bytes padding
        return spender == address(this);
    }

    /**
     * Check if batch execution contains approve(address(this), amount) for the correct token
     * Decodes: executeBatch([(tokenAddress, 0, approve(paymaster, amount))])
     */
    function _checkBatchApproval(bytes calldata callData, address tokenAddress) internal view returns (bool) {
        // executeBatch((address,uint256,bytes)[]) = 4 + 32 (array offset) + 32 (array length) + Call structs
        if (callData.length < 100) return false; // Minimum for non-empty batch

        // Get array offset and length
        uint256 arrayOffset = uint256(bytes32(callData[4:36]));
        uint256 absoluteArrayOffset = 4 + arrayOffset;

        if (callData.length < absoluteArrayOffset + 32) return false;

        uint256 arrayLength = uint256(bytes32(callData[absoluteArrayOffset:absoluteArrayOffset + 32]));
        if (arrayLength == 0) return false;

        // For simplicity, only check if the first call is an approval to the correct token
        // Each Call struct has: target(32) + value(32) + data_offset(32) + data_length(32) + data
        uint256 firstCallOffset = absoluteArrayOffset + 32;

        if (callData.length < firstCallOffset + 96) return false; // Need at least target+value+offset

        address target = address(bytes20(callData[firstCallOffset + 12:firstCallOffset + 32]));
        uint256 value = uint256(bytes32(callData[firstCallOffset + 32:firstCallOffset + 64]));

        // Check if target is the expected token and value is 0
        if (target != tokenAddress || value != 0) {
            return false;
        }

        // Get data offset and length for the first call
        uint256 dataOffset = uint256(bytes32(callData[firstCallOffset + 64:firstCallOffset + 96]));
        uint256 absoluteDataOffset = firstCallOffset + dataOffset;

        if (callData.length < absoluteDataOffset + 32) return false;

        uint256 dataLength = uint256(bytes32(callData[absoluteDataOffset:absoluteDataOffset + 32]));
        uint256 dataStart = absoluteDataOffset + 32;

        if (callData.length < dataStart + dataLength || dataLength < 68) return false;

        // Check if the inner data is approve(paymaster, amount)
        bytes4 innerSelector = bytes4(callData[dataStart:dataStart + 4]);
        if (innerSelector != IERC20.approve.selector) {
            return false;
        }

        // Check if spender is this paymaster
        address spender = address(bytes20(callData[dataStart + 16:dataStart + 36]));
        return spender == address(this);
    }

    /**
     * Get token decimals for a given token (helper function for calculating exchange rates)
     * @param token The token address to query decimals for
     * @return decimals The number of decimals for the token, defaults to 18 if not available
     */
    function getTokenDecimals(address token) external view returns (uint8 decimals) {
        if (token == address(0)) {
            revert InvalidToken(); // Native tokens are not supported
        }

        // Check if the address has code
        if (token.code.length == 0) {
            return 18; // Not a contract, default to 18
        }

        try IERC20Metadata(token).decimals() returns (uint8 result) {
            return result;
        } catch {
            return 18; // Default to 18 if decimals() call fails
        }
    }

    /**
     * Calculate exchange rate for a token given how many tokens equal 1 ETH
     * @param tokenDecimals The number of decimals the token has
     * @param tokensPerEth How many tokens you get for 1 ETH (before decimal scaling)
     * @return exchangeRate The exchange rate to use in PaymasterData
     *
     * Example: USDC (6 decimals) where 1 ETH = 2000 USDC
     *          exchangeRate = tokensPerEth * 10^tokenDecimals
     *          exchangeRate = 2000 * 10^6 = 2000000000
     */
    function calculateExchangeRate(uint8 tokenDecimals, uint256 tokensPerEth)
        external
        pure
        returns (uint256 exchangeRate)
    {
        if (tokensPerEth == 0) {
            revert InvalidExchangeRate();
        }

        // exchangeRate = tokensPerEth * 10^tokenDecimals
        // This gives us how many token units per 1 ETH
        return tokensPerEth * (10 ** tokenDecimals);
    }

    /**
     * Add or remove an authorized bundler
     */
    function setAuthorizedBundler(address bundler, bool authorized) external onlyOwner {
        authorizedBundlers[bundler] = authorized;
        emit AuthorizedBundlerUpdated(bundler, authorized);
    }

    /**
     * Set the beneficiary address for collected ERC20 tokens
     */
    function setBeneficiary(address _beneficiary) external onlyOwner {
        require(_beneficiary != address(0), "Invalid beneficiary");
        address oldBeneficiary = beneficiary;
        beneficiary = _beneficiary;
        emit BeneficiaryUpdated(oldBeneficiary, _beneficiary);
    }

    /**
     * Withdraw accumulated ERC20 tokens (only if beneficiary is this contract)
     */
    function withdrawTokens(address token, address to, uint256 amount) external onlyOwner {
        require(beneficiary == address(this), "Beneficiary is not this contract");
        require(to != address(0), "Invalid recipient");

        // Only ERC20 tokens are supported for withdrawal
        if (token == address(0)) {
            revert InvalidToken();
        }

        // Withdraw ERC20
        IERC20(token).safeTransfer(to, amount);

        emit TokensWithdrawn(token, to, amount);
    }

    /**
     * Get the version of this paymaster contract
     */
    function version() public pure virtual returns (string memory) {
        return "1.0.0";
    }

    /**
     * Allow contract to receive ETH and automatically deposit to EntryPoint
     */
    receive() external payable {
        entryPoint.depositTo{value: msg.value}(address(this));
    }
    // Test helper function to expose _isApprovalOperation for testing

    function _isApprovalOperation_exposed(PackedUserOperation calldata userOp, address tokenAddress)
        external
        view
        returns (bool)
    {
        return _isApprovalOperation(userOp, tokenAddress);
    }
}
