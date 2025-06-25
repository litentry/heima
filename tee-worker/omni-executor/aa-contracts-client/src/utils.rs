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

use alloy::primitives::{keccak256, Address, Bytes, FixedBytes, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::SolCall;

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

/// Calculate the CREATE2 address for a SmartAccount without calling the EntryPoint
/// This mirrors the getAddress function in SmartAccountFactory.sol
pub fn calculate_smart_account_address(
	factory_address: Address,
	account_implementation: Address,
	oa: FixedBytes<32>,
	client_id: &[u8],
	root: Address,
) -> Address {
	use crate::types::initializeCall;

	let initialize_call = initializeCall { oa, clientId: Bytes::from(client_id.to_vec()), root };
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

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::primitives::address;

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
	fn test_calculate_smart_account_address() {
		use alloy::hex;

		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let account_implementation = address!("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0");
		let oa = FixedBytes::from_slice(
			&hex::decode("6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd96211391")
				.unwrap(),
		);
		let client_id = b"test_client";
		let root = address!("0x0000000000000000000000000000000000000001");

		let calculated_address = calculate_smart_account_address(
			factory_address,
			account_implementation,
			oa,
			client_id,
			root,
		);

		assert_ne!(calculated_address, Address::ZERO);

		let calculated_address_2 = calculate_smart_account_address(
			factory_address,
			account_implementation,
			oa,
			client_id,
			root,
		);
		assert_eq!(calculated_address, calculated_address_2);
	}

	#[test]
	fn test_calculate_smart_account_address_different_parameters() {
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

		let address_1 = calculate_smart_account_address(
			factory_address,
			account_implementation,
			oa,
			client_id_1,
			root,
		);

		let address_2 = calculate_smart_account_address(
			factory_address,
			account_implementation,
			oa,
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
}
