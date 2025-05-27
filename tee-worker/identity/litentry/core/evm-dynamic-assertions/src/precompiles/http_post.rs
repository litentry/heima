use crate::{
	precompiles::{macros::do_post, PrecompileResult},
	*,
};
use itc_rest_client::http_client::SendHttpRequest;

http_post_precompile_fn!(http_post_bool, Bool, as_bool);
http_post_precompile_fn!(http_post_i64, Uint, as_i64);
http_post_precompile_fn!(http_post_string, String, as_str);

pub fn http_post<T: SendHttpRequest>(input: Vec<u8>, client: T) -> PrecompileResult {
	let decoded = match decode(
		&[
			ParamType::String,
			ParamType::String,
			ParamType::Array(ParamType::Tuple(vec![ParamType::String, ParamType::String]).into()),
		],
		&input,
	) {
		Ok(d) => d,
		Err(e) => {
			log::debug!("Could not decode bytes {:?}, reason: {:?}", input, e);
			return Ok(failure_precompile_output(Token::String(Default::default())));
		},
	};
	let value: serde_json::Value = match do_post(client, &decoded, 0, 2, 1) {
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
			http_post::{http_post_bool, http_post_i64, http_post_string},
			mocks::MockedHttpClient,
		},
		success_precompile_output,
	};
	use ethabi::ethereum_types::U256;
	use evm::ExitSucceed;

	#[test]
	pub fn test_post_bool() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/bool", "{}");

		// when
		let result = http_post_bool(data, client).unwrap();

		// then
		assert_eq!(success_precompile_output(ethabi::Token::Bool(true)), result)
	}

	#[test]
	pub fn test_post_i64() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/i64", "{}");

		// when
		let result = http_post_i64(data, client).unwrap();

		// then
		assert_eq!(
			success_precompile_output(ethabi::Token::Uint(U256::try_from(10).unwrap())),
			result
		)
	}

	#[test]
	pub fn test_post_string() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/string", "{}");

		// when
		let result = http_post_string(data, client).unwrap();

		// then
		assert_eq!(success_precompile_output(ethabi::Token::String("string".to_string())), result)
	}

	#[test]
	pub fn returns_failure_for_invalid_url() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("invalid_url", "/string", "{}");

		// when
		let result = http_post_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(ethabi::Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_invalid_json_pointer() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "invalid_pointer", "{}");

		// when
		let result = http_post_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(ethabi::Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_malformed_json() {
		// given
		let client = MockedHttpClient::malformed_json();
		let data = prepare_input_data("https://www.heima.network/", "string", "{}");

		// when
		let result = http_post_string(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(ethabi::Token::String(Default::default())), result)
	}

	#[test]
	pub fn returns_failure_for_value_of_type_other_than_expected() {
		// given
		let client = MockedHttpClient::default();
		let data = prepare_input_data("https://www.heima.network/", "/not_bool", "{}");

		// when
		let result = http_post_bool(data, client).unwrap();

		// then
		assert_eq!(failure_precompile_output(ethabi::Token::Bool(false)), result)
	}

	#[test]
	pub fn returns_failure_for_invalid_input_data() {
		// given
		let client = MockedHttpClient::default();
		let data = [0u8, 11];

		// when
		let result = http_post_bool(data.to_vec(), client).unwrap();

		// then
		assert!(matches!(result.exit_status, ExitSucceed::Returned));
		assert_eq!(failure_precompile_output(ethabi::Token::Bool(false)), result)
	}

	#[test]
	pub fn returns_error_for_http_error() {
		// given
		let client = MockedHttpClient::http_error();
		let data = prepare_input_data("https://www.heima.network/", "string", "{}");

		// when
		let result = http_post_string(data, client).unwrap();

		// then
		assert!(matches!(result.exit_status, ExitSucceed::Returned));
		assert_eq!(failure_precompile_output(ethabi::Token::String(Default::default())), result)
	}

	fn prepare_input_data(url: &str, pointer: &str, payload: &str) -> Vec<u8> {
		ethabi::encode(&[
			ethabi::Token::String(url.to_string()),
			ethabi::Token::String(pointer.to_string()),
			ethabi::Token::String(payload.to_string()),
			ethabi::Token::Array(vec![]),
		])
	}
}

