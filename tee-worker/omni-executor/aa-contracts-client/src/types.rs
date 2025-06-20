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

	// entry point
	function handleOps(PackedUserOperation[] calldata ops, address payable beneficiary) external;
	function getSenderAddress(bytes calldata initCode) external;
	function getUserOpHash(PackedUserOperation calldata userOp) public view returns (bytes32);
	function depositTo(address account) public payable virtual;
	error SenderAddressResult(address sender);

	// smart account factory
	function createAccount(bytes32 oa, bytes memory clientId, address root) public;

	// smart account
	function getNonce() public view virtual returns (uint256);
	function addRootSigner(address root) public;
	function removeRootSigner(address root) public;
}
