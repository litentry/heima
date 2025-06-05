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

use crate::{failure_precompile_output, precompiles::PrecompileResult, success_precompile_output};
use base58::ToBase58;
use blake2_rfc::blake2b::Blake2b;
use litentry_primitives::{
	p2pkh_address, p2sh_address, p2tr_address, p2wpkh_address, p2wsh_address, Web3Network,
};
use ss58_registry::Ss58AddressFormat;
use std::{format, string::String, vec, vec::Vec};

pub fn identity_to_string(input: Vec<u8>) -> PrecompileResult {
	let decoded =
		match ethabi::decode(&[ethabi::ParamType::Uint(32), ethabi::ParamType::Bytes], &input) {
			Ok(d) => d,
			Err(e) => {
				log::debug!("Could not decode input {:?}, reason: {:?}", input, e);
				return Ok(failure_precompile_output(ethabi::Token::String(Default::default())));
			},
		};

	let network_type = decoded.get(0).and_then(|v| v.clone().into_uint().map(|t| t.as_u32() as u8));
	let identity_value = decoded.get(1).and_then(|v| v.clone().into_bytes());

	let value = match (network_type, identity_value) {
		// Substrate
		(Some(n), Some(v)) if n <= Web3Network::SubstrateTestnet.get_code() => {
			let network = match web3_network_to_chain(n) {
				Ok(s) => s,
				Err(e) => {
					log::debug!("{:?}", e);
					return Ok(failure_precompile_output(
						ethabi::Token::String(Default::default()),
					));
				},
			};
			match ss58_address_of(v.as_ref(), &network) {
				Ok(s) => s,
				Err(e) => {
					log::debug!("Cannot parse {:?} to ss58 address, reason: {:?}", v, e);
					return Ok(failure_precompile_output(
						ethabi::Token::String(Default::default()),
					));
				},
			}
		},
		// Evm
		(Some(n), Some(v))
			if [
				Web3Network::Ethereum.get_code(),
				Web3Network::Bsc.get_code(),
				Web3Network::Polygon.get_code(),
				Web3Network::Arbitrum.get_code(),
				Web3Network::Combo.get_code(),
			]
			.contains(&n) =>
		{
			format!("0x{}", hex::encode(&v))
		},
		// Bitcoin
		(Some(n), Some(v))
			if n >= Web3Network::BitcoinP2tr.get_code()
				&& n <= Web3Network::BitcoinP2wsh.get_code() =>
		{
			let address = hex::encode(&v);
			pubkey_to_address(n, &address)
		},
		// Solana
		(Some(n), Some(v)) if n == Web3Network::Solana.get_code() => v.to_base58(),
		_ => {
			log::debug!(
				"Could not decode input {:?}, reason: network type or identity value is invalid",
				input
			);
			return Ok(failure_precompile_output(ethabi::Token::String(Default::default())));
		},
	};

	Ok(success_precompile_output(ethabi::Token::String(value)))
}

// mostly copied from https://github.com/hack-ink/substrate-minimal/blob/main/subcryptor/src/lib.rs
// no_std version is used here
fn ss58_address_of(public_key: &[u8], network: &str) -> core::result::Result<String, String> {
	let network = Ss58AddressFormat::try_from(network).map_err(|e| {
		format!("Fail to parse ss58 address, network: {:?}, reason: {:?}", network, e)
	})?;
	let prefix = u16::from(network);
	let mut bytes = match prefix {
		0..=63 => vec![prefix as u8],
		64..=16_383 => {
			let first = ((prefix & 0b0000_0000_1111_1100) as u8) >> 2;
			let second = ((prefix >> 8) as u8) | ((prefix & 0b0000_0000_0000_0011) as u8) << 6;

			vec![first | 0b01000000, second]
		},
		_ => Err(format!("Fail to parse ss58 address, network: {:?}", network))?,
	};

	bytes.extend(public_key);

	let blake2b = {
		let mut context = Blake2b::new(64);
		context.update(b"SS58PRE");
		context.update(&bytes);
		context.finalize()
	};

	bytes.extend(&blake2b.as_bytes()[0..2]);

	Ok(bytes.to_base58())
}

fn web3_network_to_chain(network: u8) -> Result<String, String> {
	Web3Network::from_code(network)
		.map(|v| v.get_name())
		.ok_or(format!("Invalid network: {:?}", network))
}

fn pubkey_to_address(network: u8, pubkey: &str) -> String {
	match network {
		// BitcoinP2tr
		9 => p2tr_address(pubkey),
		// BitcoinP2pkh
		10 => p2pkh_address(pubkey),
		// BitcoinP2sh
		11 => p2sh_address(pubkey),
		// BitcoinP2wpkh
		12 => p2wpkh_address(pubkey),
		// BitcoinP2wsh
		13 => p2wsh_address(pubkey),
		// others
		_ => "".into(),
	}
}

