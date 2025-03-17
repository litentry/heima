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
	precompiles::{macros::do_get, PrecompileResult},
	*,
};
use itc_rest_client::http_client::SendHttpRequest;

http_get_precompile_fn!(http_get_bool, Bool, as_bool);
http_get_precompile_fn!(http_get_i64, Uint, as_i64);
http_get_precompile_fn!(http_get_string, String, as_str);

pub fn http_get<T: SendHttpRequest>(input: Vec<u8>, client: T) -> PrecompileResult {
	let decoded = match decode(
		&[
			ParamType::String,
			ParamType::Array(ParamType::Tuple(vec![ParamType::String, ParamType::String]).into()),
		],
		&input,
	) {
		Ok(d) => d,
		Err(e) => {
			log::debug!("Could not decode bytes {:?}, reason: {:?}", input, e);
			return Ok(failure_precompile_output(Token::String(Default::default())))
		},
	};
	let value: serde_json::Value = match do_get(client, &decoded, 0, 1) {
		Ok(v) => v,
		Err(_) => return Ok(failure_precompile_output(Token::String(Default::default()))),
	};
	Ok(success_precompile_output(Token::String(value.to_string())))
}

#[cfg(test)]
pub mod test {
	use crate::{
		failure_precompile_output,
		precompiles::{
			http_get::{http_get, http_get_bool, http_get_i64, http_get_string},
			mocks::MockedHttpClient,
		},
		success_precompile_output,
	};
	use ethabi::{encode, ethereum_types::U256, Token};
	use serde_json::json;

	#[test]
	pub fn test_get_bool() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/bool");