#[cfg(test)]
pub mod integration_test {
	use crate::{execute_smart_contract, prepare_function_call_input};
	use ethabi::{encode, ethereum_types::U256, Token};
	use lc_mock_server::run;
	use serde_json::Value;

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/PostI64.sol
	const POST_I64_FUNCTION_HASH_0: &str = "2381daad"; // callPostI64(string,string,string)
	const POST_I64_FUNCTION_HASH_1: &str = "18f0d608"; // callPostI64TwiceAndReturnSecondResult(string,string,string,string,string,string)
	const POST_I64_BYTE_CODE: &str = "608060405234801561001057600080fd5b5061051a806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806318f0d6081461003b5780632381daad1461006c575b600080fd5b61004e61004936600461025a565b61007f565b60408051921515835260079190910b60208301520160405180910390f35b61004e61007a36600461034f565b6100ed565b6040805160008082526020820190925281908190816100c0565b60408051808201909152606080825260208201528152602001906001900390816100995790505b5090506100cf8989898461014a565b50506100dd8686868461014a565b9250925050965096945050505050565b60408051600080825260208201909252819081908161012e565b60408051808201909152606080825260208201528152602001906001900390816101075790505b50905061013d8686868461014a565b9250925050935093915050565b6000806000806000888888886040516020016101699493929190610424565b60408051601f198184030181528282528051909350919081836020860160006103eb600019f161019857600080fd5b8051602082015160409283019092529b909a5098505050505050505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126101de57600080fd5b813567ffffffffffffffff808211156101f9576101f96101b7565b604051601f8301601f19908116603f01168101908282118183101715610221576102216101b7565b8160405283815286602085880101111561023a57600080fd5b836020870160208301376000602085830101528094505050505092915050565b60008060008060008060c0878903121561027357600080fd5b863567ffffffffffffffff8082111561028b57600080fd5b6102978a838b016101cd565b975060208901359150808211156102ad57600080fd5b6102b98a838b016101cd565b965060408901359150808211156102cf57600080fd5b6102db8a838b016101cd565b955060608901359150808211156102f157600080fd5b6102fd8a838b016101cd565b9450608089013591508082111561031357600080fd5b61031f8a838b016101cd565b935060a089013591508082111561033557600080fd5b5061034289828a016101cd565b9150509295509295509295565b60008060006060848603121561036457600080fd5b833567ffffffffffffffff8082111561037c57600080fd5b610388878388016101cd565b9450602086013591508082111561039e57600080fd5b6103aa878388016101cd565b935060408601359150808211156103c057600080fd5b506103cd868287016101cd565b9150509250925092565b6000815180845260005b818110156103fd576020818501810151868301820152016103e1565b8181111561040f576000602083870101525b50601f01601f19169290920160200192915050565b60808152600061043760808301876103d7565b60208382038185015261044a82886103d7565b915060408483038186015261045f83886103d7565b925084830360608601528286518085528385019150838160051b86010184890160005b838110156104d257878303601f19018552815180518785526104a6888601826103d7565b91890151858303868b01529190506104be81836103d7565b968901969450505090860190600101610482565b50909c9b50505050505050505050505056fea264697066735822122049f0442b4788f10091894993479dbd5aa31707e778e37c370888cdb77c24445264736f6c634300080b0033";

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/PostBool.sol
	const POST_BOOL_FUNCTION_HASH_0: &str = "9c428231"; // callPostBool(string,string,string)
	const POST_BOOL_FUNCTION_HASH_1: &str = "c668f937"; // callPostBoolTwiceAndReturnSecondResult(string,string,string,string,string,string)
	const POST_BOOL_BYTE_CODE: &str = "608060405234801561001057600080fd5b50610517806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80639c4282311461003b578063c668f93714610069575b600080fd5b61004e610049366004610257565b61007c565b60408051921515835290151560208301520160405180910390f35b61004e6100773660046102df565b6100d9565b6040805160008082526020820190925281908190816100bd565b60408051808201909152606080825260208201528152602001906001900390816100965790505b5090506100cc86868684610147565b9250925050935093915050565b60408051600080825260208201909252819081908161011a565b60408051808201909152606080825260208201528152602001906001900390816100f35790505b50905061012989898984610147565b505061013786868684610147565b9250925050965096945050505050565b6000806000806000888888886040516020016101669493929190610421565b60408051601f198184030181528282528051909350919081836020860160006103ec600019f161019557600080fd5b8051602082015160409283019092529b909a5098505050505050505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126101db57600080fd5b813567ffffffffffffffff808211156101f6576101f66101b4565b604051601f8301601f19908116603f0116810190828211818310171561021e5761021e6101b4565b8160405283815286602085880101111561023757600080fd5b836020870160208301376000602085830101528094505050505092915050565b60008060006060848603121561026c57600080fd5b833567ffffffffffffffff8082111561028457600080fd5b610290878388016101ca565b945060208601359150808211156102a657600080fd5b6102b2878388016101ca565b935060408601359150808211156102c857600080fd5b506102d5868287016101ca565b9150509250925092565b60008060008060008060c087890312156102f857600080fd5b863567ffffffffffffffff8082111561031057600080fd5b61031c8a838b016101ca565b9750602089013591508082111561033257600080fd5b61033e8a838b016101ca565b9650604089013591508082111561035457600080fd5b6103608a838b016101ca565b9550606089013591508082111561037657600080fd5b6103828a838b016101ca565b9450608089013591508082111561039857600080fd5b6103a48a838b016101ca565b935060a08901359150808211156103ba57600080fd5b506103c789828a016101ca565b9150509295509295509295565b6000815180845260005b818110156103fa576020818501810151868301820152016103de565b8181111561040c576000602083870101525b50601f01601f19169290920160200192915050565b60808152600061043460808301876103d4565b60208382038185015261044782886103d4565b915060408483038186015261045c83886103d4565b925084830360608601528286518085528385019150838160051b86010184890160005b838110156104cf57878303601f19018552815180518785526104a3888601826103d4565b91890151858303868b01529190506104bb81836103d4565b96890196945050509086019060010161047f565b50909c9b50505050505050505050505056fea2646970667358221220e2f5fd57d4e1af1e82babd9ad9c76106e329c8efcff87511ffe94203235eadc664736f6c634300080b0033";

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/PostString.sol
	const POST_STRING_FUNCTION_HASH_0: &str = "a7481684"; // callPostString(string,string,string)
	const POST_STRING_FUNCTION_HASH_1: &str = "1ad1cfaf"; // callPostStringTwiceAndReturnSecondResult(string,string,string,string,string,string)
	const POST_STRING_BYTE_CODE: &str = "608060405234801561001057600080fd5b5061053c806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80631ad1cfaf1461003b578063a748168414610065575b600080fd5b61004e610049366004610259565b610078565b60405161005c92919061039b565b60405180910390f35b61004e6100733660046103be565b6100e7565b604080516000808252602082019092526060908290816100ba565b60408051808201909152606080825260208201528152602001906001900390816100935790505b5090506100c989898984610145565b50506100d786868684610145565b9250925050965096945050505050565b60408051600080825260208201909252606090829081610129565b60408051808201909152606080825260208201528152602001906001900390816101025790505b50905061013886868684610145565b9250925050935093915050565b60006060600060606000888888886040516020016101669493929190610446565b60408051601f198184030181529082905280519092509061100081836020860160006103ed600019f161019857600080fd5b805161100082016040908152909c91019a5098505050505050505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126101dd57600080fd5b813567ffffffffffffffff808211156101f8576101f86101b6565b604051601f8301601f19908116603f01168101908282118183101715610220576102206101b6565b8160405283815286602085880101111561023957600080fd5b836020870160208301376000602085830101528094505050505092915050565b60008060008060008060c0878903121561027257600080fd5b863567ffffffffffffffff8082111561028a57600080fd5b6102968a838b016101cc565b975060208901359150808211156102ac57600080fd5b6102b88a838b016101cc565b965060408901359150808211156102ce57600080fd5b6102da8a838b016101cc565b955060608901359150808211156102f057600080fd5b6102fc8a838b016101cc565b9450608089013591508082111561031257600080fd5b61031e8a838b016101cc565b935060a089013591508082111561033457600080fd5b5061034189828a016101cc565b9150509295509295509295565b6000815180845260005b8181101561037457602081850181015186830182015201610358565b81811115610386576000602083870101525b50601f01601f19169290920160200192915050565b82151581526040602082015260006103b6604083018461034e565b949350505050565b6000806000606084860312156103d357600080fd5b833567ffffffffffffffff808211156103eb57600080fd5b6103f7878388016101cc565b9450602086013591508082111561040d57600080fd5b610419878388016101cc565b9350604086013591508082111561042f57600080fd5b5061043c868287016101cc565b9150509250925092565b608081526000610459608083018761034e565b60208382038185015261046c828861034e565b9150604084830381860152610481838861034e565b925084830360608601528286518085528385019150838160051b86010184890160005b838110156104f457878303601f19018552815180518785526104c88886018261034e565b91890151858303868b01529190506104e0818361034e565b9689019694505050908601906001016104a4565b50909c9b50505050505050505050505056fea2646970667358221220d9f558cc5e728c0489649cd34bf73d7e31d7c2e6bdc209e25925d9e3326e36db64736f6c634300080b0033";

