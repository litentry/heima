// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use alloy::sol;

sol! {
	// common
	struct PackedUserOperation {
		address sender;
		uint256 nonce;
		bytes initCode;
		bytes callData;
		bytes32 accountGasLimits;
		uint256 preVerificationGas;
		bytes32 gasFees;
		bytes paymasterAndData;
		address sessionAccount;
		uint256 sessionExpiration;
		bytes sessionAccountProof;
		bytes signature;
	}

	// EIP-712 domain separator struct for hash calculation
	struct EIP712Domain {
		bytes32 name;
		bytes32 version;
		uint256 chainId;
		address verifyingContract;
	}

	// PackedUserOperation struct for EIP-712 hashing (with typehash)
	struct PackedUserOperationForHashing {
		bytes32 typeHash;
		address sender;
		uint256 nonce;
		bytes32 initCode;
		bytes32 callData;
		bytes32 accountGasLimits;
		uint256 preVerificationGas;
		bytes32 gasFees;
		bytes32 paymasterAndData;
		address sessionAccount;
		uint256 sessionExpiration;
		bytes32 sessionAccountProof;
	}

	// entry point
	function handleOps(PackedUserOperation[] calldata ops, address payable beneficiary) external;
	function getSenderAddress(bytes calldata initCode) external;
	function getUserOpHash(PackedUserOperation calldata userOp) public view returns (bytes32);
	function depositTo(address account) public payable virtual;
	error SenderAddressResult(address sender);

	// smart account factory
	function createAccount(bytes32 oa, bytes memory clientId, address root) public;
	function getAddress(bytes32 oa, bytes memory clientId, address root) public view returns (address);

	// smart account
	function getNonce() public view virtual returns (uint256);
	function addRootSigner(address root) public;
	function removeRootSigner(address root) public;
	function initialize(bytes32 oa, bytes memory clientId, address root) public;
	function getOwner() public view returns (bytes32);

	// paymaster
	function deposit() public payable;
	function setAuthorizedBundler(address bundler, bool authorized) external;

	// simulation types
	struct StakeInfo {
		uint256 stake;
		uint256 unstakeDelaySec;
	}

	struct ReturnInfo {
		uint256 preOpGas;
		uint256 prefund;
		uint256 accountValidationData;
		uint256 paymasterValidationData;
		bytes context;
	}

	struct AggregatorStakeInfo {
		address aggregator;
		StakeInfo stakeInfo;
	}

	struct ValidationResult {
		ReturnInfo returnInfo;
		StakeInfo senderInfo;
		StakeInfo factoryInfo;
		StakeInfo paymasterInfo;
		AggregatorStakeInfo aggregatorInfo;
	}

	struct ExecutionResult {
		uint256 preOpGas;
		uint256 paid;
		uint256 accountValidationData;
		uint256 paymasterValidationData;
		bool targetSuccess;
		bytes targetResult;
	}

	// simulation functions
	function simulateValidation(PackedUserOperation calldata userOp) external returns (ValidationResult memory);
	function simulateHandleOp(PackedUserOperation calldata op, address target, bytes calldata targetCallData) external returns (ExecutionResult memory);
}
