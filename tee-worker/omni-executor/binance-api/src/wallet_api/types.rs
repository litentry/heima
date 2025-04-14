use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
	pub id: String,
	pub amount: String,
	pub coin: String,
	pub network: String,
	pub status: u8,
	pub address: String,
	pub address_tag: Option<String>,
	pub tx_id: Option<String>,
	pub insert_time: u64,
	pub complete_time: Option<u64>,
	pub transfer_type: u8,
	pub confirm_times: String,
	pub unlock_confirm: u8,
	pub wallet_type: u8,
	pub source_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawOrder {
	pub id: String,
}
