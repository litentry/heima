use crate::{
	failure_precompile_output, json_get_fn,
	precompiles::{
		logging::{contract_logging, LOGGING_LEVEL_WARN},
		PrecompileResult,
	},
	success_precompile_output, Precompiles,
};
use ethabi::Token;
use serde_json::Value;
use std::vec::Vec;

json_get_fn!(json_get_string, String, as_str);
json_get_fn!(json_get_bool, Bool, as_bool);
json_get_fn!(json_get_i64, Uint, as_i64);

pub fn get_array_len(input: Vec<u8>, precompiles: &Precompiles) -> PrecompileResult {
	let decoded =
		match ethabi::decode(&[ethabi::ParamType::String, ethabi::ParamType::String], &input) {
			Ok(d) => d,
			Err(e) => {
				let message = std::format!("Could not decode bytes {:?}, reason: {:?}", input, e);
				log::debug!("{}", message);
				contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
				return Ok(failure_precompile_output(ethabi::Token::String(Default::default())));
			},
		};

	let json = decoded.get(0).unwrap().clone().into_string().unwrap();
	let pointer = decoded.get(1).unwrap().clone().into_string().unwrap();

	let value: Value = match serde_json::from_str(&json) {
		Ok(v) => v,
		Err(e) => {
			let message = std::format!("Could not parse json {:?}, reason: {:?}", json, e);
			log::debug!("{}", message);
			contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
			return Ok(failure_precompile_output(Token::Int(Default::default())));
		},
	};

	let result = match value.pointer(&pointer) {
		Some(v) => match v.as_array() {
			Some(arr) => arr.len(),
			None => {
				let message = std::format!(
					"There is no value or it might be of different type, pointer: ${:?}",
					pointer
				);
				log::debug!("{}", message);
				contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
				return Ok(failure_precompile_output(Token::Int(Default::default())));
			},
		},
		None => {
			let message = std::format!("No value under given pointer: :{:?}", pointer);
			log::debug!("{}", message);
			contract_logging(precompiles, LOGGING_LEVEL_WARN, message);
			return Ok(failure_precompile_output(Token::Int(Default::default())));
		},
	};

	Ok(success_precompile_output(Token::Int(result.into())))
}

#[cfg(test)]
pub mod test {
	use crate::{
		precompiles::json_utils::{get_array_len, json_get_bool, json_get_i64, json_get_string},
		success_precompile_output, Precompiles,
	};
	use ethabi::{encode, Token};
	use serde_json::json;

