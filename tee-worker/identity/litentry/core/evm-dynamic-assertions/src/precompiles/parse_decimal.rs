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

use crate::{
	alloc::string::ToString, failure_precompile_output, precompiles::PrecompileResult,
	success_precompile_output,
};
use ethabi::ethereum_types::U256;
use rust_decimal::prelude::{Decimal, FromStr};
use std::vec::Vec;

pub fn parse_decimal(input: Vec<u8>) -> PrecompileResult {
	let decoded =
		match ethabi::decode(&[ethabi::ParamType::String, ethabi::ParamType::Uint(8)], &input) {
			Ok(d) => d,
			Err(e) => {
				log::debug!("Could not decode input {:?}, reason: {:?}", input, e);
				return Ok(failure_precompile_output(ethabi::Token::Uint(Default::default())));
			},
		};

	let string_value = decoded.get(0).and_then(|v| v.clone().into_string());
	let decimals = decoded
		.get(1)
		.and_then(|t| t.clone().into_uint())
		.map(|v| v.as_u32())
		.unwrap_or(0);

	let value = match string_value {
		Some(s) => {
			let decimal = match Decimal::from_str(s.as_str()) {
				Ok(d) => d,
				Err(e) => {
					log::debug!("Cannot parse string {:?} to decimal, reason: {:?}", s, e);
					return Ok(failure_precompile_output(ethabi::Token::Uint(Default::default())));
				},
			};

			// Split the decimal into integer and fractional parts
			let integer_part = decimal.trunc();
			let fractional_part = decimal.fract();

			// Process the integer part
			let integer_string = integer_part.to_string();
			let integer_u256 = U256::from_dec_str(&integer_string).unwrap()
				* U256::from(10).pow(U256::from(decimals));

			// Process the fractional part
			let fractional_string = (fractional_part * Decimal::new(10_i64.pow(decimals), 0))
				.normalize()
				.to_string();
			let fractional_u256 = U256::from_dec_str(&fractional_string).unwrap();

			// Combine the results
			integer_u256 + fractional_u256
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
		failure_precompile_output, precompiles::parse_decimal::parse_decimal,
		success_precompile_output,
	};
	use ethabi::{encode, Token};

	#[test]
	pub fn test_parse_decimal() {
		// given
		let encoded =
			encode(&[Token::String("1.00000000000000001".into()), Token::Uint(18.into())]);

		// when
		let result = parse_decimal(encoded).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Uint(1000000000000000010_u128.into())), result)
	}

	#[test]
	pub fn test_parse_decimal_fail() {
		// given
		let encoded =
			encode(&[Token::String("1.0000000000000000A".to_string()), Token::Uint(18.into())]);

		// when
		let result = parse_decimal(encoded).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::Uint(Default::default())), result)
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/ParseDecimal.sol
	const FUNCTION_HASH: &str = "1abfaf23"; // callParseDecimal(string,uint8)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b5061042a806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80631abfaf2314610030575b600080fd5b61004a60048036038101906100459190610274565b610061565b604051610058929190610304565b60405180910390f35b60008061006e8484610079565b915091509250929050565b600080600084846040516020016100919291906103c4565b60405160208183030381529060405290506000815190506040516082818360208601600061041e600019f16100c557600080fd5b8051945060208101519350608281016040525050509250929050565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b610148826100ff565b810181811067ffffffffffffffff8211171561016757610166610110565b5b80604052505050565b600061017a6100e1565b9050610186828261013f565b919050565b600067ffffffffffffffff8211156101a6576101a5610110565b5b6101af826100ff565b9050602081019050919050565b82818337600083830152505050565b60006101de6101d98461018b565b610170565b9050828152602081018484840111156101fa576101f96100fa565b5b6102058482856101bc565b509392505050565b600082601f830112610222576102216100f5565b5b81356102328482602086016101cb565b91505092915050565b600060ff82169050919050565b6102518161023b565b811461025c57600080fd5b50565b60008135905061026e81610248565b92915050565b6000806040838503121561028b5761028a6100eb565b5b600083013567ffffffffffffffff8111156102a9576102a86100f0565b5b6102b58582860161020d565b92505060206102c68582860161025f565b9150509250929050565b60008115159050919050565b6102e5816102d0565b82525050565b6000819050919050565b6102fe816102eb565b82525050565b600060408201905061031960008301856102dc565b61032660208301846102f5565b9392505050565b600081519050919050565b600082825260208201905092915050565b60005b8381101561036757808201518184015260208101905061034c565b83811115610376576000848401525b50505050565b60006103878261032d565b6103918185610338565b93506103a1818560208601610349565b6103aa816100ff565b840191505092915050565b6103be8161023b565b82525050565b600060408201905081810360008301526103de818561037c565b90506103ed60208301846103b5565b939250505056fea2646970667358221220e2176c1afbb0895b7b8e73026b1ecb7859a6286cca205ecd1d1520f81539788764736f6c634300080b0033";

	#[test]
	pub fn test_parse_decimal() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Uint(256)];

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[Token::String("1.00000000000000001".to_string()), Token::Uint(18.into())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		let expected_result: U256 = 1000000000000000010_u128.into();
		assert_eq!(expected_result, decoded[1].clone().into_uint().unwrap());
	}

	#[test]
	pub fn test_parse_decimal_fail() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Uint(256)];

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH,
			encode(&[Token::String("1.0000000000000000A".to_string()), Token::Uint(18.into())]),
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
