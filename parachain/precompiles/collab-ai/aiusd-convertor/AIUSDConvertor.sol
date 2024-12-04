// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IAIUSDConvertor {
	/// @notice Lock target asset_id and mint AIUSD
	/// @param asset_id: target asset id in exchange for AIUSD
    /// @param aiusd_amount: The amount of AIUSD token
    /// @custom:selector 0x1a15980f
	/// 				 mintAIUSD(uint256,uint256)
    function mintAIUSD(uint256 asset_id, uint256 aiusd_amount) external;

    /// @notice Burn aiusd and get target asset_id token released
	/// @param asset_id: target asset id in exchange for AIUSD
    /// @param aiusd_amount: The amount of AIUSD token
    /// @custom:selector 0xa89bb55f
	/// 				 burnAIUSD(uint256,uint256)
    function burnAIUSD(uint256 asset_id, uint256 aiusd_amount) external;
}
