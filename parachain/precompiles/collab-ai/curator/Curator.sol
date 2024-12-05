// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ICurator {
    /// @dev Defines the candidate status type.
    enum CandidateStatus {
        Unverified,
	    Verified,
	    Banned
    }

    /// @dev A structure for curator query result
    struct CuratorQueryResult {
        bool exist;
        bytes32 info_hash;
        uint256 update_block;
        bytes32 curator;
        CandidateStatus status;
    }

	/// @notice Regist info hash of curator and reserve funds, only work if not already registed
	/// @param info_hash: H256 hash of info image
    /// @custom:selector 0x8ead391c
	/// 				 registCurator(bytes32)
    function registCurator(bytes32 info_hash) external;

	/// @notice Update info hash of curator, only work if already registed
	/// @param info_hash: H256 hash of info image
    /// @custom:selector 0x457c00e6
	/// 				 updateCurator(bytes32)
    function updateCurator(bytes32 info_hash) external;

    /// @notice clean curator info and return funds if not banned, otherwise no fund return
    /// @custom:selector 0xe3b134e6
	/// 				 cleanCurator()
    function cleanCurator() external;

    /// @notice public curator count of next curator index will be
    /// @custom:selector 0x566537c5
	/// 				 publicCuratorCount()
    function publicCuratorCount() external view returns (uint256 count);

    /// @notice public curator to index, substrate address format, bool represents if such index exists
    /// @param curator: substrate format address
    /// @custom:selector 0x039997d0
	/// 				 publicCuratorToIndex(bytes32)
    function publicCuratorToIndex(bytes32 curator) external view returns (bool exist, uint256 index);

    /// @notice Curator index to curator info, bool represents if such info exists
    /// @param index: Curator index
    /// @custom:selector 0x74cded61
	/// 				 curatorIndexToInfo(uint256)
    function curatorIndexToInfo(uint256 index) external view returns (CuratorQueryResult memory result);

    /// @notice Curator index to curator info, bool represents if such info exists
    /// @param start_id: Guardian index start_id, included
    /// @param end_id: Guardian index end id, excluded
    /// @custom:selector 0xf2e508c7
	/// 				 batchCuratorIndexToInfo(uint256,uint256)
    function batchCuratorIndexToInfo(uint256 start_id, uint256 end_id) external view returns (CuratorQueryResult[] memory result);

}