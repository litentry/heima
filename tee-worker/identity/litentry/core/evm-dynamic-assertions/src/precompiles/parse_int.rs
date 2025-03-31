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
use ethabi::ethereum_types::U256;
use std::vec::Vec;

pub fn parse_int(input: Vec<u8>) -> PrecompileResult {
	let decoded = match ethabi::decode(&[ethabi::ParamType::String], &input) {
		Ok(d) => d,
		Err(e) => {
			log::debug!("Could not decode string {:?}, reason: {:?}", input, e);
			return Ok(failure_precompile_output(ethabi::Token::Uint(Default::default())));
		},
	};

	let string_value = decoded.get(0).and_then(|v| v.clone().into_string());

	let value = match string_value {
		Some(v) => match U256::from_dec_str(v.as_str()) {
			Ok(n) => n,
			Err(e) => {
				log::debug!("Cannot parse string {:?} to U256, reason: {:?}", v, e);
				return Ok(failure_precompile_output(ethabi::Token::Uint(Default::default())));
			},
		},
		None => {
			log::debug!("Could not decode input {:?}, reason: string value is invalid", input);
			return Ok(failure_precompile_output(ethabi::Token::Uint(Default::default())));
		},
	};

	Ok(success_precompile_output(ethabi::Token::Uint(value)))
}

#[cfg(test)]
pub mod test {
	use crate::{
		failure_precompile_output, precompiles::parse_int::parse_int, success_precompile_output,
	};
	use ethabi::{encode, Token};

	#[test]
	fn test_parse_int() {
		// given
		let encoded = encode(&[Token::String("100000000000000001".into())]);

		// when
		let result = parse_int(encoded).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Uint(100000000000000001_u128.into())), result)
	}

	#[test]
	fn test_parse_int_fail() {
		// given
		let encoded = encode(&[Token::String("123XYZ".to_string())]);

		// when
		let result = parse_int(encoded).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::Uint(Default::default())), result)
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/ParseInt.sol
	const FUNCTION_HASH: &str = "1fc5c204"; // callParseInt(string)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b506103ba806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80631fc5c20414610030575b600080fd5b61004a60048036038101906100459190610234565b610061565b6040516100589291906102b1565b60405180910390f35b60008061006d83610076565b91509150915091565b60008060008360405160200161008c9190610362565b60405160208183030381529060405290506000815190506040516082818360208601600061041f600019f16100c057600080fd5b805194506020810151935060828101604052505050915091565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b610141826100f8565b810181811067ffffffffffffffff821117156101605761015f610109565b5b80604052505050565b60006101736100da565b905061017f8282610138565b919050565b600067ffffffffffffffff82111561019f5761019e610109565b5b6101a8826100f8565b9050602081019050919050565b82818337600083830152505050565b60006101d76101d284610184565b610169565b9050828152602081018484840111156101f3576101f26100f3565b5b6101fe8482856101b5565b509392505050565b600082601f83011261021b5761021a6100ee565b5b813561022b8482602086016101c4565b91505092915050565b60006020828403121561024a576102496100e4565b5b600082013567ffffffffffffffff811115610268576102676100e9565b5b61027484828501610206565b91505092915050565b60008115159050919050565b6102928161027d565b82525050565b6000819050919050565b6102ab81610298565b82525050565b60006040820190506102c66000830185610289565b6102d360208301846102a2565b9392505050565b600081519050919050565b600082825260208201905092915050565b60005b838110156103145780820151818401526020810190506102f9565b83811115610323576000848401525b50505050565b6000610334826102da565b61033e81856102e5565b935061034e8185602086016102f6565b610357816100f8565b840191505092915050565b6000602082019050818103600083015261037c8184610329565b90509291505056fea26469706673582212201792beef9f71280990133d525ed3ac2f774f78b67e37f1c9606aa22a009a71f664736f6c634300080b0033";

	#[test]
	pub fn test_parse_int() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Uint(256)];

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[Token::String("100000000000000001".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		let expected_result: U256 = 100000000000000001_u128.into();
		assert_eq!(expected_result, decoded[1].clone().into_uint().unwrap());
	}

	#[test]
	pub fn test_parse_int_fail() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Uint(256)];

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[Token::String("123XYZ".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		let expected_result: U256 = U256::zero();
		assert_eq!(expected_result, decoded[1].clone().into_uint().unwrap());
	}
}
