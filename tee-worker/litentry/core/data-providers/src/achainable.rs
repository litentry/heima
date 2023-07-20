// Copyright 2020-2023 Litentry Technologies GmbH.
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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{build_client, Error, HttpError, GLOBAL_DATA_PROVIDER_CONFIG};
use http::header::{AUTHORIZATION, CONNECTION};
use http_req::response::Headers;
use itc_rest_client::{
	http_client::{DefaultSend, HttpClient},
	rest_client::RestClient,
	RestGet, RestPath, RestPost,
};
use litentry_primitives::Web3Network;
use log::{debug, error};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use std::{
	collections::HashMap,
	format, str,
	string::{String, ToString},
	vec,
	vec::Vec,
};
pub struct AchainableClient {
	client: RestClient<HttpClient<DefaultSend>>,
}

impl Default for AchainableClient {
	fn default() -> Self {
		Self::new()
	}
}

impl AchainableClient {
	pub fn new() -> Self {
		let mut headers = Headers::new();
		headers.insert(CONNECTION.as_str(), "close");
		headers.insert(
			AUTHORIZATION.as_str(),
			// GLOBAL_DATA_PROVIDER_CONFIG.read().unwrap().achainable_auth_key.clone().as_str(),
			"26353d4c-b01c-4466-98a5-80d3fc53a9d8",
		);
		let client = build_client(
			// GLOBAL_DATA_PROVIDER_CONFIG.read().unwrap().achainable_url.clone().as_str(),
			"https://label-production.graph.tdf-labs.io",
			headers,
		);

		AchainableClient { client }
	}
}


pub trait AchainablePost {
	fn post(&mut self, params: SystemLabelReqPath, body: &ReqBody) -> Result<serde_json::Value, Error>;
}

impl AchainablePost for AchainableClient {
	fn post(&mut self, params: SystemLabelReqPath, body: &ReqBody) -> Result<serde_json::Value, Error> {
		let response =
			self.client.post_capture::<SystemLabelReqPath, ReqBody, serde_json::Value>(params, body);
		debug!("ReqBody response: {:?}", response);
		match response {
			Ok(res) =>
				if let Some(value) = res.get("result") {
					Ok(value.clone())
				} else {
					Err(Error::AchainableError("Invalid response".to_string()))
				},
			Err(e) => Err(Error::RequestError(format!("{:?}", e))),
		}
	}
}

pub trait AchainableResultParser {
	type Item;
	fn parse(value: serde_json::Value) -> Result<Self::Item, Error>;
}

