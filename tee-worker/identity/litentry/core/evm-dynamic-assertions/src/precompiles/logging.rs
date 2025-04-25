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
	failure_precompile_output, precompiles::PrecompileResult, success_precompile_output,
	Precompiles,
};

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::chrono::{
	offset::Utc as TzUtc, DateTime, NaiveDateTime, SecondsFormat,
};

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "std")]
use chrono::{offset::Utc as TzUtc, DateTime, SecondsFormat};

use std::{format, string::String, vec::Vec};

pub const LOGGING_LEVEL_DEBUG: u8 = 0;
#[allow(dead_code)]
pub const LOGGING_LEVEL_INFO: u8 = 1;
pub const LOGGING_LEVEL_WARN: u8 = 2;
pub const LOGGING_LEVEL_ERROR: u8 = 3;
pub const LOGGING_LEVEL_FATAL: u8 = 4;

pub fn logging(input: Vec<u8>, precompiles: &Precompiles) -> PrecompileResult {
	let decoded =
		match ethabi::decode(&[ethabi::ParamType::Uint(8), ethabi::ParamType::String], &input) {
			Ok(d) => d,
			Err(e) => {
				log::debug!("Could not decode input {:?}, reason: {:?}", input, e);
				return Ok(failure_precompile_output(ethabi::Token::Bool(Default::default())));
			},
		};
	let level = match decoded.get(0).and_then(|v| v.clone().into_uint()) {
		Some(v) => v,
		None => {
			log::debug!("Could not convert decoded[0] to uint");
			return Ok(failure_precompile_output(ethabi::Token::Bool(Default::default())));
		},
	}
	.as_u32() as u8;
	let message = match decoded.get(1).and_then(|v| v.clone().into_string()) {
		Some(v) => v,
		None => {
			log::debug!("Could not convert decoded[1] to string");
			return Ok(failure_precompile_output(ethabi::Token::Bool(Default::default())));
		},
	};

	contract_logging(precompiles, level, message);

	Ok(success_precompile_output(ethabi::Token::Bool(true)))
}

pub fn contract_logging(precompiles: &Precompiles, level: u8, message: String) {
	precompiles.contract_logs.borrow_mut().push(format!(
		"[{} {}] {}",
		now().to_rfc3339_opts(SecondsFormat::Micros, true),
		loggin_level_to_string(level),
		message
	));
}

fn loggin_level_to_string(level: u8) -> String {
	match level {
		LOGGING_LEVEL_DEBUG => "DEBUG",
		LOGGING_LEVEL_WARN => "WARN",
		LOGGING_LEVEL_ERROR => "ERROR",
		LOGGING_LEVEL_FATAL => "FATAL",
		_ => "INFO",
	}
	.into()
}

fn now() -> DateTime<TzUtc> {
	#[cfg(feature = "std")]
	{
		TzUtc::now()
	}

	#[cfg(all(not(feature = "std"), feature = "sgx"))]
	{
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("system time before Unix epoch");
		let naive =
			NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos() as u32)
				.unwrap();

		DateTime::from_utc(naive, TzUtc)
	}
}

#[cfg(test)]
pub mod test {
	use crate::{
		failure_precompile_output,
		precompiles::logging::{logging, LOGGING_LEVEL_INFO},
		success_precompile_output, Precompiles,
	};
	use ethabi::{encode, Token};

