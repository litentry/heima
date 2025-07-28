// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "./BasePaymaster.sol";
import "../interfaces/PackedUserOperation.sol";

/**
 * Demo paymaster implementation that sponsors gas for any user operation
 * without bundler restrictions. For demonstration purposes only.
 * 
 * SECURITY WARNING: This paymaster accepts any operation as long as it has
 * sufficient deposit. Do not use in production.
 */
contract DemoPaymaster is BasePaymaster {
    event UserOpSponsored(address indexed account, uint256 actualGasCost);

    constructor(IEntryPoint _entryPoint) BasePaymaster(_entryPoint) {}

    /**
     * Validate a user operation.
     * Sponsors any operation as long as we have sufficient deposit.
     */
    function _validatePaymasterUserOp(PackedUserOperation calldata userOp, bytes32 /* userOpHash */, uint256 maxCost)
        internal
        view
        override
        returns (bytes memory context, uint256 validationData)
    {
        // Check if we have enough deposit to cover the cost
        uint256 ourDeposit = entryPoint.balanceOf(address(this));
        if (ourDeposit < maxCost) {
            // Reject - insufficient funds
            return ("", 1);
        }

        // Accept the operation - sponsor any account
        // Return smart account address in context for postOp logging
        return (abi.encode(userOp.sender), 0);
    }

    /**
     * Post-operation handler.
     * Log the sponsored operation.
     */
    function _postOp(PostOpMode /* mode */, bytes calldata context, uint256 actualGasCost, uint256 /* actualUserOpFeePerGas */)
        internal
        override
    {
        // Decode sender from context
        address sender = abi.decode(context, (address));

        // Log the sponsored operation
        emit UserOpSponsored(sender, actualGasCost);
    }

    /**
     * Allow contract to receive ETH and automatically deposit to EntryPoint.
     */
    receive() external payable {
        entryPoint.depositTo{value: msg.value}(address(this));
    }
}