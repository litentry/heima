use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DepositAddress {
	pub address: String,
	pub coin: String,
	pub tag: String,
	pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
	pub coin: String,
	pub deposit_all_enable: bool,
	pub free: String,
	pub freeze: String,
	pub ipoable: String,
	pub ipoing: String,
	pub is_legal_money: bool,
	pub locked: String,
	pub name: String,
	pub network_list: Vec<NetworkInfo>,
	pub storage: Option<String>,
	pub trading: Option<bool>,
	pub withdraw_all_enable: Option<bool>,
	pub withdrawing: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
	pub address_regex: String,
	pub coin: String,
	// Optional fields that are only present under certain conditions
	pub deposit_desc: Option<String>,
	pub deposit_enable: bool,
	pub is_default: bool,
	pub memo_regex: String,
	pub min_confirm: u32,
	pub name: String,
	pub network: String,
	pub special_tips: String,
	pub special_withdraw_tips: Option<String>,
	pub un_lock_confirm: u32,
	pub withdraw_desc: Option<String>,
	pub withdraw_enable: bool,
	pub withdraw_fee: String,
	pub withdraw_integer_multiple: String,
	pub withdraw_max: String,
	pub withdraw_min: String,
	pub withdraw_internal_min: Option<String>,
	pub same_address: bool,
	pub estimated_arrival_time: u32,
	pub busy: bool,
	pub contract_address_url: String,
	pub contract_address: String,
	pub reset_address_status: Option<bool>,
	pub deposit_dust: Option<String>,
	pub denomination: Option<u64>,
}