	#[test]
	pub fn test_logging() {
		// given
		let encoded = encode(&[
			Token::Uint(LOGGING_LEVEL_INFO.into()),
			Token::String("This is an info message".into()),
		]);

		// when
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = logging(encoded, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Bool(true)), result);
		assert_eq!(precompiles.contract_logs.borrow_mut().len(), 1);
	}

	#[test]
	pub fn test_logging_fail() {
		// given
		let encoded = encode(&[Token::String("This is an info message".into())]);

		// when
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = logging(encoded, &precompiles).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::Bool(Default::default())), result);
		assert_eq!(precompiles.contract_logs.borrow_mut().len(), 0);
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, precompiles::logging::*, prepare_function_call_input};
	use ethabi::{encode, Token};

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/Logging.sol
	const FUNCTION_HASH: &str = "a5c89545"; // callLogging(uint8,string)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b50610413806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063a5c8954514610030575b600080fd5b61004a60048036038101906100459190610252565b61004c565b005b61006b8260ff166004811115610065576100646102ae565b5b8261006f565b5050565b600082826040516020016100849291906103ad565b60405160208183030381529060405290506000815190506040516040818360208601600061041a600019f16100b857600080fd5b5050505050565b6000604051905090565b600080fd5b600080fd5b600060ff82169050919050565b6100e9816100d3565b81146100f457600080fd5b50565b600081359050610106816100e0565b92915050565b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b61015f82610116565b810181811067ffffffffffffffff8211171561017e5761017d610127565b5b80604052505050565b60006101916100bf565b905061019d8282610156565b919050565b600067ffffffffffffffff8211156101bd576101bc610127565b5b6101c682610116565b9050602081019050919050565b82818337600083830152505050565b60006101f56101f0846101a2565b610187565b90508281526020810184848401111561021157610210610111565b5b61021c8482856101d3565b509392505050565b600082601f8301126102395761023861010c565b5b81356102498482602086016101e2565b91505092915050565b60008060408385031215610269576102686100c9565b5b6000610277858286016100f7565b925050602083013567ffffffffffffffff811115610298576102976100ce565b5b6102a485828601610224565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602160045260246000fd5b600581106102ee576102ed6102ae565b5b50565b60008190506102ff826102dd565b919050565b600061030f826102f1565b9050919050565b61031f81610304565b82525050565b600081519050919050565b600082825260208201905092915050565b60005b8381101561035f578082015181840152602081019050610344565b8381111561036e576000848401525b50505050565b600061037f82610325565b6103898185610330565b9350610399818560208601610341565b6103a281610116565b840191505092915050565b60006040820190506103c26000830185610316565b81810360208301526103d48184610374565b9050939250505056fea264697066735822122013f32e65e5c8e0d8c134673c0a6f6cf67ba2acb957c1c6b87b4580b4f182777f64736f6c634300080b0033";

	#[test]
	pub fn test_logging() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();

		let result = execute_smart_contract(
			byte_code.clone(),
			prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(LOGGING_LEVEL_DEBUG.into()),
					Token::String("This is a debug message".to_string()),
				]),
			)
			.unwrap(),
		);
		assert_eq!(&result.2[0][29..], "DEBUG] This is a debug message");

		let result = execute_smart_contract(
			byte_code.clone(),
			prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(LOGGING_LEVEL_INFO.into()),
					Token::String("This is an info message".to_string()),
				]),
			)
			.unwrap(),
		);
		assert_eq!(&result.2[0][29..], "INFO] This is an info message");

		let result = execute_smart_contract(
			byte_code.clone(),
			prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(LOGGING_LEVEL_WARN.into()),
					Token::String("This is a warn message".to_string()),
				]),
			)
			.unwrap(),
		);
		assert_eq!(&result.2[0][29..], "WARN] This is a warn message");

		let result = execute_smart_contract(
			byte_code.clone(),
			prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(LOGGING_LEVEL_ERROR.into()),
					Token::String("This is a error message".to_string()),
				]),
			)
			.unwrap(),
		);
		assert_eq!(&result.2[0][29..], "ERROR] This is a error message");

		let result = execute_smart_contract(
			byte_code,
			prepare_function_call_input(
				FUNCTION_HASH,
				encode(&[
					Token::Uint(LOGGING_LEVEL_FATAL.into()),
					Token::String("This is a fatal message".to_string()),
				]),
			)
			.unwrap(),
		);
		assert_eq!(&result.2[0][29..], "FATAL] This is a fatal message");
	}
}
