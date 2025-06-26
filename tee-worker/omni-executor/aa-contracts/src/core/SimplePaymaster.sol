// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "./BasePaymaster.sol";
import "../interfaces/PackedUserOperation.sol";

/**
 * Simple paymaster implementation that sponsors gas for any user operation
 * submitted by authorized off-chain bundlers.
 */
contract SimplePaymaster is BasePaymaster {
    // Mapping of authorized bundler addresses
    mapping(address => bool) public authorizedBundlers;

    event UserOpSponsored(address indexed account, uint256 actualGasCost);
    event AuthorizedBundlerUpdated(address indexed bundler, bool authorized);

    constructor(IEntryPoint _entryPoint, address _initialBundler) BasePaymaster(_entryPoint) {
        authorizedBundlers[_initialBundler] = true;
        emit AuthorizedBundlerUpdated(_initialBundler, true);
    }

    /**
     * Validate a user operation.
     * Only sponsor operations submitted by authorized bundlers.
     */
    function _validatePaymasterUserOp(PackedUserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost)
        internal
        override
        returns (bytes memory context, uint256 validationData)
    {
        // Check if the transaction is being submitted by an authorized bundler
        if (!authorizedBundlers[tx.origin]) {
            // Reject - not from an authorized bundler
            return ("", 1);
        }

        // Check if we have enough deposit to cover the cost
        uint256 ourDeposit = entryPoint.balanceOf(address(this));
        if (ourDeposit < maxCost) {
            // Reject - insufficient funds
            return ("", 1);
        }

        // Accept the operation - sponsor any account as long as bundler is authorized
        // Return smart account address in context for postOp logging
        return (abi.encode(userOp.sender), 0);
    }

    /**
     * Post-operation handler.
     * Log the sponsored operation.
     */
    function _postOp(PostOpMode mode, bytes calldata context, uint256 actualGasCost, uint256 actualUserOpFeePerGas)
        internal
        override
    {
        // Decode sender from context
        address sender = abi.decode(context, (address));

        // Log the sponsored operation
        emit UserOpSponsored(sender, actualGasCost);
    }

    /**
     * Add or remove an authorized bundler.
     */
    function setAuthorizedBundler(address bundler, bool authorized) external onlyOwner {
        authorizedBundlers[bundler] = authorized;
        emit AuthorizedBundlerUpdated(bundler, authorized);
    }

    /**
     * Allow contract to receive ETH and automatically deposit to EntryPoint.
     */
    receive() external payable {
        entryPoint.depositTo{value: msg.value}(address(this));
    }
}