	// https://github.com/litentry/DynamicAssertion/tree/dev/contracts/tests/Post.sol
	const POST_FUNCTION_HASH_0: &str = "55695c96"; // callPost(string,string)
	const POST_FUNCTION_HASH_1: &str = "b1eeadd3"; // callPostTwiceAndReturnSecondResult(string,string,string,string)
	const POST_BYTE_CODE: &str = "608060405234801561001057600080fd5b506104b2806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806355695c961461003b578063b1eeadd314610065575b600080fd5b61004e610049366004610250565b610078565b60405161005c929190610301565b60405180910390f35b61004e610073366004610324565b6100d4565b604080516000808252602082019092526060908290816100ba565b60408051808201909152606080825260208201528152602001906001900390816100935790505b5090506100c885858361013f565b92509250509250929050565b60408051600080825260208201909252606090829081610116565b60408051808201909152606080825260208201528152602001906001900390816100ef5790505b50905061012487878361013f565b505061013185858361013f565b925092505094509492505050565b6000606060006060600087878760405160200161015e939291906103d1565b60408051601f198184030181529082905280519092509061100081836020860160006103ef600019f161019057600080fd5b805161100082016040908152909b91019950975050505050505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126101d457600080fd5b813567ffffffffffffffff808211156101ef576101ef6101ad565b604051601f8301601f19908116603f01168101908282118183101715610217576102176101ad565b8160405283815286602085880101111561023057600080fd5b836020870160208301376000602085830101528094505050505092915050565b6000806040838503121561026357600080fd5b823567ffffffffffffffff8082111561027b57600080fd5b610287868387016101c3565b9350602085013591508082111561029d57600080fd5b506102aa858286016101c3565b9150509250929050565b6000815180845260005b818110156102da576020818501810151868301820152016102be565b818111156102ec576000602083870101525b50601f01601f19169290920160200192915050565b821515815260406020820152600061031c60408301846102b4565b949350505050565b6000806000806080858703121561033a57600080fd5b843567ffffffffffffffff8082111561035257600080fd5b61035e888389016101c3565b9550602087013591508082111561037457600080fd5b610380888389016101c3565b9450604087013591508082111561039657600080fd5b6103a2888389016101c3565b935060608701359150808211156103b857600080fd5b506103c5878288016101c3565b91505092959194509250565b6060815260006103e460608301866102b4565b6020838203818501526103f782876102b4565b91506040848303818601528286518085528385019150838160051b86010184890160005b8381101561046b57878303601f190185528151805187855261043f888601826102b4565b91890151858303868b015291905061045781836102b4565b96890196945050509086019060010161041b565b50909b9a505050505050505050505056fea26469706673582212201203d4ae591e5878e3e24497b87e82e27fb2f5dbe11c4665edfbe1ef8b3bdbcb64736f6c634300080b0033";

