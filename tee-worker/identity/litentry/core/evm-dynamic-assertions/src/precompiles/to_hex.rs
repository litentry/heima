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
use std::{format, vec::Vec};

pub fn to_hex(input: Vec<u8>) -> PrecompileResult {
	let decoded = match ethabi::decode(&[ethabi::ParamType::Bytes], &input) {
		Ok(d) => d,
		Err(e) => {
			log::debug!("Could not decode bytes {:?}, reason: {:?}", input, e);
			return Ok(failure_precompile_output(ethabi::Token::String(Default::default())));
		},
	};
	let bytes = match decoded.get(0).and_then(|v| v.clone().into_bytes()) {
		Some(v) => v,
		None => {
			log::debug!("Could not convert decoded[0] to bytes");
			return Ok(failure_precompile_output(ethabi::Token::String(Default::default())));
		},
	};
	let hex_encoded = format!("0x{}", hex::encode(&bytes));
	Ok(success_precompile_output(ethabi::Token::String(hex_encoded)))
}

#[cfg(test)]
pub mod test {
	use crate::{
		failure_precompile_output, precompiles::to_hex::to_hex, success_precompile_output,
	};
	use ethabi::{encode, Token};

	#[test]
	pub fn test_to_hex() {
		// given
		let bytes = [1, 2, 3, 4];
		let encoded = encode(&[Token::Bytes(bytes.to_vec())]);

		// when
		let result = to_hex(encoded).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::String("0x01020304".to_string())), result)
	}

	#[test]
	pub fn test_to_hex_fail() {
		// given
		let encoded = encode(&[]);

		// when
		let result = to_hex(encoded).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result)
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{decode, encode, ParamType, Token};

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/ToHex.sol
	const FUNCTION_HASH: &str = "8876183c"; // callToHex(string)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b506103ff806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80638876183c14610030575b600080fd5b61004a60048036038101906100459190610236565b610061565b604051610058929190610322565b60405180910390f35b6000606061006e83610077565b91509150915091565b6000606060008360405160200161008e91906103a7565b6040516020818303038152906040529050600081519050604051611000818360208601600061041b600019f16100c357600080fd5b8094506040810193506110008101604052505050915091565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b610143826100fa565b810181811067ffffffffffffffff821117156101625761016161010b565b5b80604052505050565b60006101756100dc565b9050610181828261013a565b919050565b600067ffffffffffffffff8211156101a1576101a061010b565b5b6101aa826100fa565b9050602081019050919050565b82818337600083830152505050565b60006101d96101d484610186565b61016b565b9050828152602081018484840111156101f5576101f46100f5565b5b6102008482856101b7565b509392505050565b600082601f83011261021d5761021c6100f0565b5b813561022d8482602086016101c6565b91505092915050565b60006020828403121561024c5761024b6100e6565b5b600082013567ffffffffffffffff81111561026a576102696100eb565b5b61027684828501610208565b91505092915050565b60008115159050919050565b6102948161027f565b82525050565b600081519050919050565b600082825260208201905092915050565b60005b838110156102d45780820151818401526020810190506102b9565b838111156102e3576000848401525b50505050565b60006102f48261029a565b6102fe81856102a5565b935061030e8185602086016102b6565b610317816100fa565b840191505092915050565b6000604082019050610337600083018561028b565b818103602083015261034981846102e9565b90509392505050565b600081519050919050565b600082825260208201905092915050565b600061037982610352565b610383818561035d565b93506103938185602086016102b6565b61039c816100fa565b840191505092915050565b600060208201905081810360008301526103c1818461036e565b90509291505056fea26469706673582212209868a7cc2e186fcf1850b88ae8ea9657e8ae69edb105e38004a6a97e91d2a34e64736f6c634300080b0033";

	#[test]
	pub fn test_to_hex() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[Token::String("test".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			format!("0x{}", &hex::encode("test")),
			decoded[1].clone().into_string().unwrap()
		);
	}
}
