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