	#[test]
	pub fn test_post_i64() {
		run(19534).unwrap();

		// given
		let byte_code = hex::decode(POST_I64_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_I64_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:19534/v1/run/system-labels".to_string()),
															 Token::String("/runningCost".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Int(2)];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(1), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_post_i64_with_failure() {
		// given
		let byte_code = hex::decode(POST_I64_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_I64_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/runningCost".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Int(2)];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(0), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_post_i64_returns_second_result_in_case_of_first_request_failure() {
		run(19535).unwrap();

		// given
		let byte_code = hex::decode(POST_I64_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_I64_FUNCTION_HASH_1,
													 encode(
														 &[
															 // this one uses different port so service is unavailable
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/runningCost".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string()),
															 Token::String("http://localhost:19535/v1/run/system-labels".to_string()),
															 Token::String("/runningCost".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ]
													 )
		).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Int(2)];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(U256::from(1), decoded[1].clone().into_int().unwrap());
	}

	#[test]
	pub fn test_post_bool() {
		run(19536).unwrap();

		// given
		let byte_code = hex::decode(POST_BOOL_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_BOOL_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:19536/v1/run/system-labels".to_string()),
															 Token::String("/result".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Bool];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(true, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_post_bool_with_failure() {
		// given
		let byte_code = hex::decode(POST_BOOL_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_BOOL_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/result".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Bool];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!(false, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_post_bool_returns_second_result_in_case_of_first_request_failure() {
		run(19537).unwrap();

		// given
		let byte_code = hex::decode(POST_BOOL_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_BOOL_FUNCTION_HASH_1,
													 encode(
														 &[
															 // this one uses different port so service is unavailable
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/result".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string()),
															 Token::String("http://localhost:19537/v1/run/system-labels".to_string()),
															 Token::String("/result".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ]
													 )
		).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::Bool];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(true, decoded[1].clone().into_bool().unwrap());
	}

	#[test]
	pub fn test_post_string() {
		run(19538).unwrap();

		// given
		let byte_code = hex::decode(POST_STRING_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_STRING_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:19538/v1/run/system-labels".to_string()),
															 Token::String("/display/0/text".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			"Total transactions under 1 (Transactions: 41)",
			decoded[1].clone().into_string().unwrap()
		);
	}

	#[test]
	pub fn test_post_string_with_failure() {
		// given
		let byte_code = hex::decode(POST_STRING_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_STRING_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/display/0/text".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!("", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_post_string_returns_second_result_in_case_of_first_request_failure() {
		run(19539).unwrap();

		// given
		let byte_code = hex::decode(POST_STRING_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_STRING_FUNCTION_HASH_1,
													 encode(
														 &[
															 // this one uses different port so service is unavailable
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String("/display/0/text".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string()),
															 Token::String("http://localhost:19539/v1/run/system-labels".to_string()),
															 Token::String("/display/0/text".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ]
													 )
		).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());
		assert_eq!(
			"Total transactions under 1 (Transactions: 41)",
			decoded[1].clone().into_string().unwrap()
		);
	}

	#[test]
	pub fn test_post() {
		// given
		let url = run(0).unwrap();
		let byte_code = hex::decode(POST_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String(format!("{}/v1/run/system-labels", url)),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];
		let expected_value: Value = serde_json::from_str("{\"result\":true,\"display\":[{\"text\":\"Total transactions under 1 (Transactions: 41)\",\"result\":true}],\"runningCost\":1}").unwrap();

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());

		let actual_value: Value =
			serde_json::from_str(&decoded[1].clone().into_string().unwrap()).unwrap();

		assert_eq!(expected_value, actual_value);
	}

	#[test]
	pub fn test_post_fail() {
		// given
		let byte_code = hex::decode(POST_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_FUNCTION_HASH_0,
													 encode(
														 &[
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ])).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(false, decoded[0].clone().into_bool().unwrap());
		assert_eq!("", decoded[1].clone().into_string().unwrap());
	}

	#[test]
	pub fn test_post_returns_second_result_in_case_of_first_request_failure() {
		// given
		let url = run(0).unwrap();
		let byte_code = hex::decode(POST_BYTE_CODE).unwrap();
		let input_data = prepare_function_call_input(POST_FUNCTION_HASH_1,
													 encode(
														 &[
															 // this one uses different port so service is unavailable
															 Token::String("http://localhost:1/v1/run/system-labels".to_string()),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string()),
															 Token::String(format!("{}/v1/run/system-labels", url)),
															 Token::String(r#"{"name": "Account total transactions under {amount}", "address": "test", "params": {"chain": "ChainName"}, "includeMetadata": false }"#.to_string())
														 ]
													 )
		).unwrap();
		let (_, return_data, _) = execute_smart_contract(byte_code, input_data);
		let types = vec![ethabi::ParamType::Bool, ethabi::ParamType::String];
		let expected_value: Value = serde_json::from_str("{\"result\":true,\"display\":[{\"text\":\"Total transactions under 1 (Transactions: 41)\",\"result\":true}],\"runningCost\":1}").unwrap();

		// when
		let decoded = ethabi::decode(&types, &return_data).unwrap();

		// then
		assert_eq!(true, decoded[0].clone().into_bool().unwrap());

		let actual_value: Value =
			serde_json::from_str(&decoded[1].clone().into_string().unwrap()).unwrap();

		assert_eq!(expected_value, actual_value);
	}
}
