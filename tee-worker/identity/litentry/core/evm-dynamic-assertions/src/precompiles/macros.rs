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

use ethabi::Token;
use itc_rest_client::http_client::SendHttpRequest;
#[macro_export]
macro_rules! json_get_fn {
	($name:ident, $token:ident, $parse_fn_name:ident) => {
		pub fn $name(input: Vec<u8>, precompiles: &Precompiles) -> $crate::precompiles::PrecompileResult {
			let decoded = match ethabi::decode(
				&[ethabi::ParamType::String, ethabi::ParamType::String],
				&input,
			) {
				Ok(d) => d,
				Err(e) => {
					let message = std::format!("Could not decode bytes {:?}, reason: {:?}", input, e);
					log::debug!("{}", message);
					contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
					return Ok(failure_precompile_output(ethabi::Token::$token(Default::default())))
				},
			};

			// safe to unwrap
			let json = decoded.get(0).unwrap().clone().into_string().unwrap();
			let pointer = decoded.get(1).unwrap().clone().into_string().unwrap();
			let value: serde_json::Value = match serde_json::from_str(&json) {
				Ok(v) => v,
				Err(e) => {
					let message = std::format!("Could not parse json {:?}, reason: {:?}", json, e);
					log::debug!("{}", message);
					contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
					return Ok(failure_precompile_output(ethabi::Token::$token(Default::default())))
				},
			};
			let result = match value.pointer(&pointer) {
				Some(v) => v,
				None => {
					let message = std::format!("No value under given pointer: :{:?}", pointer);
					log::debug!("{}", message);
					contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
					return Ok(failure_precompile_output(ethabi::Token::$token(Default::default())))
				},
			};

			let encoded = match result.$parse_fn_name() {
				Some(v) => ethabi::Token::$token(v.into()),
				None => {
					let message = std::format!(
						"There is no value or it might be of different type, pointer: ${:?}",
						pointer
					);
					log::debug!("{}", message);
					contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
					return Ok(failure_precompile_output(ethabi::Token::$token(Default::default())))
				},
			};
			Ok(success_precompile_output(encoded))
		}
	};
}

