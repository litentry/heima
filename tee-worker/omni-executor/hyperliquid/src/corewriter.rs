use alloy::primitives::Address;
use executor_primitives::utils::hex::ToHexPrefixed;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_cloid() -> u128 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

pub fn encode_limit_order_action(
	asset: u32,
	is_buy: bool,
	limit_px: u64,
	sz: u64,
	cloid: u128,
) -> Vec<u8> {
	let reduce_only = false;
	let encoded_tif: u8 = 3; // Ioc

	let encoded = ethabi::encode(&[
		ethabi::Token::Uint(asset.into()),
		ethabi::Token::Bool(is_buy),
		ethabi::Token::Uint(limit_px.into()),
		ethabi::Token::Uint(sz.into()),
		ethabi::Token::Bool(reduce_only),
		ethabi::Token::Uint(encoded_tif.into()),
		ethabi::Token::Uint(cloid.into()),
	]);

	let mut data = Vec::new();
	data.push(0x01); // version
	data.extend_from_slice(&[0x00, 0x00, 0x01]); // action_id = 1
	data.extend_from_slice(&encoded);

	data
}

pub fn encode_omni_account_execute(target: Address, call_data: Vec<u8>) -> String {
	const EXECUTE_SELECTOR: [u8; 4] = [0xb6, 0x1d, 0x27, 0xf6];

	let target_h160 = ethabi::ethereum_types::H160::from_slice(target.as_slice());

	let encoded = ethabi::encode(&[
		ethabi::Token::Address(target_h160),
		ethabi::Token::Uint(0.into()),
		ethabi::Token::Bytes(call_data),
	]);

	let mut result = Vec::new();
	result.extend_from_slice(&EXECUTE_SELECTOR);
	result.extend_from_slice(&encoded);

	result.to_hex()
}

pub fn encode_send_raw_action(action_data: Vec<u8>) -> Vec<u8> {
	const SEND_RAW_ACTION_SELECTOR: [u8; 4] = [0x17, 0x93, 0x8e, 0x13];

	let encoded = ethabi::encode(&[ethabi::Token::Bytes(action_data)]);

	let mut result = Vec::new();
	result.extend_from_slice(&SEND_RAW_ACTION_SELECTOR);
	result.extend_from_slice(&encoded);

	result
}

pub fn build_spot_sell_order(asset_id: u32, size: u64, cloid: u128) -> Vec<u8> {
	let is_buy = false;
	let aggressive_price = 1u64;
	encode_limit_order_action(asset_id, is_buy, aggressive_price, size, cloid)
}

pub fn build_perp_long_order(asset_id: u32, size: u64, cloid: u128) -> Vec<u8> {
	let is_buy = true;
	let aggressive_price = u64::MAX;
	encode_limit_order_action(asset_id, is_buy, aggressive_price, size, cloid)
}

pub fn calculate_size_units(amount: f64) -> u64 {
	(amount * 1e8) as u64
}

pub fn encode_usd_class_transfer_action(ntl: u64, to_perp: bool) -> Vec<u8> {
	let encoded = ethabi::encode(&[ethabi::Token::Uint(ntl.into()), ethabi::Token::Bool(to_perp)]);

	let mut data = Vec::new();
	data.push(0x01); // version
	data.extend_from_slice(&[0x00, 0x00, 0x07]); // action_id = 7
	data.extend_from_slice(&encoded);

	data
}

pub fn build_usd_class_transfer_to_perp(ntl: u64) -> Vec<u8> {
	encode_usd_class_transfer_action(ntl, true)
}