#[cfg(test)]
pub mod test {
	use crate::{
		failure_precompile_output, precompiles::identity_to_string::identity_to_string,
		success_precompile_output,
	};
	use base58::FromBase58;
	use ethabi::{encode, Token};
	use litentry_primitives::{decode_hex, Web3Network};

	#[test]
	fn test_substrate_identity_to_string() {
		let address = "0xd4e35b16ec6b417386b948e7eaf5cc642a243096cecf366e6313689b90969f42";

		vec![
			(Web3Network::Polkadot.get_code(), "15p8h3KAmkREatSn2e9TkD7ALJDo5UXZC56q7Bat2QQdxRgn"),
			(Web3Network::Kusama.get_code(), "HPTD2PyYLAgu1FhqhuWW1e1dGWPBqnbZxD6LYsUx7bcXD1j"),
			(Web3Network::Litentry.get_code(), "4BDaho6fgmXmRhzA79RCT9MLNLeunA3h3EGuBNXDNuqkgerR"),
			(Web3Network::Litmus.get_code(), "jcS3pqDZ5mnXNxkLTBM3kHa5ypL4pSi39CWGoCASC51onUzHW"),
		]
		.into_iter()
		.for_each(|(network, expected_address)| {
			// given
			let encoded = encode(&[
				Token::Uint(network.into()),
				Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
			]);

			// when
			let result = identity_to_string(encoded).unwrap();

			// then
			assert_eq!(success_precompile_output(Token::String(expected_address.into())), result);
		});

		// Unsupported networks below
		// LitentryRococo
		// Khala
		// SubstrateTestnet
	}

	#[test]
	fn test_evm_identity_to_string() {
		let address = "0x582d872a1b094fc48f5de31d3b73f2d9be47def1";

		vec![
			Web3Network::Ethereum,
			Web3Network::Bsc,
			Web3Network::Polygon,
			Web3Network::Arbitrum,
			Web3Network::Combo,
		]
		.into_iter()
		.map(|v| v.get_code())
		.for_each(|network| {
			// given
			let encoded = encode(&[
				Token::Uint(network.into()),
				Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
			]);

			// when
			let result = identity_to_string(encoded).unwrap();

			// then
			assert_eq!(success_precompile_output(Token::String(address.into())), result);
		});
	}

	#[test]
	fn test_bitcoin_identity_to_string() {
		// given
		let address = "0x02e8c39e82aaaa143c3def8d3c7084a539b227244ac9067c3f7fc86cb73a0b7aed";
		// BitcoinP2tr
		let encoded = encode(&[
			Token::Uint(Web3Network::BitcoinP2tr.get_code().into()),
			Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
		]);

		// when
		let result = identity_to_string(encoded).unwrap();

		// then
		assert_eq!(
			success_precompile_output(Token::String(
				"bc1pgr5fw4p9gl9me0vzjklnlnap669caxc0gsk4j62gff2qktlw6naqm4m3d0".into()
			)),
			result
		);

		// BitcoinP2wsh
		let encoded = encode(&[
			Token::Uint(Web3Network::BitcoinP2wsh.get_code().into()),
			Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
		]);

		// when
		let result = identity_to_string(encoded).unwrap();

		// then
		assert_eq!(
			success_precompile_output(Token::String(
				"bc1qvr0n2tgcevl26kx0vu76nujlju6fwkdzllv7qx5pz5ed3y8yf22st9hqmw".into()
			)),
			result
		);
	}

