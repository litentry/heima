// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IPoolProposal {
    /// @dev A structure for bonding of curator's pool desposit
    struct DepositBond {
        uint256 owner;
        uint256 amount;
    }

    /// @dev A structure for bonding of user's official pre-staking
    struct StakingBond {
        uint256 poolIndex;
        bytes32 owner;
        uint256 amount;
    }

    /// @dev A structure for bonding of user's queued pre-staking
    struct QueuedStakingBond {
        uint256 poolIndex;
        bytes32 owner;
        uint256 amount;
        uint256 queuedTime;
    }

    /// @dev A structure for proposal and its expiry time
    struct PoolProposalStatus {
        uint256 index;
        uint256 expiryTime;
    }

    /// @dev A strcuture for proposal details
    struct PoolProposalInfo {
        bool exist;
	    // Proposer/Curator
	    bytes32 proposer;
	    // Hash of pool info like legal files etc.
	    bytes32 infoHash;
	    // The maximum investing amount that the pool can handle
	    uint256 maxPoolSize;
	    // If proposal passed, when the investing pool will start, Block number
	    uint256 poolStartTime;
	    // If proposal passed, when the investing pool will end, Block number
	    uint256 poolEndTime;
	    // estimated APR, but in percentage form
	    // i.e. 100 => 100%
	    uint256 estimatedEpochReward;
	    // Proposal status flags
    	// 	/// Whether the pool proposal passing the committee/democracy voting.
		// /// A valid pool must passing committee/public's audit procedure regarding legal files and other pool parameters.
		// const PUBLIC_VOTE_PASSED = 0b0000_0001;
        // ///
		// /// Whether the minimum Investing amount proposed by curator is satisfied.
		// /// Currently, a full size must be satisfied.
		// /// Once a pool is satisfied this requirement, all Investing amount can no longer be withdrawed
		// /// unless the pool is later denied passing by voting or until the end of pool maturity.
		// /// Otherwise, the pool will be refunded.
		// const STAKE_AMOUNT_PASSED = 0b0000_0010;
        // ///
		// /// Whether the pool guardian has been selected
		// /// A valid pool must have guardian or a default one will be used (committee)
		// const GUARDIAN_SELECTED = 0b0000_0100;
        // ///
		// /// Whether the proposal expired yet
		// /// Has nothing to do with pool. Only related to proposal expired time
		// const PROPOSAL_EXPIRED = 0b0000_1000;
	    uint8 proposalStatusFlags;
    }

	/// @notice Propose an investing pool proposal
    /// @param max_pool_size: At most this amount of raised money curator/investing pool willing to take
    /// @param proposal_last_time: How does the proposal lasts for voting/preinvesting.
    /// @param pool_last_time: How long does the investing pool last if passed
    /// @param estimated_pool_reward: This number is only for displaying purpose without any techinical meaning
	/// @param pool_info_hash: H256 hash of pool info for including pool details
    /// @custom:selector 0x7bc55add
	/// 				 proposeInvestingPool(uint256,uint256,uint256,uint256,bytes32)
    function proposeInvestingPool(uint256 max_pool_size, uint256 proposal_last_time, uint256 pool_last_time, uint256 estimated_pool_reward, bytes32 pool_info_hash) external;

	/// @notice Prestake the pool proposal
	/// @param pool_proposal_index: Index of pool proposal
    /// @param amount: Amount of per-staking user provides
    /// @custom:selector 0x68e3a76c
	/// 				 preStakeProposal(uint256,uint256)
    function preStakeProposal(uint256 pool_proposal_index, uint256 amount) external;

    /// @notice Withdrawal the prestaking the pool proposal
	/// @param pool_proposal_index: Index of pool proposal
    /// @param amount: Amount of per-staking user provides
    /// @custom:selector 0x389cd4af
	/// 				 withdrawPreInvesting(uint256,uint256)
    function withdrawPreInvesting(uint256 pool_proposal_index, uint256 amount) external;

    /// @notice A guardian declaring his incentive of participating pool
    /// @param pool_proposal_index: Index of pool proposal
    /// @custom:selector 0x619c08e2
	/// 				 guardianParticipateProposal(uint256)
    function guardianParticipateProposal(uint256 pool_proposal_index) external;

    /// @notice The next free Pool Proposal index, aka the number of pool proposed so far.
    /// @custom:selector 0x1b9e695b
	/// 				 poolProposalCount()
    function poolProposalCount() external view returns (uint256 next_proposal_index);

    /// @notice Query a curator's pool proposal deposit, bond owner = proposal index
    /// @param curator: curator address, substrate
    /// @custom:selector 0x87178c26
	/// 				 poolProposalDepositOf(bytes32)
    function poolProposalDepositOf(bytes32 curator) external view returns (DepositBond[] memory deposit_record);

    /// @notice Query all pending pool proposal and their schedule
    /// @custom:selector 0x2f4f6b2a
	/// 				 pendingPoolProposalStatus()
    function pendingPoolProposalStatus() external view returns (PoolProposalStatus[] memory proposal_status);

    /// @notice Query a single pool proposal and its detail, bool represents if such info exists
    /// @param pool_proposal_index: Index of pool proposal
    /// @custom:selector 0x18afd9ad
	/// 				 poolProposal(uint256)
    function poolProposal(uint256 pool_proposal_index) external view returns (PoolProposalInfo memory proposal_info);

    /// @notice Query a batch of pool proposal and their details, bool represents if such info exists
    /// @param start_id: Proposal index start_id, included
    /// @param end_id: Proposal index end id, excluded
    /// @custom:selector 0x1b3fb27c
	/// 				 batchPoolProposal(uint256,uint256)
    function batchPoolProposal(uint256 start_id, uint256 end_id) external view returns (PoolProposalInfo[] memory proposal_info);

    /// @notice Batch query pool proposals and their existing included pre stakings
    /// @param start_id: Proposal index start_id, included
    /// @param end_id: Proposal index end id, excluded
    /// @custom:selector 0x49dc251a
	/// 				 poolPreInvestings(uint256,uint256)
    function poolPreInvestings(uint256 start_id, uint256 end_id) external view returns (StakingBond[] memory pre_investing_bond);

    /// @notice Batch query pool proposals and their queued pre stakings
    /// @param start_id: Proposal index start_id, included
    /// @param end_id: Proposal index end id, excluded
    /// @custom:selector 0x201be573
	/// 				 poolPreInvestingsQueued(uint256,uint256)
    function poolPreInvestingsQueued(uint256 start_id, uint256 end_id) external view returns (QueuedStakingBond[] memory queued_bond);

    /// @notice Query a single pool proposal and its potential guardian detail
    /// @param pool_proposal_index: Index of pool proposal
    /// @custom:selector 0x6630e6ee
	/// 				 poolGuardian(uint256)
    function poolGuardian(uint256 pool_proposal_index) external view returns (bytes32[] memory guardian);

    /// @notice Query a user's all invested positions
    /// @param user_address: user address, substrate
    /// @custom:selector 0x814f8437
	/// 				 userPoolPreInvestings(bytes32)
    function userPoolPreInvestings(bytes32 user_address) external view returns (StakingBond[] memory pre_investing_bond);

    /// @notice Query a user's all queued positions
    /// @param user_address: user address, substrate
    /// @custom:selector 0x64833a66
	/// 				 userPoolPreInvestingsQueued(bytes32)
    function userPoolPreInvestingsQueued(bytes32 user_address) external view returns (QueuedStakingBond[] memory queued_bond);

    /// @notice Query corresponding pool proposal index's total pre-investing amount 
    /// @param start_id: Proposal index start_id, included
    /// @param end_id: Proposal index end id, excluded
    /// @custom:selector 0x3a4fbb0f
	/// 				 poolPreInvestingsTotal(uint256,uint256)
    function poolPreInvestingsTotal(uint256 start_id, uint256 end_id) external view returns (uint256[] memory pre_investing_total);

    /// @notice Query corresponding pool proposal index's total pre-investing queued amount 
    /// @param start_id: Proposal index start_id, included
    /// @param end_id: Proposal index end id, excluded
    /// @custom:selector 0xa54205c4
	/// 				 poolPreInvestingsQueuedTotal(uint256,uint256)
    function poolPreInvestingsQueuedTotal(uint256 start_id, uint256 end_id) external view returns (uint256[] memory queued_total);
}