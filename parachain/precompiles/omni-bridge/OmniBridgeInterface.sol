// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IOmniBridge {
	/// @notice Used to transfer assets through token bridge.
	/// @param amount: The amount of tokens to be transferred.
    /// @param dest_id: The destination chain id indicator
    /// @param native: Indicator of if asset is native. If true, resource_id will be ignored
    /// @param resource_id: Resource indicator of type of assets transferred (In substrate runtime it is u128)
    /// @param recipient: Recipient address, typically H160/H256
    /// @custom:selector 0xef185624
	/// 				 payIn(uint256,uint8,bool,uint256,bytes)
    function payIn(uint256 amount, uint8 dest_id, bool native, uint256 resource_id, bytes calldata recipient) external;
}