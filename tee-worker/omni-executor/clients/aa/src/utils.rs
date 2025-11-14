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

use crate::types::{OwnerType, PackedUserOperationForHashing};
use crate::PackedUserOperation;
use alloy::primitives::{keccak256, Address, Bytes, FixedBytes, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::{SolCall, SolValue};

/// Build a transaction request for a contract call (no value transfer)
pub fn build_call_transaction(to: Address, call_data: Vec<u8>) -> TransactionRequest {
	TransactionRequest {
		to: Some(TxKind::Call(to)),
		input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
		..Default::default()
	}
}

/// Build a transaction request for a payable contract call (with value transfer)
pub fn build_payable_transaction(
	to: Address,
	call_data: Vec<u8>,
	value: U256,
) -> TransactionRequest {
	TransactionRequest {
		to: Some(TxKind::Call(to)),
		input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
		value: Some(value),
		..Default::default()
	}
}

/// Calculate the CREATE2 address for an OmniAccount without calling the EntryPoint
/// This mirrors the getAddress function in OmniAccountFactory.sol
pub fn calculate_omni_account_address(
	factory_address: Address,
	account_implementation: Address,
	oa: FixedBytes<32>,
	oa_type: OwnerType,
	client_id: &[u8],
	root: Address,
) -> Address {
	use crate::types::initializeCall;

	let initialize_call =
		initializeCall { oa, oaType: oa_type, clientId: Bytes::from(client_id.to_vec()), root };
	let initialize_data = initialize_call.abi_encode();

	let mut constructor_params = Vec::new();

	// Encode address (32 bytes, left-padded)
	constructor_params.extend_from_slice(&[0u8; 12]); // padding
	constructor_params.extend_from_slice(account_implementation.as_slice());

	// Encode offset to bytes data (always 0x40 = 64 for address + offset)
	constructor_params.extend_from_slice(&[0u8; 31]);
	constructor_params.push(0x40);

	// Encode bytes length
	let initialize_data_len = initialize_data.len();
	let len_bytes = (initialize_data_len as u32).to_be_bytes();
	constructor_params.extend_from_slice(&[0u8; 28]);
	constructor_params.extend_from_slice(&len_bytes);

	// Encode bytes data with padding to 32-byte boundary
	constructor_params.extend_from_slice(&initialize_data);
	let padding_needed = (32 - (initialize_data_len % 32)) % 32;
	constructor_params.extend_from_slice(&vec![0u8; padding_needed]);

	// Get ERC1967Proxy creation code
	let proxy_creation_code = get_erc1967_proxy_creation_code();

	// Use abi.encodePacked equivalent: direct concatenation without length prefix
	let init_code = [proxy_creation_code.as_slice(), constructor_params.as_slice()].concat();
	let init_code_hash = keccak256(&init_code);

	// Calculate CREATE2 address: keccak256(0xff + factory_address + salt + init_code_hash)[12:]
	let mut create2_input = Vec::new();
	create2_input.push(0xff);
	create2_input.extend_from_slice(factory_address.as_slice());
	create2_input.extend_from_slice(oa.as_slice());
	create2_input.extend_from_slice(init_code_hash.as_slice());

	let hash = keccak256(&create2_input);
	Address::from_slice(&hash[12..])
}

/// Get the creation code for ERC1967Proxy
///
/// This returns the exact bytecode from Solidity's `type(ERC1967Proxy).creationCode`.
///
/// ## How to extract this bytecode:
/// ```bash
/// forge inspect @openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy bytecode
/// ```
///
/// Note: The bytecode may change between OpenZeppelin versions, so verify the hash
/// in the test `test_proxy_creation_code_hash()` when updating dependencies.
fn get_erc1967_proxy_creation_code() -> Vec<u8> {
	use alloy::hex;
	// ERC1967Proxy creation bytecode (680 bytes)
	hex::decode("60806040526102a88038038061001481610168565b92833981016040828203126101645781516001600160a01b03811692909190838303610164576020810151906001600160401b03821161016457019281601f8501121561016457835161006e610069826101a1565b610168565b9481865260208601936020838301011161016457815f926020809301865e86010152823b15610152577f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc80546001600160a01b031916821790557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a282511561013a575f8091610122945190845af43d15610132573d91610113610069846101a1565b9283523d5f602085013e6101bc565b505b604051608d908161021b8239f35b6060916101bc565b50505034156101245763b398979f60e01b5f5260045ffd5b634c9c8ce360e01b5f5260045260245ffd5b5f80fd5b6040519190601f01601f191682016001600160401b0381118382101761018d57604052565b634e487b7160e01b5f52604160045260245ffd5b6001600160401b03811161018d57601f01601f191660200190565b906101e057508051156101d157602081519101fd5b63d6bda27560e01b5f5260045ffd5b81511580610211575b6101f1575090565b639996b31560e01b5f9081526001600160a01b0391909116600452602490fd5b50803b156101e956fe60806040525f8073ffffffffffffffffffffffffffffffffffffffff7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416368280378136915af43d5f803e156053573d5ff35b3d5ffdfea2646970667358221220251552741a02ceec145dfb729e6c1abd4dc30b27a494f4dbc9cc046648f3ef4164736f6c634300081c0033")
		.expect("Valid ERC1967Proxy bytecode")
}

/// Calculate user operation hash locally without calling EntryPoint.getUserOpHash
/// This implements the same logic as EntryPoint.getUserOpHash() and UserOperationLib.hash()
///
/// Based on:
/// - EntryPoint.sol getUserOpHash(): MessageHashUtils.toTypedDataHash(getDomainSeparatorV4(), userOp.hash(overrideInitCodeHash))
/// - UserOperationLib.sol hash(): keccak256(encode(userOp, overrideInitCodeHash))
/// - UserOperationLib.sol encode(): abi.encode with PACKED_USEROP_TYPEHASH and all fields
///
/// @param user_op - The PackedUserOperation to hash
/// @param entry_point_address - The EntryPoint contract address for domain separator
/// @param chain_id - The chain ID for domain separator
/// @return The user operation hash as computed by EntryPoint.getUserOpHash()
pub fn calculate_user_operation_hash(
	user_op: &PackedUserOperation,
	entry_point_address: Address,
	chain_id: u64,
) -> FixedBytes<32> {
	// EIP-712 Domain Constants (from EntryPoint.sol)
	const DOMAIN_NAME: &str = "ERC4337";
	const DOMAIN_VERSION: &str = "1";

	// Calculate domain separator (EIP-712)
	let domain_separator =
		calculate_domain_separator(DOMAIN_NAME, DOMAIN_VERSION, chain_id, entry_point_address);

	// Encode user operation (equivalent to UserOperationLib.encode)
	let encoded_user_op = encode_user_operation(user_op);

	// Hash the encoded user operation (equivalent to UserOperationLib.hash)
	let user_op_struct_hash = keccak256(&encoded_user_op);

	// Final EIP-712 hash (equivalent to MessageHashUtils.toTypedDataHash)
	calculate_eip712_hash(domain_separator, user_op_struct_hash)
}

/// Calculate EIP-712 domain separator
/// Based on EIP-712 standard: keccak256(abi.encode(EIP712DOMAIN_TYPEHASH, name, version, chainId, verifyingContract))
fn calculate_domain_separator(
	name: &str,
	version: &str,
	chain_id: u64,
	verifying_contract: Address,
) -> FixedBytes<32> {
	// EIP712Domain TypeHash - calculated at runtime
	let eip712_domain_typehash = keccak256(
		"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
			.as_bytes(),
	);

	// We need to manually encode since the domain struct should include the typehash
	// Manual encoding to match Solidity's exact behavior
	let mut encoded = Vec::new();

	// EIP712Domain typehash (32 bytes)
	encoded.extend_from_slice(eip712_domain_typehash.as_slice());

	// name hash (32 bytes)
	let name_hash = keccak256(name.as_bytes());
	encoded.extend_from_slice(name_hash.as_slice());

	// version hash (32 bytes)
	let version_hash = keccak256(version.as_bytes());
	encoded.extend_from_slice(version_hash.as_slice());

	// chain ID (32 bytes, big-endian)
	let chain_id_bytes = U256::from(chain_id).to_be_bytes::<32>();
	encoded.extend_from_slice(&chain_id_bytes);

	// verifying contract (32 bytes, left-padded)
	let mut contract_bytes = [0u8; 32];
	contract_bytes[12..].copy_from_slice(verifying_contract.as_slice());
	encoded.extend_from_slice(&contract_bytes);

	keccak256(&encoded)
}

/// Encode PackedUserOperation for hashing (equivalent to UserOperationLib.encode)
fn encode_user_operation(user_op: &PackedUserOperation) -> Vec<u8> {
	// PackedUserOperation TypeHash - calculated at runtime
	let packed_userop_typehash = keccak256("PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData)".as_bytes());

	// Hash dynamic fields (bytes data)
	let init_code_hash = keccak256(&user_op.initCode);
	let call_data_hash = keccak256(&user_op.callData);
	let paymaster_and_data_hash = keccak256(&user_op.paymasterAndData);

	let user_op_for_hashing = PackedUserOperationForHashing {
		typeHash: packed_userop_typehash,
		sender: user_op.sender,
		nonce: user_op.nonce,
		initCode: init_code_hash,
		callData: call_data_hash,
		accountGasLimits: user_op.accountGasLimits,
		preVerificationGas: user_op.preVerificationGas,
		gasFees: user_op.gasFees,
		paymasterAndData: paymaster_and_data_hash,
	};

	// Use sol! type's ABI encoding
	user_op_for_hashing.abi_encode()
}

/// Calculate final EIP-712 hash (equivalent to MessageHashUtils.toTypedDataHash)
/// EIP-712 final hash: keccak256(abi.encodePacked("\x19\x01", domain_separator, struct_hash))
fn calculate_eip712_hash(
	domain_separator: FixedBytes<32>,
	struct_hash: FixedBytes<32>,
) -> FixedBytes<32> {
	let mut data = Vec::new();
	data.extend_from_slice(b"\x19\x01"); // EIP-712 prefix
	data.extend_from_slice(domain_separator.as_slice());
	data.extend_from_slice(struct_hash.as_slice());

	keccak256(&data)
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::primitives::address;
	use test_log::test;

	#[test]
	fn test_build_call_transaction() {
		let address = address!("0x1234567890123456789012345678901234567890");
		let call_data = vec![0x12, 0x34, 0x56];

		let tx = build_call_transaction(address, call_data.clone());

		assert_eq!(tx.to, Some(TxKind::Call(address)));
		assert_eq!(tx.input.data, Some(call_data.into()));
		assert_eq!(tx.value, None);
	}

	#[test]
	fn test_build_payable_transaction() {
		let address = address!("0x1234567890123456789012345678901234567890");
		let call_data = vec![0x12, 0x34, 0x56];
		let value = U256::from(1000);

		let tx = build_payable_transaction(address, call_data.clone(), value);

		assert_eq!(tx.to, Some(TxKind::Call(address)));
		assert_eq!(tx.input.data, Some(call_data.into()));
		assert_eq!(tx.value, Some(value));
	}

	#[test]
	fn test_calculate_omni_account_address() {
		use alloy::hex;

		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let account_implementation = address!("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0");
		let oa = FixedBytes::from_slice(
			&hex::decode("6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd96211391")
				.unwrap(),
		);
		let client_id = b"test_client";
		let root = address!("0x0000000000000000000000000000000000000001");

		let calculated_address = calculate_omni_account_address(
			factory_address,
			account_implementation,
			oa,
			OwnerType::Evm,
			client_id,
			root,
		);

		assert_ne!(calculated_address, Address::ZERO);

		let calculated_address_2 = calculate_omni_account_address(
			factory_address,
			account_implementation,
			oa,
			OwnerType::Evm,
			client_id,
			root,
		);
		assert_eq!(calculated_address, calculated_address_2);
	}

	#[test]
	fn test_calculate_omni_account_address_different_parameters() {
		use alloy::hex;

		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let account_implementation = address!("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0");
		let oa = FixedBytes::from_slice(
			&hex::decode("6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd96211391")
				.unwrap(),
		);
		let client_id_1 = b"test_client";
		let client_id_2 = b"different_client";
		let root = address!("0x0000000000000000000000000000000000000001");

		let address_1 = calculate_omni_account_address(
			factory_address,
			account_implementation,
			oa,
			OwnerType::Evm,
			client_id_1,
			root,
		);

		let address_2 = calculate_omni_account_address(
			factory_address,
			account_implementation,
			oa,
			OwnerType::Evm,
			client_id_2,
			root,
		);

		assert_ne!(address_1, address_2);
	}

	#[test]
	fn test_proxy_creation_code_hash() {
		use alloy::hex;

		let creation_code = get_erc1967_proxy_creation_code();
		let creation_code_hash = keccak256(&creation_code);

		// Verify creation code hash matches Solidity's type(ERC1967Proxy).creationCode
		let expected_hash =
			hex::decode("c330d96df910d6fe0d4ddadc3702759a198d337c8784f77183a243e296b5b984")
				.unwrap();

		assert_eq!(
			creation_code_hash.as_slice(),
			expected_hash.as_slice(),
			"Creation code hash should match Solidity implementation"
		);
	}

	#[test]
	fn test_calculate_user_operation_hash_basic() {
		use alloy::hex;

		let user_op = PackedUserOperation {
			sender: address!("0x1234567890123456789012345678901234567890"),
			nonce: U256::from(42),
			initCode: Bytes::from(hex::decode("deadbeef").unwrap()),
			callData: Bytes::from(hex::decode("cafebabe").unwrap()),
			accountGasLimits: FixedBytes::from([1u8; 32]),
			preVerificationGas: U256::from(21000),
			gasFees: FixedBytes::from([2u8; 32]),
			paymasterAndData: Bytes::from(hex::decode("abcdef").unwrap()),
			signature: Bytes::from(hex::decode("445566").unwrap()),
		};

		let entry_point_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let chain_id = 31337;

		let hash = calculate_user_operation_hash(&user_op, entry_point_address, chain_id);

		// Ensure we get a non-zero hash
		assert_ne!(hash, FixedBytes::ZERO);

		// Ensure deterministic - same input should produce same hash
		let hash2 = calculate_user_operation_hash(&user_op, entry_point_address, chain_id);
		assert_eq!(hash, hash2);
	}

	#[test]
	fn test_calculate_user_operation_hash_different_inputs() {
		use alloy::hex;

		let user_op1 = PackedUserOperation {
			sender: address!("0x1234567890123456789012345678901234567890"),
			nonce: U256::from(42),
			initCode: Bytes::from(hex::decode("deadbeef").unwrap()),
			callData: Bytes::from(hex::decode("cafebabe").unwrap()),
			accountGasLimits: FixedBytes::from([1u8; 32]),
			preVerificationGas: U256::from(21000),
			gasFees: FixedBytes::from([2u8; 32]),
			paymasterAndData: Bytes::from(hex::decode("abcdef").unwrap()),
			signature: Bytes::from(hex::decode("445566").unwrap()),
		};

		let mut user_op2 = user_op1.clone();
		user_op2.nonce = U256::from(43); // Different nonce

		let entry_point_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let chain_id = 31337;

		let hash1 = calculate_user_operation_hash(&user_op1, entry_point_address, chain_id);
		let hash2 = calculate_user_operation_hash(&user_op2, entry_point_address, chain_id);

		// Different inputs should produce different hashes
		assert_ne!(hash1, hash2);
	}

	#[test]
	fn test_calculate_user_operation_hash_different_chain_ids() {
		use alloy::hex;

		let user_op = PackedUserOperation {
			sender: address!("0x1234567890123456789012345678901234567890"),
			nonce: U256::from(42),
			initCode: Bytes::from(hex::decode("deadbeef").unwrap()),
			callData: Bytes::from(hex::decode("cafebabe").unwrap()),
			accountGasLimits: FixedBytes::from([1u8; 32]),
			preVerificationGas: U256::from(21000),
			gasFees: FixedBytes::from([2u8; 32]),
			paymasterAndData: Bytes::from(hex::decode("abcdef").unwrap()),
			signature: Bytes::from(hex::decode("445566").unwrap()),
		};

		let entry_point_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");

		let hash_mainnet = calculate_user_operation_hash(&user_op, entry_point_address, 1);
		let hash_sepolia = calculate_user_operation_hash(&user_op, entry_point_address, 11155111);

		// Different chain IDs should produce different hashes
		assert_ne!(hash_mainnet, hash_sepolia);
	}

	#[test]
	fn test_domain_separator_components() {
		let name = "ERC4337";
		let version = "1";
		let chain_id = 31337;
		let verifying_contract = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");

		let domain_separator =
			calculate_domain_separator(name, version, chain_id, verifying_contract);

		// Should be deterministic
		let domain_separator2 =
			calculate_domain_separator(name, version, chain_id, verifying_contract);
		assert_eq!(domain_separator, domain_separator2);

		// Different parameters should produce different separators
		let domain_separator_different =
			calculate_domain_separator("DifferentName", version, chain_id, verifying_contract);
		assert_ne!(domain_separator, domain_separator_different);
	}

	#[test]
	fn test_encode_user_operation_consistency() {
		use alloy::hex;

		let user_op = PackedUserOperation {
			sender: address!("0x1234567890123456789012345678901234567890"),
			nonce: U256::from(42),
			initCode: Bytes::from(hex::decode("deadbeef").unwrap()),
			callData: Bytes::from(hex::decode("cafebabe").unwrap()),
			accountGasLimits: FixedBytes::from([1u8; 32]),
			preVerificationGas: U256::from(21000),
			gasFees: FixedBytes::from([2u8; 32]),
			paymasterAndData: Bytes::from(hex::decode("abcdef").unwrap()),
			signature: Bytes::from(hex::decode("445566").unwrap()),
		};

		let encoded1 = encode_user_operation(&user_op);
		let encoded2 = encode_user_operation(&user_op);

		// Encoding should be deterministic
		assert_eq!(encoded1, encoded2);

		// Should contain the typehash at the beginning
		assert!(!encoded1.is_empty());
	}

	#[test]
	fn test_eip712_hash_components() {
		let domain_separator = FixedBytes::from([1u8; 32]);
		let struct_hash = FixedBytes::from([2u8; 32]);

		let eip712_hash = calculate_eip712_hash(domain_separator, struct_hash);

		// Should be deterministic
		let eip712_hash2 = calculate_eip712_hash(domain_separator, struct_hash);
		assert_eq!(eip712_hash, eip712_hash2);

		// Different inputs should produce different hashes
		let different_domain = FixedBytes::from([3u8; 32]);
		let eip712_hash_different = calculate_eip712_hash(different_domain, struct_hash);
		assert_ne!(eip712_hash, eip712_hash_different);
	}

	/// Integration test to verify local hash calculation matches EntryPoint.getUserOpHash
	///
	/// This test requires a running local blockchain with deployed contracts.
	/// To run:
	/// 1. Deploy contracts: `cd aa-contracts && ./local-deploy.sh`
	/// 2. Run: `cargo test test_local_vs_entrypoint_user_op_hash -- --ignored`
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn test_local_vs_entrypoint_user_op_hash() {
		use crate::EntryPointClient;
		use alloy::hex;
		use oe_client_ethereum::AlloyRpcProvider;
		use std::sync::Arc;

		// Test configuration - update these addresses based on your local deployment
		let entry_point_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let chain_id = 31337; // Local hardhat chain ID

		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entry_point_client = EntryPointClient::new(entry_point_address, rpc_client);

		// Create a test user operation
		let user_op = PackedUserOperation {
			sender: address!("0x1234567890123456789012345678901234567890"),
			nonce: U256::from(42),
			initCode: Bytes::from(hex::decode("deadbeef").unwrap()),
			callData: Bytes::from(hex::decode("cafebabe").unwrap()),
			accountGasLimits: FixedBytes::from([1u8; 32]),
			preVerificationGas: U256::from(21000),
			gasFees: FixedBytes::from([2u8; 32]),
			paymasterAndData: Bytes::from(hex::decode("abcdef").unwrap()),
			signature: Bytes::from(hex::decode("445566").unwrap()),
		};

		// Get hash from EntryPoint contract
		let entrypoint_hash = entry_point_client
			.get_user_op_hash(user_op.clone())
			.await
			.expect("EntryPoint hash calculation should succeed");

		// Calculate hash locally
		let local_hash = calculate_user_operation_hash(&user_op, entry_point_address, chain_id);

		// Hashes should match exactly
		assert_eq!(
			entrypoint_hash, local_hash,
			"Local hash calculation should match EntryPoint.getUserOpHash()\nEntryPoint: {}\nLocal: {}",
			hex::encode(entrypoint_hash),
			hex::encode(local_hash)
		);
	}

	/// Additional integration test with real user operation data
	/// Tests against multiple different user operations to ensure consistency
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn test_multiple_user_operations_hash_consistency() {
		use crate::EntryPointClient;
		use alloy::hex;
		use oe_client_ethereum::AlloyRpcProvider;
		use std::sync::Arc;

		let entry_point_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let chain_id = 31337;

		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entry_point_client = EntryPointClient::new(entry_point_address, rpc_client);

		// Test multiple different user operations
		let test_cases = vec![
			// Minimal user operation
			PackedUserOperation {
				sender: address!("0x1111111111111111111111111111111111111111"),
				nonce: U256::from(0),
				initCode: Bytes::new(),
				callData: Bytes::new(),
				accountGasLimits: FixedBytes::ZERO,
				preVerificationGas: U256::from(21000),
				gasFees: FixedBytes::from([1u8; 32]),
				paymasterAndData: Bytes::new(),
				signature: Bytes::new(),
			},
			// Complex user operation with all fields populated
			PackedUserOperation {
				sender: address!("0x2222222222222222222222222222222222222222"),
				nonce: U256::from(12345),
				initCode: Bytes::from(hex::decode("60806040526000356101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550").unwrap()),
				callData: Bytes::from(hex::decode("a9059cbb0000000000000000000000003333333333333333333333333333333333333333000000000000000000000000000000000000000000000000016345785d8a0000").unwrap()),
				accountGasLimits: FixedBytes::from([0x12u8; 32]),
				preVerificationGas: U256::from(50000),
				gasFees: FixedBytes::from([0x34u8; 32]),
				paymasterAndData: Bytes::from(hex::decode("444444444444444444444444444444444444444400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002").unwrap()),
				signature: Bytes::from(hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1c").unwrap()),
			},
		];

		for (i, user_op) in test_cases.into_iter().enumerate() {
			let entrypoint_hash = entry_point_client
				.get_user_op_hash(user_op.clone())
				.await
				.expect(&format!("EntryPoint hash calculation should succeed for test case {}", i));

			let local_hash = calculate_user_operation_hash(&user_op, entry_point_address, chain_id);

			assert_eq!(
				entrypoint_hash, local_hash,
				"Test case {}: Local hash calculation should match EntryPoint.getUserOpHash()\nEntryPoint: {}\nLocal: {}",
				i,
				hex::encode(entrypoint_hash),
				hex::encode(local_hash)
			);
		}
	}
}