	#[test]
	fn test_solana_identity_to_string() {
		// given
		let address = "EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6";
		let encoded = encode(&[
			Token::Uint(Web3Network::Solana.get_code().into()),
			Token::Bytes(address.from_base58().unwrap()),
		]);

		// when
		let result = identity_to_string(encoded).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::String(address.into())), result);
	}

	#[test]
	fn test_identity_to_string_fail() {
		// given
		let encoded = encode(&[Token::Uint(Web3Network::Ethereum.get_code().into())]);

		// when
		let result = identity_to_string(encoded).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result);
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use base58::FromBase58;
	use ethabi::{decode, encode, ParamType, Token};
	use litentry_primitives::{decode_hex, Web3Network};

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/IdentityToString.sol
	const FUNCTION_HASH: &str = "10e6b834"; // callIdentityToString(uint32,bytes)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b50610472806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c806310e6b83414610030575b600080fd5b61004a60048036038101906100459190610279565b610061565b604051610058929190610378565b60405180910390f35b6000606061006f848461007a565b915091509250929050565b600060606000848460405160200161009392919061040c565b6040516020818303038152906040529050600081519050604051611000818360208601600061041c600019f16100c857600080fd5b80945060408101935061100081016040525050509250929050565b6000604051905090565b600080fd5b600080fd5b600063ffffffff82169050919050565b610110816100f7565b811461011b57600080fd5b50565b60008135905061012d81610107565b92915050565b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6101868261013d565b810181811067ffffffffffffffff821117156101a5576101a461014e565b5b80604052505050565b60006101b86100e3565b90506101c4828261017d565b919050565b600067ffffffffffffffff8211156101e4576101e361014e565b5b6101ed8261013d565b9050602081019050919050565b82818337600083830152505050565b600061021c610217846101c9565b6101ae565b90508281526020810184848401111561023857610237610138565b5b6102438482856101fa565b509392505050565b600082601f8301126102605761025f610133565b5b8135610270848260208601610209565b91505092915050565b600080604083850312156102905761028f6100ed565b5b600061029e8582860161011e565b925050602083013567ffffffffffffffff8111156102bf576102be6100f2565b5b6102cb8582860161024b565b9150509250929050565b60008115159050919050565b6102ea816102d5565b82525050565b600081519050919050565b600082825260208201905092915050565b60005b8381101561032a57808201518184015260208101905061030f565b83811115610339576000848401525b50505050565b600061034a826102f0565b61035481856102fb565b935061036481856020860161030c565b61036d8161013d565b840191505092915050565b600060408201905061038d60008301856102e1565b818103602083015261039f818461033f565b90509392505050565b6103b1816100f7565b82525050565b600081519050919050565b600082825260208201905092915050565b60006103de826103b7565b6103e881856103c2565b93506103f881856020860161030c565b6104018161013d565b840191505092915050565b600060408201905061042160008301856103a8565b818103602083015261043381846103d3565b9050939250505056fea2646970667358221220a8e59fddeaf8d08bc21acecdf46a244285a286724932b9533e674b6c2d22046464736f6c634300080b0033";

	#[test]
	pub fn test_substrate_identity_to_string() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		let address = "0xd4e35b16ec6b417386b948e7eaf5cc642a243096cecf366e6313689b90969f42";

		vec![
			(Web3Network::Polkadot.get_code(), "15p8h3KAmkREatSn2e9TkD7ALJDo5UXZC56q7Bat2QQdxRgn"),
			(Web3Network::Kusama.get_code(), "HPTD2PyYLAgu1FhqhuWW1e1dGWPBqnbZxD6LYsUx7bcXD1j"),
			(Web3Network::Litentry.get_code(), "4BDaho6fgmXmRhzA79RCT9MLNLeunA3h3EGuBNXDNuqkgerR"),
			(Web3Network::Litmus.get_code(), "jcS3pqDZ5mnXNxkLTBM3kHa5ypL4pSi39CWGoCASC51onUzHW"),
		]
		.into_iter()
		.for_each(|(network, expected_address)| {
			// given
			let input_data = prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(network.into()),
					Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
				]),
			)
			.unwrap();

			// when
			let (_, return_data, _) = execute_smart_contract(byte_code.clone(), input_data);

			// then
			let decoded = decode(&return_types, &return_data).unwrap();
			assert_eq!(true, decoded[0].clone().into_bool().unwrap());
			assert_eq!(expected_address, decoded[1].clone().into_string().unwrap());
		});
	}

	#[test]
	fn test_evm_identity_to_string() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		let address = "0x582d872a1b094fc48f5de31d3b73f2d9be47def1";

		vec![
			Web3Network::Ethereum,
			Web3Network::Bsc,
			Web3Network::Polygon,
			Web3Network::Arbitrum,
			Web3Network::Combo,
		]
		.into_iter()
		.map(|v| v.get_code())
		.for_each(|network| {
			// given
			let input_data = prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(network.into()),
					Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
				]),
			)
			.unwrap();

			// when
			let (_, return_data, _) = execute_smart_contract(byte_code.clone(), input_data);

			// then
			let decoded = decode(&return_types, &return_data).unwrap();
			assert_eq!(true, decoded[0].clone().into_bool().unwrap());
			assert_eq!(address, decoded[1].clone().into_string().unwrap());
		});
	}

	#[test]
	fn test_bitcoin_identity_to_string() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let address = "0x02e8c39e82aaaa143c3def8d3c7084a539b227244ac9067c3f7fc86cb73a0b7aed";
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[
				Token::Uint(Web3Network::BitcoinP2tr.get_code().into()),
				Token::Bytes(decode_hex(address.as_bytes()).unwrap()),
			]),
		)
		.unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// when
		let decoded = decode(&return_types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			"bc1pgr5fw4p9gl9me0vzjklnlnap669caxc0gsk4j62gff2qktlw6naqm4m3d0",
			decoded[1].clone().into_string().unwrap()
		);
	}

	#[test]
	fn test_solana_identity_to_string() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let address = "EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6";
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[
				Token::Uint(Web3Network::Solana.get_code().into()),
				Token::Bytes(address.from_base58().unwrap()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(address, decoded[1].clone().into_string().unwrap());
	}
}