impl AchainableResultParser for AchainableClient {
	type Item = bool;
	fn parse(value: serde_json::Value) -> Result<Self::Item, Error> {
		if let Some(b) = value.as_bool() {
			Ok(b)
		} else {
			Err(Error::AchainableError("Invalid response".to_string()))
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemLabelReqPath {
	path: String,
}

impl Default for SystemLabelReqPath {
	fn default() -> Self {
		Self {
			path: "/v1/run/system-labels".into(),
		}
	}
}

impl SystemLabelReqPath {
	pub fn new(path: &str) -> Self {
		Self { path: path.into() }
	}
}

// #[derive(Debug)]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
	pub name: String,
	pub address: String,
	pub params: Params,
}

impl RestPath<SystemLabelReqPath> for ReqBody {
	fn get_path(req_params: SystemLabelReqPath) -> core::result::Result<String, HttpError> {
		Ok(req_params.path)
	}
}

impl ReqBody {
	pub fn new(address: String, params: Params) -> Self {
		ReqBody {
			name: params.name(),
			address,
			params
		}
	}
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Params {
	AmountHoding(AmountHoding),         //A4-A7-A10-A11
	TotalTransaction(TotalTransaction), //A8
	FreshAccount(FreshAccount),
	OgAccount(OgAccount),
	ClassOfYear(ClassOfYear),
	AddressFoundOnBsc(AddressFoundOnBsc),
	EthDrainedInLastFortnight(EthDrainedInLastFortnight),
	Validator(Validator),
}

impl AchainableSystemLabelName for Params {
	fn name(&self) -> String {
		match self {
			Params::AmountHoding(a) => a.name(),
			Params::TotalTransaction(t) => t.name(),
			Params::FreshAccount(i) => i.name(),
			Params::OgAccount(i) => i.name(),
			Params::ClassOfYear(c) => c.name(),
			Params::AddressFoundOnBsc(a) => a.name(),
			Params::EthDrainedInLastFortnight(e) => e.name(),
			Params::Validator(v) => v.name(),
		}
	}
}

/// The parameter types of the method are defined here
pub trait AchainableSystemLabelName {
	fn name(&self) -> String;
}

/// A4/A7/A10/A11 Holder params
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmountHoding {
	pub chain: String,
	pub amount: String,
	pub date: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub token: Option<String>,
}

impl AmountHoding {
	pub fn new(chain: String, amount: String, date: String, token: Option<String>) -> Self {
		Self {
			chain,
			amount,
			date,
			token,
		}
	}
}

impl AchainableSystemLabelName for AmountHoding {
	fn name(&self) -> String {
		if self.token.is_some() {
			"ERC20 hodling {amount} of {token} since {date}".into()
		} else {
			"Balance hodling {amount} since {date}".into()
		}
	}
}

/// A8 Total transaction params
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TotalTransaction {
	pub chain: String,
	pub amount: String,
}

impl TotalTransaction {
	fn new(chain: String) -> Self {
		Self {
			chain,
			amount: "1".into(), //TODO: Be 1 always.
		}
	}
}

impl AchainableSystemLabelName for TotalTransaction {
	fn name(&self) -> String {
		"Account total transactions under {amount}".into()
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FreshAccount {
	pub chain: String,
	pub date: String,
}

impl Default for FreshAccount {
	fn default() -> Self {
		Self {
			chain: "ethereum".into(),
			date: "30D".into(),
		}
	}
}

impl AchainableSystemLabelName for FreshAccount {
	fn name(&self) -> String {
		"Account created after {date}".into()
	}
}

// OgAccount
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OgAccount {
	pub chain: String,
	pub date: String,
}

impl Default for OgAccount {
	fn default() -> Self {
		Self {
			chain: "ethereum".into(),
			date: "2020-01-01T00:00:00.000Z".into(),
		}
	}
}

impl AchainableSystemLabelName for OgAccount {
	fn name(&self) -> String {
		"Account created before {date}".into()
	}
}

// ClassOfYear
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassOfYear {
	pub chain: String,
	pub date1: String,
	pub date2: String,
}

pub enum EClassOfYear {
	Year2020,
	Year2021,
	Year2022,
}

impl EClassOfYear {
	pub fn get(&self) -> ClassOfYear {
		match self {
			EClassOfYear::Year2020 => ClassOfYear::class_of_2020(),
			EClassOfYear::Year2021 => ClassOfYear::class_of_2021(),
			EClassOfYear::Year2022 => ClassOfYear::class_of_2022(),
		}
	}
}

impl ClassOfYear {
	fn class_of_2020() -> Self {
		Self {
			chain: "ethereum".into(),
			date1: "2020-01-01T00:00:00.000Z".into(),
			date2: "2020-12-31T23:59:59.999Z".into(),
		}
	}

	fn class_of_2021() -> Self {
		Self {
			chain: "ethereum".into(),
			date1: "2021-01-01T00:00:00.000Z".into(),
			date2: "2021-12-31T23:59:59.999Z".into(),
		}
	}

	fn class_of_2022() -> Self {
		Self {
			chain: "ethereum".into(),
			date1: "2022-01-01T00:00:00.000Z".into(),
			date2: "2022-12-31T23:59:59.999Z".into(),
		}
	}
}

impl AchainableSystemLabelName for ClassOfYear {
	fn name(&self) -> String {
		"Account created between {dates}".into()
	}
}

// AddressFoundOnBsc
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressFoundOnBsc {
	pub chain: String,
}

impl Default for AddressFoundOnBsc {
	fn default() -> Self {
		Self {
			chain: "bsc".into(),
		}
	}
}

impl AchainableSystemLabelName for AddressFoundOnBsc {
	fn name(&self) -> String {
		"Account found on {chain}".into()
	}
}

// ETH Drained in Last Fortnight
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EthDrainedInLastFortnight {
	pub chain: String,
	pub token: String,
	pub date: String,
	pub percent: String,
}

impl Default for EthDrainedInLastFortnight {
	fn default() -> Self {
		Self {
			chain: "ethereum".into(),
			token: "ETH".into(),
			date: "14D".into(),
			percent: "80".into(),
		}
	}
}

impl AchainableSystemLabelName for EthDrainedInLastFortnight {
	fn name(&self) -> String {
		"Balance dropped {percent} since {date}".into()
	}
}

// Validator
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
	pub chain: String,
}

impl Validator {
	pub fn new(chain: String) -> Self {
		Self {
			chain
		}
	}
}

impl AchainableSystemLabelName for Validator {
	fn name(&self) -> String {
		"Validator".into()
	}
}

fn check_achainable_label(
	client: &mut AchainableClient,
	address: String,
	params: Params,
) -> Result<bool, Error> {
	let req_path = SystemLabelReqPath::default();
	let body = ReqBody::new(address, params);
	debug!("x>>> body: {:?}", body);
	let resp = client.post(req_path, &body)?;
	AchainableClient::parse(resp)
}

/// A4/A7/A10/A11
/// 
pub trait AchainableHolder {
	fn is_holder(&mut self, address: &str, amount_holding: AmountHoding) -> Result<bool, Error>;
}

impl AchainableHolder for AchainableClient {
	fn is_holder(&mut self, address: &str, amount_holding: AmountHoding) -> Result<bool, Error> {
		check_achainable_label(self, address.into(), Params::AmountHoding(amount_holding))
	}
}

/// A8 TODO:
/// TODO:
/// This is a compromise. We need to judge the range of the sum of transactions of all linked accounts, but the achanable api
/// currently only judges the range of a single account, so the current approach is to parse the returned data through
/// an assertion such as under 1 to calculate the sum, and then perform interval judgment.
pub trait AchainableTotalTransactionsParser {
	fn parse_txs(response: serde_json::Value) -> Result<u64, Error>;
}
impl AchainableTotalTransactionsParser for AchainableClient {
	fn parse_txs(response: serde_json::Value) -> Result<u64, Error> {
		let display_text = response
			.get("label")
			.and_then(|value| {
				value.get("display").and_then(|displays| {
					displays.as_array().map(|displays| {
						let mut text: std::option::Option<String> = None;
						for v in displays.iter() {
							text = v
								.get("text")
								.and_then(|text| {
									text.as_str().map(|display_text| Some(display_text.to_string()))
								})
								.flatten();
						}
						text
					})
				})
			})
			.flatten();

		debug!("Total txs, display text: {:?}", display_text);

		if let Some(display_text) = display_text {
			// TODO:
			// text field format: Total transactions under 1 (Transactions: 0)
			let split_text = display_text.split(": ").collect::<Vec<&str>>();
			if split_text.len() != 2 {
				return Err(Error::AchainableError("Invalid array".to_string()))
			}

			let mut value_text = split_text[1].to_string();

			// pop the last char: ")"
			value_text.pop();

			let value: u64 = value_text.parse::<u64>().unwrap_or_default();

			return Ok(value)
		}

		Err(Error::AchainableError("Invalid response".to_string()))
	}
}
pub trait AchainableAccountTotalTransactions {
	/// NOTE: Achinable "chain" fieild must be one of [ethereum, polkadot, kusama, litmus, litentry, khala]
	fn total_transactions(&mut self, network: &Web3Network, addresses: &[String]) -> Result<u64, Error>;
}

impl AchainableAccountTotalTransactions for AchainableClient {
	fn total_transactions(&mut self, network: &Web3Network, addresses: &[String]) -> Result<u64, Error> {
		let mut txs = 0_u64;
		addresses.iter().for_each(|address| {
			let chain = network.to_string();		
			let param = TotalTransaction::new(chain);
			let body = ReqBody::new(address.into(), Params::TotalTransaction(param));
			let req_path = SystemLabelReqPath::default();
	
			let tx = self.post(req_path, &body).and_then(|resp| {
				Self::parse_txs(resp)
			});
			txs += tx.unwrap_or_default();
		});

		Ok(txs)
	}
}

pub trait AchainableTagAccount {
	fn fresh_account(&mut self, address: &str) -> Result<bool, Error>;
	fn og_account(&mut self, address: &str) -> Result<bool, Error>;
	fn class_of_year(&mut self, address: &str, year: EClassOfYear) -> Result<bool, Error>;
	fn address_found_on_bsc(&mut self, address: &str) -> Result<bool, Error>;
	fn eth_drained_in_last_fortnight(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_validator(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_validator(&mut self, address: &str) -> Result<bool, Error>;
}

impl AchainableTagAccount for AchainableClient {
	fn fresh_account(&mut self, address: &str) -> Result<bool, Error> {
		let param = FreshAccount::default();
		check_achainable_label(self, address.into(), Params::FreshAccount(param))
	}

	fn og_account(&mut self, address: &str) -> Result<bool, Error> {
		let param = OgAccount::default();
		check_achainable_label(self, address.into(), Params::OgAccount(param))
	}

	fn class_of_year(&mut self, address: &str, year: EClassOfYear) -> Result<bool, Error> {
		let param = year.get();
		check_achainable_label(self, address.into(), Params::ClassOfYear(param))
	}
	
	fn address_found_on_bsc(&mut self, address: &str) -> Result<bool, Error> {
		let param = AddressFoundOnBsc::default();
		check_achainable_label(self, address.into(), Params::AddressFoundOnBsc(param))
	}

	fn eth_drained_in_last_fortnight(&mut self, address: &str) -> Result<bool, Error> {
		let param = EthDrainedInLastFortnight::default();
		check_achainable_label(self, address.into(), Params::EthDrainedInLastFortnight(param))
	}

	fn is_polkadot_validator(&mut self, address: &str) -> Result<bool, Error> {
		let param = Validator::new("polkadot".into());
		check_achainable_label(self, address.into(), Params::Validator(param))
	}
	
	fn is_kusama_validator(&mut self, address: &str) -> Result<bool, Error> {
		let param = Validator::new("kusama".into());
		check_achainable_label(self, address.into(), Params::Validator(param))
	}
}



///////////////////////////////////////////////
#[cfg(test)]
mod tests {
	use crate::achainable::{
		AchainableClient, AchainableTagAccount, AchainableTagBalance, AchainableTagDeFi,
		AchainableTagDotsama, AchainableTotalTransactions, GLOBAL_DATA_PROVIDER_CONFIG,
	};
	use lc_mock_server::{default_getter, run};
	use litentry_primitives::Web3Network;
	use std::sync::Arc;

	fn init() {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(Arc::new(default_getter), 0).unwrap();
		GLOBAL_DATA_PROVIDER_CONFIG.write().unwrap().set_achainable_url(url);
	}

	#[test]
	fn total_transactions_work() {
		init();

		let addresses = vec!["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5".to_string()];

		let mut client = AchainableClient::new();
		let r = client.total_transactions(&Web3Network::Litentry, &addresses);
		assert!(r.is_ok());
		let r = r.unwrap();
		assert!(r == 41)
	}

	#[test]
	fn fresh_account_works() {
		init();

		let mut client = AchainableClient::new();
		let res: Result<bool, crate::Error> = client.fresh_account("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn og_account_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.og_account("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn class_of_2020_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.class_of_2020("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn class_of_2021_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.class_of_2021("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn class_of_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.class_of_2022("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn found_on_bsc_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.found_on_bsc("0x3f349bBaFEc1551819B8be1EfEA2fC46cA749aA1");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_validator_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_polkadot_validator("17bR6rzVsVrzVJS1hM4dSJU43z2MUmz7ZDpPLh8y2fqVg7m");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_validator_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_kusama_validator("ESRBbWstgpPV1pVBsqjMo717rA8HLrtQvEUVwAGeFZyKcia");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn polkadot_dolphin_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.polkadot_dolphin("1soESeTVLfse9e2G8dRSMUyJ2SWad33qhtkjQTv9GMToRvP");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn kusama_dolphin_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.kusama_dolphin("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn polkadot_whale_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.polkadot_whale("1soESeTVLfse9e2G8dRSMUyJ2SWad33qhtkjQTv9GMToRvP");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn kusama_whale_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.kusama_whale("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn less_than_10_eth_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.less_than_10_eth_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn less_than_10_lit_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.less_than_10_lit_holder("0x2A038e100F8B85DF21e4d44121bdBfE0c288A869");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn not_less_than_100_eth_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.not_less_than_100_eth_holder("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn between_10_to_100_eth_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.between_10_to_100_eth_holder("0x082aB5505CdeA46caeF670754E962830Aa49ED2C");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn eth_millionaire_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.eth_millionaire("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn eth2_validator_eligible_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.eth2_validator_eligible("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn not_less_than_100_weth_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.not_less_than_100_weth_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn not_less_than_100_lit_bep20_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.not_less_than_100_lit_bep20_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn native_lit_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.native_lit_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn erc20_lit_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.erc20_lit_holder("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn bep20_lit_holder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.bep20_lit_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_treasury_proposal_beneficiary_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_polkadot_treasury_proposal_beneficiary(
			"5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW",
		);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_treasury_proposal_beneficiary_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_kusama_treasury_proposal_beneficiary(
			"CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq",
		);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_tip_finder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_polkadot_tip_finder("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_tip_finder_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_kusama_tip_finder("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_tip_beneficiary_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_polkadot_tip_beneficiary("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_tip_beneficiary_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_kusama_tip_beneficiary("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_opengov_proposer_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_polkadot_opengov_proposer("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_opengov_proposer_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_kusama_opengov_proposer("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_fellowship_proposer_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client
			.is_polkadot_fellowship_proposer("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_fellowship_proposer_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_kusama_fellowship_proposer("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_fellowship_member_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client
			.is_polkadot_fellowship_member("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_fellowship_member_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_kusama_fellowship_member("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_ex_councilor_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_polkadot_ex_councilor("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_ex_councilor_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_kusama_ex_councilor("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_councilor_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_polkadot_councilor("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_councilor_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.is_kusama_councilor("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_polkadot_bounty_curator_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_polkadot_bounty_curator("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn is_kusama_bounty_curator_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.is_kusama_bounty_curator("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn uniswap_v2_user_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.uniswap_v2_user("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v3_user_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.uniswap_v3_user("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v2_lp_in_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.uniswap_v2_lp_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn uniswap_v3_lp_in_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.uniswap_v3_lp_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdc_uniswap_v2_lp_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.usdc_uniswap_v2_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn usdc_uniswap_v3_lp_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.usdc_uniswap_v3_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdt_uniswap_lp_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.usdt_uniswap_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn usdt_uniswap_v2_lp_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.usdt_uniswap_v2_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn usdt_uniswap_v3_lp_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.usdt_uniswap_v3_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn aave_v2_lender_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.aave_v2_lender("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn aave_v2_borrower_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.aave_v2_borrower("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn aave_v3_lender_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.aave_v3_lender("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn aave_v3_borrower_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.aave_v3_borrower("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn curve_trader_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.curve_trader("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn curve_trader_in_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.curve_trader_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn curve_liquidity_provider_works() {
		init();

		let mut client = AchainableClient::new();
		let res = client.curve_liquidity_provider("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn curve_liquidity_provider_in_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.curve_liquidity_provider_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, false);
	}

	#[test]
	fn swapped_with_metamask_in_2022_works() {
		init();

		let mut client = AchainableClient::new();
		let res =
			client.swapped_with_metamask_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}
}