	#[test]
	pub fn test_get_string() {
		// given:
		let json = json!(
			{
				"key": "value"
			}
		);
		let data = prepare_input_data(&json.to_string(), "/key");

		// when:
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = json_get_string(data, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::String("value".into())), result)
	}

	#[test]
	pub fn test_get_i64() {
		// given:
		let json = json!(
			{
				"key": 14
			}
		);
		let data = prepare_input_data(&json.to_string(), "/key");

		// when:
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = json_get_i64(data, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Int(14.into())), result)
	}

	#[test]
	pub fn test_get_bool() {
		// given:
		let json = json!(
			{
				"key": true
			}
		);
		let data = prepare_input_data(&json.to_string(), "/key");

		// when:
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = json_get_bool(data, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Bool(true)), result)
	}

	#[test]
	pub fn test_get_array_len_with_array() {
		// given:
		let json = json!([{}, {}, {}]);
		let data = prepare_input_data(&json.to_string(), "");

		// when:
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = get_array_len(data, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Int(3.into())), result)
	}

	#[test]
	pub fn test_get_array_len_with_nested_array() {
		// given:
		let json = json!({
		   "nested": [
				{},
				{},
				{}
			]
		});
		let data = prepare_input_data(&json.to_string(), "/nested");

		// when:
		let precompiles = Precompiles { contract_logs: Vec::new().into() };
		let result = get_array_len(data, &precompiles).unwrap();

		// then
		assert_eq!(success_precompile_output(Token::Int(3.into())), result)
	}

	fn prepare_input_data(url: &str, pointer: &str) -> Vec<u8> {
		encode(&[Token::String(url.to_string()), Token::String(pointer.to_string())])
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};
	use serde_json::json;

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/JsonTest.sol
	const FUNCTION_HASH_0: &str = "73260cf2"; // callGetString(string,string)
	const FUNCTION_HASH_1: &str = "fe598591"; // callGetBool(string,string)
	const FUNCTION_HASH_2: &str = "f5e19bc0"; // callGetI64(string,string)
	const FUNCTION_HASH_3: &str = "c43558dc"; // callGetArrayLen(string,string)
	const BYTE_CODE: &str = "608060405234801561001057600080fd5b50610466806100206000396000f3fe608060405234801561001057600080fd5b506004361061004c5760003560e01c806373260cf214610051578063c43558dc1461007b578063f5e19bc0146100a8578063fe598591146100bb575b600080fd5b61006461005f36600461032e565b6100e5565b6040516100729291906103df565b60405180910390f35b61008e61008936600461032e565b6100fe565b60408051921515835260079190910b602083015201610072565b61008e6100b636600461032e565b61010b565b6100ce6100c936600461032e565b610118565b604080519215158352901515602083015201610072565b600060606100f38484610125565b915091509250929050565b6000806100f38484610190565b6000806100f384846101f7565b6000806100f38484610241565b600060606000606060008686604051602001610142929190610402565b60408051601f1981840301815290829052805190925090611000818360208601600061044c600019f161017457600080fd5b805161100082016040908152909a910198509650505050505050565b600080600080600086866040516020016101ab929190610402565b60408051601f1981840301815282825280519093509190818360208601600061044f600019f16101da57600080fd5b805160208201516040928301909252999098509650505050505050565b60008060008060008686604051602001610212929190610402565b60408051601f1981840301815282825280519093509190818360208601600061044d600019f16101da57600080fd5b6000806000806000868660405160200161025c929190610402565b60408051601f1981840301815282825280519093509190818360208601600061044e600019f16101da57600080fd5b634e487b7160e01b600052604160045260246000fd5b600082601f8301126102b257600080fd5b813567ffffffffffffffff808211156102cd576102cd61028b565b604051601f8301601f19908116603f011681019082821181831017156102f5576102f561028b565b8160405283815286602085880101111561030e57600080fd5b836020870160208301376000602085830101528094505050505092915050565b6000806040838503121561034157600080fd5b823567ffffffffffffffff8082111561035957600080fd5b610365868387016102a1565b9350602085013591508082111561037b57600080fd5b50610388858286016102a1565b9150509250929050565b6000815180845260005b818110156103b85760208185018101518683018201520161039c565b818111156103ca576000602083870101525b50601f01601f19169290920160200192915050565b82151581526040602082015260006103fa6040830184610392565b949350505050565b6040815260006104156040830185610392565b82810360208401526104278185610392565b9594505050505056fea2646970667358221220805f7414e00fe67d1d12b44977aa199685e393cf50b59d5ed0a402a9f95e8a2564736f6c634300080b0033";

	#[test]
	pub fn test_get_string() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::String];
		let json = serde_json::to_string(&json!({
			"nested": {
				"object": {
					"property": "value"
				}
			}
		}))
		.unwrap();

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH_0,
			encode(&[Token::String(json), Token::String("/nested/object/property".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!("value", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_get_bool() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Bool];
		let json = serde_json::to_string(&json!({
			"nested": {
				"object": {
					"property": true
				}
			}
		}))
		.unwrap();

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH_1,
			encode(&[Token::String(json), Token::String("/nested/object/property".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(true, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_get_i64() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Int(64)];
		let json = serde_json::to_string(&json!({
			"nested": {
				"object": {
					"property": 14
				}
			}
		}))
		.unwrap();

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH_2,
			encode(&[Token::String(json), Token::String("/nested/object/property".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(14), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_get_get_array_len() {
		let byte_code = hex::decode(BYTE_CODE).unwrap();
		let return_types = vec![ParamType::Bool, ParamType::Int(64)];
		let json = serde_json::to_string(&json!({
			"nested": {
				"object": {
					"property": [
						{},
						{},
						{}
					]
				}
			}
		}))
		.unwrap();

		// given
		let input_data = prepare_function_call_input(
			FUNCTION_HASH_3,
			encode(&[Token::String(json), Token::String("/nested/object/property".to_string())]),
		)
		.unwrap();

		// when
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);

		// then
		let decoded = decode(&return_types, &return_data).unwrap();
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(3), decoded[1].clone().into_int().unwrap());
	}
}
