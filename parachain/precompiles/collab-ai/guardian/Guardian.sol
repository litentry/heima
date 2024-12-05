// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IGuardian {
    /// @dev Defines the candidate status type.
    enum CandidateStatus {
        Unverified,
	    Verified,
	    Banned
    }

    /// @dev A structure for Guardian query result
    struct GuardianQueryResult {
        bool exist;
        bytes32 info_hash;
        uint256 update_block;
        bytes32 guardian;
        CandidateStatus status;
    }
    
    /// @dev Defines GuardianVote type
    enum GuardianVote {
        Neutral,
        Aye,
        Nay,
        Specific
    }
	/// @notice Regist info hash of guardian and reserve funds, only work if not already registed
	/// @param info_hash: H256 hash of info image
    /// @custom:selector 0x3cf5464a
	/// 				 registGuardian(bytes32)
    function registGuardian(bytes32 info_hash) external;

	/// @notice Update info hash of guardian, only work if already registed
	/// @param info_hash: H256 hash of info image
    /// @custom:selector 0x2b764649
	/// 				 updateGuardian(bytes32)
    function updateGuardian(bytes32 info_hash) external;

    /// @notice Clean guardian info and return funds if not banned, otherwise no fund return
    /// @custom:selector 0xc654e77d
	/// 				 cleanGuardian()
    function cleanGuardian() external;

    /// @notice Vote guardian and express the corresponding status, evm
    /// @custom:selector 0x55b90ea7
	/// 				 vote(bytes32,uint8,uint256)
    function vote(bytes32 guardian, GuardianVote status, uint256 potential_proposal_index) external;

    /// @notice Remove msg.sender's all existing guardian vote
    /// @custom:selector 0x3219bdc0
	/// 				 removeAllVotes()
    function removeAllVotes() external;

    /// @notice public guardian count of next guardian index will be
    /// @custom:selector 0x69d0b14e
	/// 				 publicGuardianCount()
    function publicGuardianCount() external view returns (uint256 count);

    /// @notice public guardian to index, substrate address format, bool represents if such index exists
    /// @param guardian: substrate format address
    /// @custom:selector 0xf46175b8
	/// 				 publicGuardianToIndex(bytes32)
    function publicGuardianToIndex(bytes32 guardian) external view returns (bool exist, uint256 index);

    /// @notice Guardian index to guardian info, bool represents if such info exists
    /// @param index: Guardian index
    /// @custom:selector 0x59c95743
	/// 				 guardianIndexToInfo(address)
    function guardianIndexToInfo(uint256 index) external view returns (GuardianQueryResult memory result);

    /// @notice Guardian index to guardian info, bool represents if such info exists
    /// @param start_id: Guardian index start_id, included
    /// @param end_id: Guardian index end id, excluded
    /// @custom:selector 0x92bd7975
	/// 				 batchGuardianIndexToInfo(uint256,uint256)
    function batchGuardianIndexToInfo(uint256 start_id, uint256 end_id) external view returns (GuardianQueryResult[] memory result);
    
    /// @notice Query voter's vote of one specific guardian given its guardian index, substrate
    /// @param voter: voter address, substrate
    /// @param guardian_index: Guardian index
    /// @custom:selector 0xfaad0ba2
	/// 				 guardianVotes(bytes32,uint256)
    function guardianVotes(bytes32 voter, uint256 guardian_index) external view returns (GuardianVote guardian_vote, uint256 potential_proposal_index);
}