#[macro_export]
macro_rules! http_get_precompile_fn {
	($name:ident, $token:ident, $parse_fn_name:ident) => {
		pub fn $name<T: SendHttpRequest>(
			input: Vec<u8>,
			client: T,
		) -> $crate::precompiles::PrecompileResult {
			let decoded = match ethabi::decode(
				&[
					ethabi::ParamType::String,
					ethabi::ParamType::String,
					ethabi::ParamType::Array(
						ethabi::ParamType::Tuple(vec![
							ethabi::ParamType::String,
							ethabi::ParamType::String,
						])
						.into(),
					),
				],
				&input,
			) {
				Ok(d) => d,
				Err(e) => {
					log::debug!("Could not decode bytes {:?}, reason: {:?}", input, e);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};
			let value: serde_json::Value =
				match $crate::precompiles::macros::do_get(client, &decoded, 0, 2) {
					Ok(v) => v,
					Err(_) => {
						return Ok(failure_precompile_output(ethabi::Token::$token(
							Default::default(),
						)))
					},
				};

			// safe to unwrap
			let pointer = decoded.get(1).unwrap().clone().into_string().unwrap();
			let result = match value.pointer(&pointer) {
				Some(v) => v,
				None => {
					log::debug!("No value under given pointer: :{:?}", pointer);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};

			let encoded = match result.$parse_fn_name() {
				Some(v) => ethabi::Token::$token(v.into()),
				None => {
					log::debug!(
						"There is no value or it might be of different type, pointer: ${:?}",
						pointer
					);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};
			Ok(success_precompile_output(encoded))
		}
	};
}

#[macro_export]
macro_rules! http_post_precompile_fn {
	($name:ident, $token:ident, $parse_fn_name:ident) => {
		pub fn $name<T: SendHttpRequest>(
			input: Vec<u8>,
			client: T,
		) -> $crate::precompiles::PrecompileResult {
			let decoded = match ethabi::decode(
				&[
					ethabi::ParamType::String,
					ethabi::ParamType::String,
					ethabi::ParamType::String,
					ethabi::ParamType::Array(
						ethabi::ParamType::Tuple(vec![
							ethabi::ParamType::String,
							ethabi::ParamType::String,
						])
						.into(),
					),
				],
				&input,
			) {
				Ok(d) => d,
				Err(e) => {
					log::debug!("Could not decode bytes {:?}, reason: {:?}", input, e);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};
			let value: serde_json::Value =
				match $crate::precompiles::macros::do_post(client, &decoded, 0, 3, 2) {
					Ok(v) => v,
					Err(_) => {
						return Ok(failure_precompile_output(ethabi::Token::$token(
							Default::default(),
						)))
					},
				};

			// safe to unwrap
			let pointer = decoded.get(1).unwrap().clone().into_string().unwrap();
			let result = match value.pointer(&pointer) {
				Some(v) => v,
				None => {
					log::debug!("No value under given pointer: :{:?}", pointer);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};

			let encoded = match result.$parse_fn_name() {
				Some(v) => ethabi::Token::$token(v.into()),
				None => {
					log::debug!(
						"There is no value or it might be of different type, pointer: ${:?}",
						pointer
					);
					return Ok(failure_precompile_output(
						ethabi::Token::$token(Default::default()),
					));
				},
			};
			Ok(success_precompile_output(encoded))
		}
	};
}

pub fn extract_http_headers(
	decoded: &[Token],
	index: usize,
) -> std::vec::Vec<(std::string::String, std::string::String)> {
	decoded
		.get(index)
		.unwrap()
		.clone()
		.into_array()
		.unwrap()
		.iter()
		.map(|v| {
			let name =
				v.clone().into_tuple().unwrap().get(0).unwrap().clone().into_string().unwrap();
			let value =
				v.clone().into_tuple().unwrap().get(1).unwrap().clone().into_string().unwrap();

			(name, value)
		})
		.collect()
}

pub fn do_get<T: SendHttpRequest>(
	client: T,
	decoded: &[Token],
	url_index: usize,
	headers_index: usize,
) -> Result<serde_json::Value, ()> {
	let url = decoded.get(url_index).unwrap().clone().into_string().unwrap();
	let url = match itc_rest_client::rest_client::Url::parse(&url) {
		Ok(v) => v,
		Err(_) => return Err(()),
	};
	let http_headers: std::vec::Vec<(std::string::String, std::string::String)> =
		extract_http_headers(decoded, headers_index);
	let resp = match client.send_request_raw(
		url,
		itc_rest_client::rest_client::Method::GET,
		None,
		http_headers,
	) {
		Ok(resp) => resp,
		Err(e) => {
			log::debug!("Error while performing http call: {:?}", e);
			return Err(());
		},
	};
	match serde_json::from_slice(&resp.1) {
		Ok(v) => Ok(v),
		Err(e) => {
			log::debug!("Could not parse json {:?}, reason: {:?}", resp.1, e);
			Err(())
		},
	}
}

pub fn do_post<T: SendHttpRequest>(
	client: T,
	decoded: &[Token],
	url_index: usize,
	headers_index: usize,
	payload_index: usize,
) -> Result<serde_json::Value, ()> {
	let url = decoded.get(url_index).unwrap().clone().into_string().unwrap();
	let url = match itc_rest_client::rest_client::Url::parse(&url) {
		Ok(v) => v,
		Err(_) => return Err(()),
	};
	let payload = decoded.get(payload_index).unwrap().clone().into_string().unwrap();
	let http_headers: std::vec::Vec<(std::string::String, std::string::String)> =
		extract_http_headers(decoded, headers_index);
	let resp = match client.send_request_raw(
		url,
		itc_rest_client::rest_client::Method::POST,
		Some(payload),
		http_headers,
	) {
		Ok(resp) => resp,
		Err(e) => {
			log::debug!("Error while performing http call: {:?}", e);
			return Err(());
		},
	};
	match serde_json::from_slice(&resp.1) {
		Ok(v) => Ok(v),
		Err(e) => {
			log::debug!("Could not parse json {:?}, reason: {:?}", resp.1, e);
			Err(())
		},
	}
}