		// when
		let result = http_get_bool(data, client).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Bool(true)), result)
	}

	#[test]
	pub fn test_get_i64() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/i64");

		// when
		let result = http_get_i64(data, client).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Uint(U256::try_from(10).unwrap())), result)
	}

	#[test]
	pub fn test_get_string() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/string");

		// when
		let result = http_get_string(data, client).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::String("string".to_string())), result)
	}

	#[test]
	pub fn test_get() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_raw_input_data("https://www.heima.network/");

		// when
		let result = http_get(data, client).unwrap();

		// then
		assert_eq!(
			success_precompile_output(Token::String(
				serde_json::to_string(&json!({
					"bool": true,
					"i64": 10,
					"string": "string",
					"not_bool": 10
				}))
				.unwrap()
			)),
			result
		)
	}

	#[test]
	pub fn returns_failure_for_invalid_url() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("invalid_url", "/string");

		// when
		let result = http_get_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_invalid_json_pointer() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "invalid_pointer");

		// when
		let result = http_get_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_malformed_json() {
		// given
		let client = MockedHttpClient::malformed_json();
		let data = prepare_input_data("https://www.heima.network/", "string");

		// when
		let result = http_get_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_value_of_type_other_than_expected() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/not_bool");

		// when
		let result = http_get_bool(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::Bool(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_invalid_input_data() {
		// given
		let client = MockedHttpClient::default();
		let data = [0u8, 11];

		// when
		let result = http_get_bool(data.to_vec(), client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::Bool(Default::default())), result)
	}

	#[test]
	pub fn returns_error_for_http_error() {
		// given
		let client = MockedHttpClient::http_error();
		let data = prepare_input_data("https://www.heima.network/", "string");

		// when
		let result = http_get_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(Token::String(Default::default())), result)
	}

	fn prepare_input_data(url: &str, pointer: &str) -> Vec<u8> {
		encode(&[
			Token::String(url.to_string()),
			Token::String(pointer.to_string()),
			Token::Array(vec![]),
		])
	}

	fn prepare_raw_input_data(url: &str) -> Vec<u8> {
		encode(&[Token::String(url.to_string()), Token::Array(vec![])])
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};
	use lc_mock_server::run;
	use serde_json::json;

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/GetI64.sol
	const GET_I64_FUNCTION_HASH_0: &str = "f5e19bc0"; // callGetI64(string,string)
	const GET_I64_FUNCTION_HASH_1: &str = "ed043e0f"; // callGetI64TwiceAndReturnSecondResult(string,string,string,string)
	const GET_I64_BYTE_CODE: &str = "608060405234801561001057600080fd5b50610783806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063ed043e0f1461003b578063f5e19bc01461006c575b600080fd5b6100556004803603810190610050919061037a565b61009d565b604051610063929190610488565b60405180910390f35b610086600480360381019061008191906104b1565b61011e565b604051610094929190610488565b60405180910390f35b60008060008067ffffffffffffffff8111156100bc576100bb61024f565b5b6040519080825280602002602001820160405280156100f557816020015b6100e2610206565b8152602001906001900390816100da5790505b509050610103878783610190565b5050610110858583610190565b925092505094509492505050565b60008060008067ffffffffffffffff81111561013d5761013c61024f565b5b60405190808252806020026020018201604052801561017657816020015b610163610206565b81526020019060019003908161015b5790505b509050610184858583610190565b92509250509250929050565b60008060008060008787876040516020016101ad93929190610701565b6040516020818303038152906040529050600081519050604051604081836020860160006103e8600019f16101e157600080fd5b8051945060208101519350604081016040525083839550955050505050935093915050565b604051806040016040528060608152602001606081525090565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6102878261023e565b810181811067ffffffffffffffff821117156102a6576102a561024f565b5b80604052505050565b60006102b9610220565b90506102c5828261027e565b919050565b600067ffffffffffffffff8211156102e5576102e461024f565b5b6102ee8261023e565b9050602081019050919050565b82818337600083830152505050565b600061031d610318846102ca565b6102af565b90508281526020810184848401111561033957610338610239565b5b6103448482856102fb565b509392505050565b600082601f83011261036157610360610234565b5b813561037184826020860161030a565b91505092915050565b600080600080608085870312156103945761039361022a565b5b600085013567ffffffffffffffff8111156103b2576103b161022f565b5b6103be8782880161034c565b945050602085013567ffffffffffffffff8111156103df576103de61022f565b5b6103eb8782880161034c565b935050604085013567ffffffffffffffff81111561040c5761040b61022f565b5b6104188782880161034c565b925050606085013567ffffffffffffffff8111156104395761043861022f565b5b6104458782880161034c565b91505092959194509250565b60008115159050919050565b61046681610451565b82525050565b60008160070b9050919050565b6104828161046c565b82525050565b600060408201905061049d600083018561045d565b6104aa6020830184610479565b9392505050565b600080604083850312156104c8576104c761022a565b5b600083013567ffffffffffffffff8111156104e6576104e561022f565b5b6104f28582860161034c565b925050602083013567ffffffffffffffff8111156105135761051261022f565b5b61051f8582860161034c565b9150509250929050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610563578082015181840152602081019050610548565b83811115610572576000848401525b50505050565b600061058382610529565b61058d8185610534565b935061059d818560208601610545565b6105a68161023e565b840191505092915050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600082825260208201905092915050565b60006105f982610529565b61060381856105dd565b9350610613818560208601610545565b61061c8161023e565b840191505092915050565b6000604083016000830151848203600086015261064482826105ee565b9150506020830151848203602086015261065e82826105ee565b9150508091505092915050565b60006106778383610627565b905092915050565b6000602082019050919050565b6000610697826105b1565b6106a181856105bc565b9350836020820285016106b3856105cd565b8060005b858110156106ef57848403895281516106d0858261066b565b94506106db8361067f565b925060208a019950506001810190506106b7565b50829750879550505050505092915050565b6000606082019050818103600083015261071b8186610578565b9050818103602083015261072f8185610578565b90508181036040830152610743818461068c565b905094935050505056fea26469706673582212207c6995442fefa9ad2c5b108cffcea33d88d6360353d695bdc6669f09b1fb3cbb64736f6c634300080b0033";
	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/GetBool.sol
	const GET_BOOL_FUNCTION_HASH_0: &str = "fe598591"; // callGetBool(string,string)
	const GET_BOOL_FUNCTION_HASH_1: &str = "7083d8ec"; // callGetBoolTwiceAndReturnSecondResult(string,string,string,string)
	const GET_BOOL_BYTE_CODE: &str = "608060405234801561001057600080fd5b50610767806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80637083d8ec1461003b578063fe5985911461006c575b600080fd5b6100556004803603810190610050919061037a565b61009d565b60405161006392919061046c565b60405180910390f35b61008660048036038101906100819190610495565b61011e565b60405161009492919061046c565b60405180910390f35b60008060008067ffffffffffffffff8111156100bc576100bb61024f565b5b6040519080825280602002602001820160405280156100f557816020015b6100e2610206565b8152602001906001900390816100da5790505b509050610103878783610190565b5050610110858583610190565b925092505094509492505050565b60008060008067ffffffffffffffff81111561013d5761013c61024f565b5b60405190808252806020026020018201604052801561017657816020015b610163610206565b81526020019060019003908161015b5790505b509050610184858583610190565b92509250509250929050565b60008060008060008787876040516020016101ad939291906106e5565b6040516020818303038152906040529050600081519050604051604081836020860160006103e9600019f16101e157600080fd5b8051945060208101519350604081016040525083839550955050505050935093915050565b604051806040016040528060608152602001606081525090565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6102878261023e565b810181811067ffffffffffffffff821117156102a6576102a561024f565b5b80604052505050565b60006102b9610220565b90506102c5828261027e565b919050565b600067ffffffffffffffff8211156102e5576102e461024f565b5b6102ee8261023e565b9050602081019050919050565b82818337600083830152505050565b600061031d610318846102ca565b6102af565b90508281526020810184848401111561033957610338610239565b5b6103448482856102fb565b509392505050565b600082601f83011261036157610360610234565b5b813561037184826020860161030a565b91505092915050565b600080600080608085870312156103945761039361022a565b5b600085013567ffffffffffffffff8111156103b2576103b161022f565b5b6103be8782880161034c565b945050602085013567ffffffffffffffff8111156103df576103de61022f565b5b6103eb8782880161034c565b935050604085013567ffffffffffffffff81111561040c5761040b61022f565b5b6104188782880161034c565b925050606085013567ffffffffffffffff8111156104395761043861022f565b5b6104458782880161034c565b91505092959194509250565b60008115159050919050565b61046681610451565b82525050565b6000604082019050610481600083018561045d565b61048e602083018461045d565b9392505050565b600080604083850312156104ac576104ab61022a565b5b600083013567ffffffffffffffff8111156104ca576104c961022f565b5b6104d68582860161034c565b925050602083013567ffffffffffffffff8111156104f7576104f661022f565b5b6105038582860161034c565b9150509250929050565b600081519050919050565b600082825260208201905092915050565b60005b8381101561054757808201518184015260208101905061052c565b83811115610556576000848401525b50505050565b60006105678261050d565b6105718185610518565b9350610581818560208601610529565b61058a8161023e565b840191505092915050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600082825260208201905092915050565b60006105dd8261050d565b6105e781856105c1565b93506105f7818560208601610529565b6106008161023e565b840191505092915050565b6000604083016000830151848203600086015261062882826105d2565b9150506020830151848203602086015261064282826105d2565b9150508091505092915050565b600061065b838361060b565b905092915050565b6000602082019050919050565b600061067b82610595565b61068581856105a0565b935083602082028501610697856105b1565b8060005b858110156106d357848403895281516106b4858261064f565b94506106bf83610663565b925060208a0199505060018101905061069b565b50829750879550505050505092915050565b600060608201905081810360008301526106ff818661055c565b90508181036020830152610713818561055c565b905081810360408301526107278184610670565b905094935050505056fea264697066735822122029e9d00dcee526443754516f9675d482d1df8c85c193c0ea11236c4163ca496f64736f6c634300080b0033";
	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/GetString.sol
	const GET_STRING_FUNCTION_HASH_0: &str = "73260cf2"; // callGetString(string,string)
	const GET_STRING_FUNCTION_HASH_1: &str = "4069716b"; // callGetStringTwiceAndReturnSecondResult(string,string,string,string)
	const GET_STRING_BYTE_CODE: &str = "608060405234801561001057600080fd5b506107e9806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80634069716b1461003b57806373260cf21461006c575b600080fd5b610055600480360381019061005091906103f5565b61009d565b60405161006392919061056f565b60405180910390f35b6100866004803603810190610081919061059f565b61011f565b60405161009492919061056f565b60405180910390f35b6000606060008067ffffffffffffffff8111156100bd576100bc6102ca565b5b6040519080825280602002602001820160405280156100f657816020015b6100e3610281565b8152602001906001900390816100db5790505b509050610104878783610192565b5050610111858583610208565b925092505094509492505050565b6000606060008067ffffffffffffffff81111561013f5761013e6102ca565b5b60405190808252806020026020018201604052801561017857816020015b610165610281565b81526020019060019003908161015d5790505b509050610186858583610208565b92509250509250929050565b60008060008060008787876040516020016101af93929190610767565b6040516020818303038152906040529050600081519050604051604081836020860160006103e8600019f16101e357600080fd5b8051945060208101519350604081016040525083839550955050505050935093915050565b6000606060006060600087878760405160200161022793929190610767565b604051602081830303815290604052905060008151905060405161100081836020860160006103ea600019f161025c57600080fd5b8051945060408101935061100081016040525083839550955050505050935093915050565b604051806040016040528060608152602001606081525090565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b610302826102b9565b810181811067ffffffffffffffff82111715610321576103206102ca565b5b80604052505050565b600061033461029b565b905061034082826102f9565b919050565b600067ffffffffffffffff8211156103605761035f6102ca565b5b610369826102b9565b9050602081019050919050565b82818337600083830152505050565b600061039861039384610345565b61032a565b9050828152602081018484840111156103b4576103b36102b4565b5b6103bf848285610376565b509392505050565b600082601f8301126103dc576103db6102af565b5b81356103ec848260208601610385565b91505092915050565b6000806000806080858703121561040f5761040e6102a5565b5b600085013567ffffffffffffffff81111561042d5761042c6102aa565b5b610439878288016103c7565b945050602085013567ffffffffffffffff81111561045a576104596102aa565b5b610466878288016103c7565b935050604085013567ffffffffffffffff811115610487576104866102aa565b5b610493878288016103c7565b925050606085013567ffffffffffffffff8111156104b4576104b36102aa565b5b6104c0878288016103c7565b91505092959194509250565b60008115159050919050565b6104e1816104cc565b82525050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610521578082015181840152602081019050610506565b83811115610530576000848401525b50505050565b6000610541826104e7565b61054b81856104f2565b935061055b818560208601610503565b610564816102b9565b840191505092915050565b600060408201905061058460008301856104d8565b81810360208301526105968184610536565b90509392505050565b600080604083850312156105b6576105b56102a5565b5b600083013567ffffffffffffffff8111156105d4576105d36102aa565b5b6105e0858286016103c7565b925050602083013567ffffffffffffffff811115610601576106006102aa565b5b61060d858286016103c7565b9150509250929050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600082825260208201905092915050565b600061065f826104e7565b6106698185610643565b9350610679818560208601610503565b610682816102b9565b840191505092915050565b600060408301600083015184820360008601526106aa8282610654565b915050602083015184820360208601526106c48282610654565b9150508091505092915050565b60006106dd838361068d565b905092915050565b6000602082019050919050565b60006106fd82610617565b6107078185610622565b93508360208202850161071985610633565b8060005b85811015610755578484038952815161073685826106d1565b9450610741836106e5565b925060208a0199505060018101905061071d565b50829750879550505050505092915050565b600060608201905081810360008301526107818186610536565b905081810360208301526107958185610536565b905081810360408301526107a981846106f2565b905094935050505056fea264697066735822122068d3d33b54fcdaec5e45509b8b96565a9f8246b11e88a0b6037d878ac7dd6f2a64736f6c634300080b0033";
	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/Get.sol
	const GET_FUNCTION_HASH_0: &str = "5294dd5d"; // callGet(string)
	const GET_FUNCTION_HASH_1: &str = "7e2b1ce8"; // callGetTwiceAndReturnSecondResult(string,string)
	const GET_BYTE_CODE: &str = "608060405234801561001057600080fd5b5061041a806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80635294dd5d1461003b5780637e2b1ce814610065575b600080fd5b61004e610049366004610246565b610078565b60405161005c9291906102d0565b60405180910390f35b61004e6100733660046102eb565b6100d1565b604080516000808252602082019092526060908290816100ba565b60408051808201909152606080825260208201528152602001906001900390816100935790505b5090506100c78482610138565b9250925050915091565b60408051600080825260208201909252606090829081610113565b60408051808201909152606080825260208201528152602001906001900390816100ec5790505b5090506101208582610138565b505061012c8482610138565b92509250509250929050565b60006060600060606000868660405160200161015592919061034f565b60408051601f198184030181529082905280519092509061100081836020860160006103ee600019f161018757600080fd5b805161100082016040908152909a910198509650505050505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126101ca57600080fd5b813567ffffffffffffffff808211156101e5576101e56101a3565b604051601f8301601f19908116603f0116810190828211818310171561020d5761020d6101a3565b8160405283815286602085880101111561022657600080fd5b836020870160208301376000602085830101528094505050505092915050565b60006020828403121561025857600080fd5b813567ffffffffffffffff81111561026f57600080fd5b61027b848285016101b9565b949350505050565b6000815180845260005b818110156102a95760208185018101518683018201520161028d565b818111156102bb576000602083870101525b50601f01601f19169290920160200192915050565b821515815260406020820152600061027b6040830184610283565b600080604083850312156102fe57600080fd5b823567ffffffffffffffff8082111561031657600080fd5b610322868387016101b9565b9350602085013591508082111561033857600080fd5b50610345858286016101b9565b9150509250929050565b6000604080835261036281840186610283565b6020848203818601528186518084528284019150828160051b85010183890160005b838110156103d457868303601f19018552815180518985526103a88a860182610283565b91880151858303868a01529190506103c08183610283565b968801969450505090850190600101610384565b50909a995050505050505050505056fea264697066735822122035dbbcf3e25e708cb011c7fe03f8b7ab6899cdaffb90b289835803b82fc4940764736f6c634300080b0033";

	#[test]
	pub fn test_get_i64() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_I64_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Int(2)];

		// given
		let input_data = prepare_function_call_input(
			GET_I64_FUNCTION_HASH_0,
			encode(&[
				Token::String(format!(
					"{}/2/users/by/username/twitterdev?user.fields=public_metrics",
					url
				)),
				Token::String("/data/public_metrics/followers_count".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(100), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_get_i64_fail() {
		let byte_code = hex::decode(GET_I64_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Int(2)];

		// given
		let input_data = prepare_function_call_input(
			GET_I64_FUNCTION_HASH_0,
			encode(&[
				Token::String(
					"http://localhost:1/2/users/by/username/twitterdev?user.fields=public_metrics"
						.into(),
				),
				Token::String("/data/public_metrics/followers_count".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(0), decoded[1].clone().into_int().unwrap());
	}

	// we want to check here that execution is not interrupted by http error
	#[test]
	pub fn test_get_i64_returns_second_result_in_case_of_first_request_failure() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_I64_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Int(2)];

		// given
		let input_data = prepare_function_call_input(
			GET_I64_FUNCTION_HASH_1,
			encode(&[
				// this one uses different port so service is unavailable
				Token::String(
					"http://localhost:1/2/users/by/username/twitterdev?user.fields=public_metrics"
						.to_string(),
				),
				Token::String("/data/public_metrics/followers_count".to_string()),
				Token::String(format!(
					"{}/2/users/by/username/twitterdev?user.fields=public_metrics",
					url
				)),
				Token::String("/data/public_metrics/followers_count".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(100), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_get_bool() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_BOOL_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Bool];

		// given
		let input_data = prepare_function_call_input(GET_BOOL_FUNCTION_HASH_0, encode(&[
			Token::String(format!("{}/events/does-user-joined-evm-campaign?account=0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", url)),
			Token::String("/hasJoined".to_string())])).unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(true, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_get_bool_fail() {
		let byte_code = hex::decode(GET_BOOL_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Bool];

		// given
		let input_data = prepare_function_call_input(GET_BOOL_FUNCTION_HASH_0, encode(&[
			Token::String("http://localhost:1/events/does-user-joined-evm-campaign?account=0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".into()),
			 Token::String("/hasJoined".to_string())])).unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!(false, decoded[1].clone().into_bool().unwrap());
	}

	// we want to check here that execution is not interrupted by http error
	#[test]
	pub fn test_get_bool_returns_second_result_in_case_of_first_request_failure() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_BOOL_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Bool];

		// given
		let input_data = prepare_function_call_input(
			GET_BOOL_FUNCTION_HASH_1,
				encode(
				&[
					// this one uses different port so service is unavailable
					Token::String("http://localhost:1/events/does-user-joined-evm-campaign?account=0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string()),
					Token::String("/hasJoined".to_string()),
					Token::String(format!("{}/events/does-user-joined-evm-campaign?account=0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", url)),
					Token::String("/hasJoined".to_string())
				]
			)
		).unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(true, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_get_string() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_STRING_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_STRING_FUNCTION_HASH_0,
			encode(&[
				Token::String(format!(
					"{}/v1/blocks/e4068e6a326243468f35dcdc0c43f686/children",
					url
				)),
				Token::String("/object".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!("list", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_get_string_fail() {
		let byte_code = hex::decode(GET_STRING_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_STRING_FUNCTION_HASH_0,
			encode(&[
				Token::String(
					"http://localhost:1/v1/blocks/e4068e6a326243468f35dcdc0c43f686/children".into(),
				),
				Token::String("/object".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!("", decoded[1].clone().into_string().unwrap());
	}

	// we want to check here that execution is not interrupted by http error
	#[test]
	pub fn test_get_string_returns_second_result_in_case_of_first_request_failure() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_STRING_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_STRING_FUNCTION_HASH_1,
			encode(&[
				// this one uses different port so service is unavailable
				Token::String(
					"http://localhost:1/v1/blocks/e4068e6a326243468f35dcdc0c43f686/children".into(),
				),
				Token::String("/object".to_string()),
				Token::String(format!(
					"{}/v1/blocks/e4068e6a326243468f35dcdc0c43f686/children",
					url
				)),
				Token::String("/object".to_string()),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!("list", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_get() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_FUNCTION_HASH_0,
			encode(&[Token::String(format!("{}/blockchain_info/rawaddr/addr", url))]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			serde_json::to_string(&json!({
				"final_balance": 0
			}))
			.unwrap(),
			decoded[1].clone().into_string().unwrap()
		);
	}

	#[test]
	pub fn test_get_fail() {
		let byte_code = hex::decode(GET_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_FUNCTION_HASH_0,
			encode(&[Token::String("http://localhost:1/blockchain_info/rawaddr/addr".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!("", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_get_returns_second_result_in_case_of_first_request_failure() {
		let url = run(0).unwrap();
		let byte_code = hex::decode(GET_BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];

		// given
		let input_data = prepare_function_call_input(
			GET_FUNCTION_HASH_1,
			encode(&[
				// this one uses different port so service is unavailable
				Token::String("http://localhost:1/blockchain_info/rawaddr/addr".to_string()),
				Token::String(format!("{}/blockchain_info/rawaddr/addr", url)),
			]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			serde_json::to_string(&json!({
				"final_balance": 0
			}))
			.unwrap(),
			decoded[1].clone().into_string().unwrap()
		);
	}
}
