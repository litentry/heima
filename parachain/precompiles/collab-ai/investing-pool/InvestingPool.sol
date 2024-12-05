// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IInvestingPool {

	/// @notice Update epoch reward of pool
    /// @param pool_proposal_index: Index of pool proposal
    /// @param epoch: Epoch index
    /// @param reward: Amount of total reward for epoch
    /// @custom:selector 0xb712851b
	/// 				 updateReward(uint256,uint256,uint256)
    function updateReward(uint256 pool_proposal_index, uint256 epoch, uint256 reward) external;

	/// @notice Claim both epoch reward and CAN token reward, will claim until the epoch where reward updated
    /// @param asset_id: Token id of corresponding token
    /// @param amount: Amount of token
    /// @custom:selector 0xc3490263
	/// 				 claim(uint256,uint256)
    function claim(uint256 asset_id, uint256 amount) external;


    /// @dev A structure for pool setting
    struct PoolSetting {
        uint256 start_time;
        uint256 epoch;
        uint256 epoch_range;
        uint256 pool_cap;
        bytes32 admin;
    }

    /// @notice Claim both epoch reward and CAN token reward, will claim until the epoch where reward updated
    /// @param pool_proposal_index: List of pool proposal index
    /// @custom:selector 0xd3e557b6
	/// 				 investingPoolSetting(uint256[])
    function investingPoolSetting(uint256[] calldata pool_proposal_index) external view returns (PoolSetting[] memory pool_setting);

    /// @dev A structure for recording epoch reward
    struct EpochReward {
        uint256 total_reward;
        uint256 claimed_reward;
    }

    /// @notice Query All epoch rewards updated by curator
    /// @param pool_proposal_index: Pool proposal index
    /// @custom:selector 0x25819dc7
	/// 				 stableInvestingPoolEpochReward(uint256)
    function stableInvestingPoolEpochReward(uint256 pool_proposal_index) external view returns (EpochReward[] memory epcoh_reward);
}