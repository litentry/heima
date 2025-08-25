// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "./BasePaymaster.sol";
import "../interfaces/PackedUserOperation.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * ERC20PaymasterV1 - A paymaster that allows users to pay gas fees with ERC20 tokens
 * while reimbursing bundlers with ETH. Supports both native token and ERC20 payments.
 * Only accepts operations from authorized bundlers for security.
 */
contract ERC20PaymasterV1 is BasePaymaster, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using UserOperationLib for PackedUserOperation;

    // Struct to decode paymaster data
    struct PaymasterData {
        address token;          // ERC20 token address (address(0) for native token)
        uint256 exchangeRate;   // Exchange rate: how many wei of token per 1 wei of ETH (scaled by 1e18)
        uint256 validUntil;     // Timestamp until when this exchange rate is valid
        uint256 validAfter;     // Timestamp after which this exchange rate is valid
    }

    // Context for postOp
    struct PostOpContext {
        address sender;         // User's account address
        address token;          // Token address (address(0) for native)
        uint256 exchangeRate;   // Exchange rate used
        uint256 maxCost;        // Maximum cost in wei
        uint256 prefundAmount;  // Amount prefunded in tokens
    }

    // Mapping of authorized bundler addresses
    mapping(address => bool) public authorizedBundlers;

    // Beneficiary address for collected ERC20 tokens (default: this contract)
    address public beneficiary;

    // Events
    event UserOpSponsored(
        address indexed account,
        address indexed token,
        uint256 actualGasCost,
        uint256 tokenAmount
    );
    event AuthorizedBundlerUpdated(address indexed bundler, bool authorized);
    event BeneficiaryUpdated(address indexed oldBeneficiary, address indexed newBeneficiary);
    event TokensWithdrawn(address indexed token, address indexed to, uint256 amount);

    // Errors
    error UnauthorizedBundler();
    error InsufficientDeposit();
    error InvalidToken();
    error InsufficientTokenBalance();
    error InsufficientTokenAllowance();
    error InvalidExchangeRate();
    error TimestampValidationFailed();
    error InvalidPaymasterData();
    error TokenTransferFailed();
    error RefundFailed();

    constructor(
        IEntryPoint _entryPoint,
        address _initialBundler
    ) BasePaymaster(_entryPoint) {
        authorizedBundlers[_initialBundler] = true;
        beneficiary = address(this);
        
        emit AuthorizedBundlerUpdated(_initialBundler, true);
        emit BeneficiaryUpdated(address(0), beneficiary);
    }

    /**
     * Validate a user operation and handle ERC20 token prefunding
     */
    function _validatePaymasterUserOp(
        PackedUserOperation calldata userOp,
        bytes32 /* userOpHash */,
        uint256 maxCost
    ) internal override returns (bytes memory context, uint256 validationData) {
        // Check if the transaction is being submitted by an authorized bundler
        if (!authorizedBundlers[tx.origin]) {
            revert UnauthorizedBundler();
        }

        // Check if we have enough ETH deposit to cover the cost
        uint256 ourDeposit = entryPoint.balanceOf(address(this));
        if (ourDeposit < maxCost) {
            revert InsufficientDeposit();
        }

        // Decode paymaster data
        PaymasterData memory data = _decodePaymasterData(userOp.paymasterAndData);

        // Validate timestamps
        uint256 currentTime = block.timestamp;
        if (currentTime < data.validAfter || currentTime > data.validUntil) {
            return ("", 1); // Reject with validation failure
        }

        // Validate exchange rate
        if (data.exchangeRate == 0) {
            revert InvalidExchangeRate();
        }

        // Handle native token payment (no prefunding needed)
        if (data.token == address(0)) {
            PostOpContext memory postOpContext = PostOpContext({
                sender: userOp.sender,
                token: address(0),
                exchangeRate: data.exchangeRate,
                maxCost: maxCost,
                prefundAmount: 0
            });
            return (abi.encode(postOpContext), 0);
        }

        // Handle ERC20 token payment
        IERC20 token = IERC20(data.token);
        
        // Calculate required token amount (maxCost * exchangeRate / 1e18)
        uint256 requiredTokenAmount = (maxCost * data.exchangeRate) / 1e18;
        
        // Check user's token balance
        uint256 userBalance = token.balanceOf(userOp.sender);
        if (userBalance < requiredTokenAmount) {
            revert InsufficientTokenBalance();
        }

        // Check if the userOp is an approval transaction
        bool isApprovalOp = _isApprovalOperation(userOp, data.token);
        
        // If not an approval operation, check current allowance
        if (!isApprovalOp) {
            uint256 currentAllowance = token.allowance(userOp.sender, address(this));
            if (currentAllowance < requiredTokenAmount) {
                revert InsufficientTokenAllowance();
            }
        }

        // Prefund by transferring tokens from user to beneficiary
        if (!isApprovalOp) {
            try token.safeTransferFrom(userOp.sender, beneficiary, requiredTokenAmount) {
                // Success - tokens transferred
            } catch {
                revert TokenTransferFailed();
            }
        }

        PostOpContext memory postOpContext = PostOpContext({
            sender: userOp.sender,
            token: data.token,
            exchangeRate: data.exchangeRate,
            maxCost: maxCost,
            prefundAmount: requiredTokenAmount
        });

        return (abi.encode(postOpContext), 0);
    }

    /**
     * Post-operation handler to handle refunds and logging
     */
    function _postOp(
        PostOpMode mode,
        bytes calldata context,
        uint256 actualGasCost,
        uint256 /* actualUserOpFeePerGas */
    ) internal override nonReentrant {
        PostOpContext memory postOpContext = abi.decode(context, (PostOpContext));
        
        // For native token payments, no refund needed
        if (postOpContext.token == address(0)) {
            emit UserOpSponsored(
                postOpContext.sender,
                address(0),
                actualGasCost,
                0
            );
            return;
        }

        // For ERC20 token payments, calculate refund
        IERC20 token = IERC20(postOpContext.token);
        uint256 actualTokenCost = (actualGasCost * postOpContext.exchangeRate) / 1e18;
        
        // Only refund if operation succeeded and we have excess
        if (mode == PostOpMode.opSucceeded && postOpContext.prefundAmount > actualTokenCost) {
            uint256 refundAmount = postOpContext.prefundAmount - actualTokenCost;
            
            // Transfer refund from beneficiary back to user
            if (beneficiary == address(this)) {
                // If beneficiary is this contract, we can refund directly
                try token.safeTransfer(postOpContext.sender, refundAmount) {
                    // Success
                } catch {
                    // Refund failed, but don't revert the entire operation
                    emit UserOpSponsored(
                        postOpContext.sender,
                        postOpContext.token,
                        actualGasCost,
                        postOpContext.prefundAmount
                    );
                    return;
                }
            }
            // If beneficiary is external, they need to handle their own refunds
        }

        emit UserOpSponsored(
            postOpContext.sender,
            postOpContext.token,
            actualGasCost,
            actualTokenCost
        );
    }

    /**
     * Decode paymaster data from paymasterAndData field
     */
    function _decodePaymasterData(bytes calldata paymasterAndData) 
        internal 
        pure 
        returns (PaymasterData memory data) 
    {
        // paymasterAndData format: paymaster_address (20) + paymaster_data
        // Our data: token(32) + exchangeRate(32) + validUntil(32) + validAfter(32)
        if (paymasterAndData.length < 20 + 128) {
            revert InvalidPaymasterData();
        }

        bytes calldata paymasterData = paymasterAndData[20:];
        
        data.token = address(bytes20(paymasterData[0:20]));
        data.exchangeRate = uint256(bytes32(paymasterData[32:64]));
        data.validUntil = uint256(bytes32(paymasterData[64:96]));
        data.validAfter = uint256(bytes32(paymasterData[96:128]));
    }

    /**
     * Check if the user operation is an ERC20 approval operation
     */
    function _isApprovalOperation(
        PackedUserOperation calldata userOp,
        address tokenAddress
    ) internal pure returns (bool) {
        // Check if callData is calling approve(address,uint256) on the token
        if (userOp.callData.length < 68) return false; // 4 + 32 + 32
        
        bytes4 selector = bytes4(userOp.callData[0:4]);
        
        // Standard ERC20 approve selector: approve(address,uint256)
        if (selector == IERC20.approve.selector) {
            // Extract the target address from callData to see if it's the token
            // This is a simplified check - in practice you might need more sophisticated parsing
            return true;
        }
        
        return false;
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
    function withdrawTokens(
        address token,
        address to,
        uint256 amount
    ) external onlyOwner {
        require(beneficiary == address(this), "Beneficiary is not this contract");
        require(to != address(0), "Invalid recipient");
        
        if (token == address(0)) {
            // Withdraw ETH
            require(amount <= address(this).balance, "Insufficient ETH balance");
            payable(to).transfer(amount);
        } else {
            // Withdraw ERC20
            IERC20(token).safeTransfer(to, amount);
        }
        
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
}