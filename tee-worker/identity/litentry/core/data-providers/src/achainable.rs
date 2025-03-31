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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{
	build_client_with_cert, ConvertParameterString, DataProviderConfig, Error, HttpError, ReqPath,
	LIT_TOKEN_ADDRESS, USDC_TOKEN_ADDRESS, USDT_TOKEN_ADDRESS, WETH_TOKEN_ADDRESS,
};
use http::header::{AUTHORIZATION, CONNECTION};
use http_req::response::Headers;
use itc_rest_client::{
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::RestClient,
	RestPath, RestPost,
};
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use litentry_primitives::{AchainableParams, VCMPError, Web3Network};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
	format, str,
	string::{String, ToString},
	vec::Vec,
};

pub struct AchainableClient {
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
}

impl AchainableClient {
	pub fn new(data_provider_config: &DataProviderConfig) -> Self {
		let mut headers = Headers::new();
		headers.insert(CONNECTION.as_str(), "close");
		headers.insert(
			AUTHORIZATION.as_str(),
			data_provider_config.achainable_auth_key.clone().as_str(),
		);
		let client =
			build_client_with_cert(data_provider_config.achainable_url.clone().as_str(), headers);

		AchainableClient { client }
	}

	pub fn query_system_label(&mut self, address: &str, params: Params) -> Result<bool, Error> {
		// TODO: double-check it with Zhouhui if we don't ever need metadata here
		let body = ReqBody::new_with_false_metadata(address.into(), params);
		self.post(SystemLabelReqPath::default(), &body)
			.and_then(AchainableClient::parse)
	}

	fn parse_class_of_year(value: serde_json::Value) -> Result<String, Error> {
		let v = value
			.get("metadata")
			.and_then(|res| res.as_array())
			.and_then(|v| v.get(0))
			.and_then(|v| v.as_str());

		Ok(v.and_then(|v| v.get(0..4)).unwrap_or("Invalid").into())
	}

	pub fn query_class_of_year(&mut self, address: &str, params: Params) -> Result<String, Error> {
		let body = ReqBody::new(address.into(), params);
		self.post(SystemLabelReqPath::default(), &body)
			.and_then(Self::parse_class_of_year)
	}
}

pub trait AchainablePost {
	fn post(
		&mut self,
		params: SystemLabelReqPath,
		body: &ReqBody,
	) -> Result<serde_json::Value, Error>;
}

impl AchainablePost for AchainableClient {
	fn post(
		&mut self,
		params: SystemLabelReqPath,
		body: &ReqBody,
	) -> Result<serde_json::Value, Error> {
		let response = self
			.client
			.post_capture::<SystemLabelReqPath, ReqBody, serde_json::Value>(params, body);
		match response {
			Ok(r) => {
				debug!("ReqBody response: {}", r);
				Ok(r)
			},
			Err(e) => Err(Error::AchainableError(format!("Achainable response error: {}", e))),
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
		value
			.get("result")
			.and_then(|res| res.as_bool())
			.ok_or_else(|| Error::AchainableError("Achainable Parse result error".to_string()))
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemLabelReqPath {
	path: String,
}

impl Default for SystemLabelReqPath {
	fn default() -> Self {
		Self { path: "/v1/run/system-labels".into() }
	}
}

impl SystemLabelReqPath {
	pub fn new(path: &str) -> Self {
		Self { path: path.into() }
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
	pub name: String,
	pub address: String,
	pub params: Params,

	/// TODO:
	/// Because some interfaces of the achainable API cannot meet the current assertion requirements well, this trade-off is being made.
	/// This field is added here to request once interface to obtain the specific account creation date, and then match it with the product's time interval.
	/// And according to TDF developers, this field is unstable and may be cancelled in the future. Even so, this is currently the most appropriate approach
	/// So, this is the current solution.
	pub include_metadata: bool,
}

impl RestPath<SystemLabelReqPath> for ReqBody {
	fn get_path(req_params: SystemLabelReqPath) -> core::result::Result<String, HttpError> {
		Ok(req_params.path)
	}
}

impl ReqBody {
	pub fn new(address: String, params: Params) -> Self {
		ReqBody { name: params.name(), address, params, include_metadata: true }
	}

	pub fn new_with_false_metadata(address: String, params: Params) -> Self {
		ReqBody { name: params.name(), address, params, include_metadata: false }
	}
}

pub trait AchainableSystemLabelName {
	fn name(&self) -> String;
}

pub fn web3_network_to_chain(network: &Web3Network) -> String {
	match network {
		Web3Network::Polkadot => "polkadot".into(),
		Web3Network::Kusama => "kusama".into(),
		Web3Network::Litentry => "litentry".into(),
		Web3Network::Litmus => "litmus".into(),
		Web3Network::LitentryRococo => "litentry_rococo".into(),
		Web3Network::Khala => "khala".into(),
		Web3Network::SubstrateTestnet => "substrate_testnet".into(),
		Web3Network::Ethereum => "ethereum".into(),
		Web3Network::Bsc => "bsc".into(),
		Web3Network::BitcoinP2tr => "bitcoin_p2tr".into(),
		Web3Network::BitcoinP2pkh => "bitcoin_p2pkh".into(),
		Web3Network::BitcoinP2sh => "bitcoin_p2sh".into(),
		Web3Network::BitcoinP2wpkh => "bitcoin_p2wpkh".into(),
		Web3Network::BitcoinP2wsh => "bitcoin_p2wsh".into(),
		Web3Network::Polygon => "polygon".into(),
		Web3Network::Arbitrum => "arbitrum".into(),
		Web3Network::Solana => "solana".into(),
		Web3Network::Combo => "combo".into(),
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Params {
	ParamsBasicType(ParamsBasicType),
	ParamsBasicTypeWithAmount(ParamsBasicTypeWithAmount),
	ParamsBasicTypeWithAmounts(ParamsBasicTypeWithAmounts),
	ParamsBasicTypeWithDate(ParamsBasicTypeWithDate),
	ParamsBasicTypeWithAmountToken(ParamsBasicTypeWithAmountToken),
	ParamsBasicTypeWithBetweenPercents(ParamsBasicTypeWithBetweenPercents),
	ParamsBasicTypeWithDateInterval(ParamsBasicTypeWithDateInterval),
	ParamsBasicTypeWithToken(ParamsBasicTypeWithToken),
	ParamsBasicTypeWithDatePercent(ParamsBasicTypeWithDatePercent),
	ParamsBasicTypeWithClassOfYear(ParamsBasicTypeWithClassOfYear),
	ParamsBasicTypeWithAmountHolding(ParamsBasicTypeWithAmountHolding),
	ParamsBasicTypeWithMirror(ParamsBasicTypeWithMirror),
}

impl AchainableSystemLabelName for Params {
	fn name(&self) -> String {
		match self {
			Params::ParamsBasicType(a) => a.name.clone(),
			Params::ParamsBasicTypeWithAmount(a) => a.name.clone(),
			Params::ParamsBasicTypeWithAmounts(a) => a.name.clone(),
			Params::ParamsBasicTypeWithDate(a) => a.name.clone(),
			Params::ParamsBasicTypeWithAmountToken(a) => a.name.clone(),
			Params::ParamsBasicTypeWithBetweenPercents(a) => a.name.clone(),
			Params::ParamsBasicTypeWithDateInterval(a) => a.name.clone(),
			Params::ParamsBasicTypeWithToken(a) => a.name.clone(),
			Params::ParamsBasicTypeWithDatePercent(e) => e.name.clone(),
			Params::ParamsBasicTypeWithClassOfYear(c) => c.name.clone(),
			Params::ParamsBasicTypeWithAmountHolding(a) => a.name.clone(),
			Params::ParamsBasicTypeWithMirror(a) => a.name.clone(),
		}
	}
}

impl TryFrom<AchainableParams> for Params {
	type Error = VCMPError;
	fn try_from(ap: AchainableParams) -> Result<Self, Self::Error> {
		match ap.clone() {
			AchainableParams::AmountHolding(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let amount = ap.to_string(&p.amount)?;
				let date = ap.to_string(&p.date)?;
				let token =
					if p.token.is_some() { Some(ap.to_string(&p.token.unwrap())?) } else { None };

				let p =
					ParamsBasicTypeWithAmountHolding::one(name, &network[0], amount, date, token);
				Ok(Params::ParamsBasicTypeWithAmountHolding(p))
			},
			AchainableParams::AmountToken(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let amount = ap.to_string(&p.amount)?;
				let token =
					if p.token.is_some() { Some(ap.to_string(&p.token.unwrap())?) } else { None };

				// At this step, we do not care about the content inside the chains and instead use real chain data to fill in the request
				// so use network[0] as a placehold.
				let p = ParamsBasicTypeWithAmountToken::new(name, &network[0], amount, token);
				Ok(Params::ParamsBasicTypeWithAmountToken(p))
			},
			AchainableParams::Amount(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let amount = ap.to_string(&p.amount)?;

				let p = ParamsBasicTypeWithAmount::new(name, &network[0], amount);
				Ok(Params::ParamsBasicTypeWithAmount(p))
			},
			AchainableParams::Amounts(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let amount1 = ap.to_string(&p.amount1)?;
				let amount2 = ap.to_string(&p.amount2)?;

				let p = ParamsBasicTypeWithAmounts::new(name, &network[0], amount1, amount2);
				Ok(Params::ParamsBasicTypeWithAmounts(p))
			},
			AchainableParams::Basic(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;

				let p = ParamsBasicType::new(name, &network[0]);
				Ok(Params::ParamsBasicType(p))
			},
			AchainableParams::BetweenPercents(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let greater_than_or_equal_to = ap.to_string(&p.greater_than_or_equal_to)?;
				let less_than_or_equal_to = ap.to_string(&p.less_than_or_equal_to)?;

				let p = ParamsBasicTypeWithBetweenPercents::new(
					name,
					&network[0],
					greater_than_or_equal_to,
					less_than_or_equal_to,
				);
				Ok(Params::ParamsBasicTypeWithBetweenPercents(p))
			},
			AchainableParams::ClassOfYear(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;

				// NOTE:
				// The front-end use case doesn’t need specific date period, but in general an overall evaluation. While Achainable interface requires date1 and date2, to avoid ambiguity, the start and end dates here are fixed as follow:
				// date1: 2015-01-01
				// date2: 2023-01-01
				let date1 = "2015-01-01".into();
				let date2 = "2023-01-01".into();

				let p = ParamsBasicTypeWithClassOfYear::new(name, &network[0], date1, date2);
				Ok(Params::ParamsBasicTypeWithClassOfYear(p))
			},
			AchainableParams::DateInterval(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let start_date = ap.to_string(&p.start_date)?;
				let end_date = ap.to_string(&p.end_date)?;

				let p =
					ParamsBasicTypeWithDateInterval::new(name, &network[0], start_date, end_date);
				Ok(Params::ParamsBasicTypeWithDateInterval(p))
			},
			AchainableParams::DatePercent(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let token = ap.to_string(&p.token)?;
				let date = ap.to_string(&p.date)?;
				let percent = ap.to_string(&p.percent)?;

				let p =
					ParamsBasicTypeWithDatePercent::new(name, &network[0], token, date, percent);
				Ok(Params::ParamsBasicTypeWithDatePercent(p))
			},
			AchainableParams::Date(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let date = ap.to_string(&p.date)?;

				let p = ParamsBasicTypeWithDate::new(name, &network[0], date);
				Ok(Params::ParamsBasicTypeWithDate(p))
			},
			AchainableParams::Token(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;
				let token = ap.to_string(&p.token)?;

				let p = ParamsBasicTypeWithToken::new(name, &network[0], token);
				Ok(Params::ParamsBasicTypeWithToken(p))
			},
			AchainableParams::Mirror(p) => {
				let name = ap.to_string(&p.name)?;
				let network = &p.chain;

				let post_quantity = if let Some(post_quantity) = p.post_quantity {
					let post = ap.to_string(&post_quantity)?;
					Some(post)
				} else {
					None
				};

				let p = ParamsBasicTypeWithMirror::new(name, &network[0], post_quantity);
				Ok(Params::ParamsBasicTypeWithMirror(p))
			},
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithAmountHolding {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub amount: String,
	pub date: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub token: Option<String>,
}

impl ParamsBasicTypeWithAmountHolding {
	pub fn new(network: &Web3Network, amount: String, date: String, token: Option<String>) -> Self {
		let chain = web3_network_to_chain(network);
		let name = if token.is_some() {
			"ERC20 hodling {amount} of {token} since {date}".into()
		} else {
			"Balance hodling {amount} since {date}".into()
		};

		Self { name, chain, amount, date, token }
	}

	pub fn one(
		name: String,
		network: &Web3Network,
		amount: String,
		date: String,
		token: Option<String>,
	) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, amount, date, token }
	}
}

// ParamsBasicTypeWithClassOfYear
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithClassOfYear {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub date1: String,
	pub date2: String,
}

impl ParamsBasicTypeWithClassOfYear {
	pub fn new(name: String, network: &Web3Network, date1: String, date2: String) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, date1, date2 }
	}
}

pub enum EClassOfYear {
	Year2020,
	Year2021,
	Year2022,
}

impl EClassOfYear {
	pub fn get(&self) -> ParamsBasicTypeWithClassOfYear {
		match self {
			EClassOfYear::Year2020 => ParamsBasicTypeWithClassOfYear::class_of_2020(),
			EClassOfYear::Year2021 => ParamsBasicTypeWithClassOfYear::class_of_2021(),
			EClassOfYear::Year2022 => ParamsBasicTypeWithClassOfYear::class_of_2022(),
		}
	}
}

impl ParamsBasicTypeWithClassOfYear {
	fn class_of_2020() -> Self {
		Self {
			name: "Account created between {dates}".into(),
			chain: "ethereum".into(),
			date1: "2020-01-01T00:00:00.000Z".into(),
			date2: "2020-12-31T23:59:59.999Z".into(),
		}
	}

	fn class_of_2021() -> Self {
		Self {
			name: "Account created between {dates}".into(),
			chain: "ethereum".into(),
			date1: "2021-01-01T00:00:00.000Z".into(),
			date2: "2021-12-31T23:59:59.999Z".into(),
		}
	}

	fn class_of_2022() -> Self {
		Self {
			name: "Account created between {dates}".into(),
			chain: "ethereum".into(),
			date1: "2022-01-01T00:00:00.000Z".into(),
			date2: "2022-12-31T23:59:59.999Z".into(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicType {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
}

impl ParamsBasicType {
	pub fn new(name: String, network: &Web3Network) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithAmount {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub amount: String,
}

impl ParamsBasicTypeWithAmount {
	pub fn new(name: String, network: &Web3Network, amount: String) -> Self {
		let chain = web3_network_to_chain(network);

		Self { name, chain, amount }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithDate {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub date: String,
}

impl ParamsBasicTypeWithDate {
	pub fn new(name: String, network: &Web3Network, date: String) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, date }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithAmounts {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub amount1: String,
	pub amount2: String,
}

impl ParamsBasicTypeWithAmounts {
	pub fn new(name: String, network: &Web3Network, amount1: String, amount2: String) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, amount1, amount2 }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithAmountToken {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub amount: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub token: Option<String>,
}

impl ParamsBasicTypeWithAmountToken {
	pub fn new(name: String, network: &Web3Network, amount: String, token: Option<String>) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, amount, token }
	}
}

// Balance between percents
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithBetweenPercents {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub greater_than_or_equal_to: String,
	pub less_than_or_equal_to: String,
}

impl ParamsBasicTypeWithBetweenPercents {
	pub fn new(
		name: String,
		network: &Web3Network,
		greater_than_or_equal_to: String,
		less_than_or_equal_to: String,
	) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, greater_than_or_equal_to, less_than_or_equal_to }
	}
}

// ParamsBasicTypeWithDateInterval
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithDateInterval {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub start_date: String,
	pub end_date: String,
}

impl ParamsBasicTypeWithDateInterval {
	pub fn new(name: String, network: &Web3Network, start_date: String, end_date: String) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, start_date, end_date }
	}
}

// ParamsBasicTypeWithToken
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithToken {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub token: String,
}

impl ParamsBasicTypeWithToken {
	pub fn new(name: String, network: &Web3Network, token: String) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, token }
	}
}

// ParamsBasicTypeWithDatePercent
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithDatePercent {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,
	pub token: String,
	pub date: String,
	pub percent: String,
}

impl ParamsBasicTypeWithDatePercent {
	pub fn new(
		name: String,
		network: &Web3Network,
		token: String,
		date: String,
		percent: String,
	) -> Self {
		let chain = web3_network_to_chain(network);
		Self { name, chain, token, date, percent }
	}
}

impl Default for ParamsBasicTypeWithDatePercent {
	fn default() -> Self {
		Self {
			name: "Balance dropped {percent} since {date}".into(),
			chain: "ethereum".into(),
			token: "ETH".into(),
			date: "14D".into(),
			percent: "80".into(),
		}
	}
}

// ParamsBasicTypeWithToken
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsBasicTypeWithMirror {
	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub name: String,

	pub chain: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub post_quantity: Option<String>,
}

impl ParamsBasicTypeWithMirror {
	pub fn new(name: String, network: &Web3Network, post_quantity: Option<String>) -> Self {
		let chain: &'static str = network.into();
		Self { name, chain: chain.to_string(), post_quantity }
	}
}

fn check_achainable_label(
	client: &mut AchainableClient,
	address: &str,
	params: Params,
) -> Result<bool, Error> {
	// TODO: double-check it with Zhouhui if we don't ever need metadata here
	let body = ReqBody::new_with_false_metadata(address.into(), params);
	client
		.post(SystemLabelReqPath::default(), &body)
		.and_then(AchainableClient::parse)
}

/// A4/A7/A10/A11
pub trait AchainableHolder {
	fn is_holder(
		&mut self,
		address: &str,
		amount_holding: ParamsBasicTypeWithAmountHolding,
	) -> Result<bool, Error>;
}

impl AchainableHolder for AchainableClient {
	fn is_holder(
		&mut self,
		address: &str,
		amount_holding: ParamsBasicTypeWithAmountHolding,
	) -> Result<bool, Error> {
		check_achainable_label(
			self,
			address,
			Params::ParamsBasicTypeWithAmountHolding(amount_holding),
		)
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
			.get("display")
			.and_then(|displays| {
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
			.flatten();

		debug!("Total txs, display text: {:?}", display_text);

		if let Some(display_text) = display_text {
			// TODO:
			// text field format: Total transactions under 1 (Transactions: 0)
			let split_text = display_text.split(": ").collect::<Vec<&str>>();
			if split_text.len() != 2 {
				return Err(Error::AchainableError("Invalid array".to_string()));
			}

			let mut value_text = split_text[1].to_string();

			// pop the last char: ")"
			value_text.pop();

			let value = value_text
				.parse::<u64>()
				.map_err(|e| Error::AchainableError(format!("Parse txn count error: {:?}", e)))?;

			return Ok(value);
		}

		Err(Error::AchainableError("Invalid response".to_string()))
	}
}
pub trait AchainableAccountTotalTransactions {
	/// NOTE: Achinable "chain" fieild must be one of [ethereum, polkadot, kusama, litmus, litentry, khala]
	fn total_transactions(
		&mut self,
		network: &Web3Network,
		addresses: Vec<String>,
	) -> Result<u64, Error>;
}

impl AchainableAccountTotalTransactions for AchainableClient {
	fn total_transactions(
		&mut self,
		network: &Web3Network,
		addresses: Vec<String>,
	) -> Result<u64, Error> {
		let mut txs = 0_u64;

		loop_with_abort_strategy::<fn(&_) -> bool, String, Error>(
			addresses,
			|address| {
				let name = "Account total transactions under {amount}".to_string();
				let amount = "1".to_string();

				let param = ParamsBasicTypeWithAmount::new(name, network, amount);
				let body = ReqBody::new_with_false_metadata(
					address.into(),
					Params::ParamsBasicTypeWithAmount(param),
				);
				let tx =
					self.post(SystemLabelReqPath::default(), &body).and_then(Self::parse_txs)?;
				txs += tx;

				Ok(LoopControls::Continue)
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		)
		.map_err(|errors| errors[0].clone())?;

		Ok(txs)
	}
}

pub trait AchainableUtils {
	fn get_balance(response: serde_json::Value) -> Result<f64, Error>;
}

impl AchainableUtils for AchainableClient {
	fn get_balance(response: serde_json::Value) -> Result<f64, Error> {
		let display_text = response
			.get("display")
			.and_then(|displays| {
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
			.flatten();
		if let Some(display_text) = display_text {
			// If it is a newly created brand new account, Achainable returns `Address has no [token] balance`,
			// so it will not be parsed and will directly return 0
			if display_text.contains("Address has no") {
				return Ok(0_f64);
			}

			// TODO:
			// text field format: Balance over 0 (Balance is 588.504602529)
			let split_text = display_text.split("Balance is ").collect::<Vec<&str>>();
			if split_text.len() != 2 {
				return Err(Error::AchainableError("Invalid array".to_string()));
			}

			let mut value_text = split_text[1].to_string();

			// pop the last char: ")"
			value_text.pop();

			let value = value_text
				.parse::<f64>()
				.map_err(|e| Error::AchainableError(format!("Parse balance error: {:?}", e)))?;

			return Ok(value);
		}

		Err(Error::AchainableError("Invalid response".to_string()))
	}
}

pub trait HoldingAmount {
	fn holding_amount(&mut self, addresses: Vec<String>, param: Params) -> Result<String, Error>;
}
impl HoldingAmount for AchainableClient {
	fn holding_amount(&mut self, addresses: Vec<String>, param: Params) -> Result<String, Error> {
		let mut total_balance = 0_f64;

		loop_with_abort_strategy::<fn(&_) -> bool, String, Error>(
			addresses,
			|address| {
				let body = ReqBody::new_with_false_metadata(address.into(), param.clone());
				let balance =
					self.post(SystemLabelReqPath::default(), &body).and_then(Self::get_balance)?;
				total_balance += balance;

				Ok(LoopControls::Continue)
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		)
		.map_err(|errors| errors[0].clone())?;

		Ok(total_balance.to_string())
	}
}

pub trait AchainableTagAccount {
	fn fresh_account(&mut self, address: &str) -> Result<bool, Error>;
	fn og_account(&mut self, address: &str) -> Result<bool, Error>;
	fn class_of_year(
		&mut self,
		address: &str,
		param: ParamsBasicTypeWithClassOfYear,
	) -> Result<bool, Error>;
	fn address_found_on_bsc(&mut self, address: &str) -> Result<bool, Error>;
	fn eth_drained_in_last_fortnight(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_validator(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_validator(&mut self, address: &str) -> Result<bool, Error>;
}

pub trait AchainableTagBalance {
	fn polkadot_dolphin(&mut self, address: &str) -> Result<bool, Error>;
	fn kusama_dolphin(&mut self, address: &str) -> Result<bool, Error>;
	fn polkadot_whale(&mut self, address: &str) -> Result<bool, Error>;
	fn kusama_whale(&mut self, address: &str) -> Result<bool, Error>;
	fn under_10_eth_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn under_10_lit_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn over_100_eth_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn between_10_to_100_eth_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn eth_millionaire(&mut self, address: &str) -> Result<bool, Error>;
	fn eth2_validator_eligible(&mut self, address: &str) -> Result<bool, Error>;
	fn over_100_weth_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn over_100_lit_bep20_amount(&mut self, address: &str) -> Result<bool, Error>;
	fn native_lit_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn erc20_lit_holder(&mut self, address: &str) -> Result<bool, Error>;
	fn bep20_lit_holder(&mut self, address: &str) -> Result<bool, Error>;
}

pub trait AchainableTagDotsama {
	fn is_polkadot_treasury_proposal_beneficiary(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_treasury_proposal_beneficiary(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_tip_finder(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_tip_finder(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_tip_beneficiary(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_tip_beneficiary(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_opengov_proposer(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_opengov_proposer(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_fellowship_proposer(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_fellowship_proposer(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_fellowship_member(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_fellowship_member(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_ex_councilor(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_ex_councilor(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_councilor(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_councilor(&mut self, address: &str) -> Result<bool, Error>;
	fn is_polkadot_bounty_curator(&mut self, address: &str) -> Result<bool, Error>;
	fn is_kusama_bounty_curator(&mut self, address: &str) -> Result<bool, Error>;
}

pub trait AchainableTagDeFi {
	fn uniswap_v2_user(&mut self, address: &str) -> Result<bool, Error>;
	fn uniswap_v3_user(&mut self, address: &str) -> Result<bool, Error>;
	fn uniswap_v2_lp_in_2022(&mut self, address: &str) -> Result<bool, Error>;
	fn uniswap_v3_lp_in_2022(&mut self, address: &str) -> Result<bool, Error>;
	fn usdc_uniswap_v2_lp(&mut self, address: &str) -> Result<bool, Error>;
	fn usdc_uniswap_v3_lp(&mut self, address: &str) -> Result<bool, Error>;
	fn usdt_uniswap_lp(&mut self, address: &str) -> Result<bool, Error>;
	fn usdt_uniswap_v2_lp(&mut self, address: &str) -> Result<bool, Error>;
	fn usdt_uniswap_v3_lp(&mut self, address: &str) -> Result<bool, Error>;
	fn aave_v2_lender(&mut self, address: &str) -> Result<bool, Error>;
	fn aave_v2_borrower(&mut self, address: &str) -> Result<bool, Error>;
	fn aave_v3_lender(&mut self, address: &str) -> Result<bool, Error>;
	fn aave_v3_borrower(&mut self, address: &str) -> Result<bool, Error>;
	fn curve_trader(&mut self, address: &str) -> Result<bool, Error>;
	fn curve_trader_in_2022(&mut self, address: &str) -> Result<bool, Error>;
	fn curve_liquidity_provider(&mut self, address: &str) -> Result<bool, Error>;
	fn curve_liquidity_provider_in_2022(&mut self, address: &str) -> Result<bool, Error>;
	fn swapped_with_metamask_in_2022(&mut self, address: &str) -> Result<bool, Error>;
}

impl AchainableTagAccount for AchainableClient {
	fn fresh_account(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Account created after {date}".to_string();
		let chain = Web3Network::Ethereum;
		let date = "30D".to_string();
		let param = ParamsBasicTypeWithDate::new(name, &chain, date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDate(param))
	}

	fn og_account(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Account created before {date}".to_string();
		let chain = Web3Network::Ethereum;
		let date = "2020-01-01T00:00:00.000Z".to_string();
		let param = ParamsBasicTypeWithDate::new(name, &chain, date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDate(param))
	}

	fn class_of_year(
		&mut self,
		address: &str,
		param: ParamsBasicTypeWithClassOfYear,
	) -> Result<bool, Error> {
		check_achainable_label(self, address, Params::ParamsBasicTypeWithClassOfYear(param))
	}

	fn address_found_on_bsc(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("Account found on {chain}".to_string(), &Web3Network::Bsc);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn eth_drained_in_last_fortnight(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicTypeWithDatePercent::default();
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDatePercent(param))
	}

	fn is_polkadot_validator(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("Validator".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_validator(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("Validator".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}
}

impl AchainableTagBalance for AchainableClient {
	fn polkadot_dolphin(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance between percents".to_string();
		let chain = Web3Network::Polkadot;
		let a1 = "0.01".to_string();
		let a2 = "0.0999999999999999".to_string();
		let param = ParamsBasicTypeWithBetweenPercents::new(name, &chain, a1, a2);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithBetweenPercents(param))
	}

	fn kusama_dolphin(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance between percents".to_string();
		let chain = Web3Network::Kusama;
		let a1 = "0.01".to_string();
		let a2 = "0.0999999999999999".to_string();
		let param = ParamsBasicTypeWithBetweenPercents::new(name, &chain, a1, a2);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithBetweenPercents(param))
	}

	fn polkadot_whale(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance between percents".to_string();
		let chain = Web3Network::Polkadot;
		let a1 = "0.01".to_string();
		let a2 = "100".to_string();
		let param = ParamsBasicTypeWithBetweenPercents::new(name, &chain, a1, a2);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithBetweenPercents(param))
	}

	fn kusama_whale(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance between percents".to_string();
		let chain = Web3Network::Kusama;
		let a1 = "0.01".to_string();
		let a2 = "100".to_string();
		let param = ParamsBasicTypeWithBetweenPercents::new(name, &chain, a1, a2);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithBetweenPercents(param))
	}

	fn under_10_eth_holder(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance under {amount}".to_string();
		let chain = Web3Network::Ethereum;
		let amount = "10".to_string();
		let param = ParamsBasicTypeWithAmount::new(name, &chain, amount);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmount(param))
	}

	fn under_10_lit_holder(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance under {amount}".to_string();
		let chain = Web3Network::Litentry;
		let amount = "10".to_string();
		let param = ParamsBasicTypeWithAmount::new(name, &chain, amount);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmount(param))
	}

	fn over_100_eth_holder(&mut self, address: &str) -> Result<bool, Error> {
		let name = "Balance over {amount}".to_string();
		let chain = Web3Network::Ethereum;
		let amount = "100".to_string();
		let param = ParamsBasicTypeWithAmount::new(name, &chain, amount);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmount(param))
	}

	fn between_10_to_100_eth_holder(&mut self, address: &str) -> Result<bool, Error> {
		// 10 - 100 ETH Holder
		let name = "Balance between {amounts}".to_string();
		let chain = Web3Network::Ethereum;
		let amount1 = "10".to_string();
		let amount2 = "100".to_string();
		let param = ParamsBasicTypeWithAmounts::new(name, &chain, amount1, amount2);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmounts(param))
	}

	fn eth_millionaire(&mut self, address: &str) -> Result<bool, Error> {
		// ETH Millionaire
		let name = "Balance over {amount} dollars".to_string();
		let chain = Web3Network::Ethereum;
		let amount = "100".to_string();
		let param = ParamsBasicTypeWithAmount::new(name, &chain, amount);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmount(param))
	}

	fn eth2_validator_eligible(&mut self, address: &str) -> Result<bool, Error> {
		// ETH2 Validator Eligible
		let name = "Balance over {amount}".to_string();
		let chain = Web3Network::Ethereum;
		let amount = "32".to_string();
		let param = ParamsBasicTypeWithAmount::new(name, &chain, amount);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmount(param))
	}

	fn over_100_weth_holder(&mut self, address: &str) -> Result<bool, Error> {
		// 100+ WETH Holder
		let name = "ERC20 balance over {amount}".to_string();
		let chain = Web3Network::Ethereum;
		let amount = "100".to_string();

		let param = ParamsBasicTypeWithAmountToken::new(
			name,
			&chain,
			amount,
			Some(WETH_TOKEN_ADDRESS.to_string()),
		);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmountToken(param))
	}

	fn over_100_lit_bep20_amount(&mut self, address: &str) -> Result<bool, Error> {
		// 100+ LIT BEP20 Holder
		let name = "BEP20 balance over {amount}".to_string();
		let chain = Web3Network::Bsc;
		let amount = "100".to_string();

		let param = ParamsBasicTypeWithAmountToken::new(
			name,
			&chain,
			amount,
			Some(LIT_TOKEN_ADDRESS.to_string()),
		);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmountToken(param))
	}

	fn native_lit_holder(&mut self, address: &str) -> Result<bool, Error> {
		// Native LIT Hodler
		let param = ParamsBasicTypeWithAmountHolding::new(
			&Web3Network::Litentry,
			"10".to_string(),
			"2023-01-01T00:00:00.000Z".to_string(),
			None,
		);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmountHolding(param))
	}

	fn erc20_lit_holder(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicTypeWithAmountHolding::new(
			&Web3Network::Ethereum,
			"10".to_string(),
			"2022-01-01T00:00:00.000Z".to_string(),
			Some(LIT_TOKEN_ADDRESS.to_string()),
		);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmountHolding(param))
	}

	fn bep20_lit_holder(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicTypeWithAmountHolding::new(
			&Web3Network::Bsc,
			"10".to_string(),
			"2022-01-01T00:00:00.000Z".to_string(),
			Some(LIT_TOKEN_ADDRESS.to_string()),
		);

		check_achainable_label(self, address, Params::ParamsBasicTypeWithAmountHolding(param))
	}
}

impl AchainableTagDotsama for AchainableClient {
	fn is_polkadot_treasury_proposal_beneficiary(&mut self, address: &str) -> Result<bool, Error> {
		let param =
			ParamsBasicType::new("TreasuryProposalBeneficiary".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_treasury_proposal_beneficiary(&mut self, address: &str) -> Result<bool, Error> {
		let param =
			ParamsBasicType::new("TreasuryProposalBeneficiary".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_tip_finder(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("TipFinder".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_tip_finder(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("TipFinder".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_tip_beneficiary(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("TipBeneficiary".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_tip_beneficiary(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("TipBeneficiary".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_opengov_proposer(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("OpenGovProposer".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_opengov_proposer(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("OpenGovProposer".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_fellowship_proposer(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("FellowshipProposer".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_fellowship_proposer(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("FellowshipProposer".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_fellowship_member(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("FellowshipMember".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_fellowship_member(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("FellowshipMember".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_ex_councilor(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("ExCouncilor".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_ex_councilor(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("ExCouncilor".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_councilor(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("Councilor".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_councilor(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("Councilor".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_polkadot_bounty_curator(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("BountyCurator".to_string(), &Web3Network::Polkadot);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}

	fn is_kusama_bounty_curator(&mut self, address: &str) -> Result<bool, Error> {
		let param = ParamsBasicType::new("BountyCurator".to_string(), &Web3Network::Kusama);
		check_achainable_label(self, address, Params::ParamsBasicType(param))
	}
}

impl AchainableTagDeFi for AchainableClient {
	fn uniswap_v2_user(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V2 trader
		let name_trader = "Uniswap V2 trader";
		// Uniswap V2 liquidity provider
		let name_provider = "Uniswap V2 liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		if request_basic_type_with_token(self, address, name_trader, &chain, None)?
			|| request_basic_type_with_token(self, address, name_provider, &chain, None)?
		{
			return Ok(true);
		}

		Ok(false)
	}

	fn uniswap_v3_user(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V3 trader
		let name_trader = "Uniswap V3 trader";
		// Uniswap V3 liquidity provider
		let name_provider = "Uniswap V3 liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		if request_basic_type_with_token(self, address, name_trader, &chain, None)?
			|| request_basic_type_with_token(self, address, name_provider, &chain, None)?
		{
			return Ok(true);
		}

		Ok(false)
	}

	fn uniswap_v2_lp_in_2022(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V2 liquidity provider
		let name = "Uniswap V2 liquidity provider".to_string();
		let chain = Web3Network::Ethereum;
		let start_date = "2022-01-01T00:00:00.000Z".to_string();
		let end_date = "2022-12-31T23:59:59.999Z".to_string();

		let param = ParamsBasicTypeWithDateInterval::new(name, &chain, start_date, end_date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDateInterval(param))
	}

	fn uniswap_v3_lp_in_2022(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V3 liquidity provider
		let name = "Uniswap V3 liquidity provider".to_string();
		let chain = Web3Network::Ethereum;
		let start_date = "2022-01-01T00:00:00.000Z".to_string();
		let end_date = "2022-12-31T23:59:59.999Z".to_string();

		let param = ParamsBasicTypeWithDateInterval::new(name, &chain, start_date, end_date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDateInterval(param))
	}

	fn usdc_uniswap_v2_lp(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V2 {token} liquidity provider
		let name = "Uniswap V2 {token} liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		request_basic_type_with_token(self, address, name, &chain, Some(USDC_TOKEN_ADDRESS))
	}

	fn usdc_uniswap_v3_lp(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V3 {token} liquidity provider
		let name = "Uniswap V3 {token} liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		request_basic_type_with_token(self, address, name, &chain, Some(USDC_TOKEN_ADDRESS))
	}

	fn usdt_uniswap_lp(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V2 {token} liquidity provider
		// Uniswap V3 {token} liquidity provider
		if self.usdt_uniswap_v2_lp(address)? || self.usdt_uniswap_v3_lp(address)? {
			return Ok(true);
		}

		Ok(false)
	}

	fn usdt_uniswap_v2_lp(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V2 {token} liquidity provider
		let name = "Uniswap V2 {token} liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		request_basic_type_with_token(self, address, name, &chain, Some(USDT_TOKEN_ADDRESS))
	}

	fn usdt_uniswap_v3_lp(&mut self, address: &str) -> Result<bool, Error> {
		// Uniswap V3 {token} liquidity provider
		let name = "Uniswap V3 {token} liquidity provider";
		let chain: Web3Network = Web3Network::Ethereum;

		request_basic_type_with_token(self, address, name, &chain, Some(USDT_TOKEN_ADDRESS))
	}

	fn aave_v2_lender(&mut self, address: &str) -> Result<bool, Error> {
		// Aave V2 Lender
		let name = "Aave V2 Lender";
		let chain: Web3Network = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn aave_v2_borrower(&mut self, address: &str) -> Result<bool, Error> {
		// Aave V2 Borrower
		let name = "Aave V2 Borrower";
		let chain: Web3Network = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn aave_v3_lender(&mut self, address: &str) -> Result<bool, Error> {
		// Aave V3 Lender
		let name = "Aave V3 Lender";
		let chain: Web3Network = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn aave_v3_borrower(&mut self, address: &str) -> Result<bool, Error> {
		// Aave V3 Borrower
		let name = "Aave V3 Borrower";
		let chain: Web3Network = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn curve_trader(&mut self, address: &str) -> Result<bool, Error> {
		// Curve Trader
		let name = "Curve Trader";
		let chain = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn curve_trader_in_2022(&mut self, address: &str) -> Result<bool, Error> {
		// Curve Trader
		let name = "Curve Trader".to_string();
		let chain = Web3Network::Ethereum;
		let start_date = "2022-01-01T00:00:00.000Z".to_string();
		let end_date = "2022-12-31T23:59:59.999Z".to_string();

		let param = ParamsBasicTypeWithDateInterval::new(name, &chain, start_date, end_date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDateInterval(param))
	}

	fn curve_liquidity_provider(&mut self, address: &str) -> Result<bool, Error> {
		// Curve Liquidity Provider
		let name = "Curve Liquidity Provider";
		let chain = Web3Network::Ethereum;
		request_basic_type_with_token(self, address, name, &chain, None)
	}

	fn curve_liquidity_provider_in_2022(&mut self, address: &str) -> Result<bool, Error> {
		// Curve Liquidity Provider
		let name = "Curve Liquidity Provider".to_string();
		let chain = Web3Network::Ethereum;
		let start_date = "2022-01-01T00:00:00.000Z".to_string();
		let end_date = "2022-12-31T23:59:59.999Z".to_string();

		let param = ParamsBasicTypeWithDateInterval::new(name, &chain, start_date, end_date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDateInterval(param))
	}

	fn swapped_with_metamask_in_2022(&mut self, address: &str) -> Result<bool, Error> {
		// MetaMask trader
		let name = "MetaMask trader".to_string();
		let chain = Web3Network::Ethereum;
		let start_date = "2022-01-01T00:00:00.000Z".to_string();
		let end_date = "2022-12-31T23:59:59.999Z".to_string();

		let param = ParamsBasicTypeWithDateInterval::new(name, &chain, start_date, end_date);
		check_achainable_label(self, address, Params::ParamsBasicTypeWithDateInterval(param))
	}
}

fn request_basic_type_with_token(
	client: &mut AchainableClient,
	address: &str,
	name: &str,
	network: &Web3Network,
	token: Option<&str>,
) -> Result<bool, Error> {
	if let Some(token) = token {
		let param = ParamsBasicTypeWithToken::new(name.to_string(), network, token.to_string());
		check_achainable_label(client, address, Params::ParamsBasicTypeWithToken(param))
	} else {
		let param = ParamsBasicType::new(name.to_string(), network);
		check_achainable_label(client, address, Params::ParamsBasicType(param))
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LabelQueryReq {
	pub params: LabelQueryReqParams,
	pub include_metadata: bool,
	pub include_widgets: bool,
}

impl<'a> RestPath<ReqPath<'a>> for LabelQueryReq {
	fn get_path(path: ReqPath) -> Result<String, HttpError> {
		Ok(path.path.into())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LabelQueryReqParams {
	pub address: String,
}

pub trait AchainableLabelQuery {
	fn query_label(&mut self, label: &str, param: &LabelQueryReq) -> Result<bool, Error>;
}

impl AchainableLabelQuery for AchainableClient {
	fn query_label(&mut self, label: &str, req: &LabelQueryReq) -> Result<bool, Error> {
		match self.client.post_capture::<ReqPath, LabelQueryReq, serde_json::Value>(
			ReqPath::new(format!("/v1/run/labels/{}", label).as_str()),
			req,
		) {
			Ok(response) => {
				debug!("Achainable query_label, response: {:?}", response);
				AchainableClient::parse(response)
			},
			Err(e) => {
				debug!("Achainable query_label, error: {:?}", e);
				Err(Error::AchainableError(format!("Achainable response error: {}", e)))
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		achainable::{
			AchainableAccountTotalTransactions, AchainableClient, AchainableTagAccount,
			AchainableTagBalance, AchainableTagDeFi, AchainableTagDotsama, AchainableUtils,
		},
		DataProviderConfig,
	};
	use lc_mock_server::run;
	use litentry_primitives::Web3Network;
	use serde_json::Value;

	fn new_achainable_client() -> AchainableClient {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();

		let mut data_provider_config = DataProviderConfig::new().unwrap();
		data_provider_config.set_achainable_url(url).unwrap();
		AchainableClient::new(&data_provider_config)
	}

	#[test]
	fn total_transactions_works() {
		let addresses = vec!["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5".to_string()];

		let mut client = new_achainable_client();
		let r = client.total_transactions(&Web3Network::Litentry, addresses);
		assert!(r.is_ok());
		let r = r.unwrap();
		assert!(r == 41)
	}

	#[test]
	fn fresh_account_works() {
		let mut client = new_achainable_client();

		let res: Result<bool, crate::Error> =
			client.fresh_account("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn og_account_works() {
		let mut client = new_achainable_client();
		let res = client.og_account("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn class_of_year_works() {
		let mut client = new_achainable_client();
		let res = client.class_of_year(
			"0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5",
			crate::achainable::EClassOfYear::Year2020.get(),
		);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn address_found_on_bsc_works() {
		let mut client = new_achainable_client();
		let res = client.address_found_on_bsc("0x3f349bBaFEc1551819B8be1EfEA2fC46cA749aA1");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_validator_works() {
		let mut client = new_achainable_client();
		let res = client.is_polkadot_validator("17bR6rzVsVrzVJS1hM4dSJU43z2MUmz7ZDpPLh8y2fqVg7m");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_validator_works() {
		let mut client = new_achainable_client();
		let res = client.is_kusama_validator("ESRBbWstgpPV1pVBsqjMo717rA8HLrtQvEUVwAGeFZyKcia");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn polkadot_dolphin_works() {
		let mut client = new_achainable_client();
		let res = client.polkadot_dolphin("1soESeTVLfse9e2G8dRSMUyJ2SWad33qhtkjQTv9GMToRvP");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn kusama_dolphin_works() {
		let mut client = new_achainable_client();
		let res = client.kusama_dolphin("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn polkadot_whale_works() {
		let mut client = new_achainable_client();
		let res = client.polkadot_whale("1soESeTVLfse9e2G8dRSMUyJ2SWad33qhtkjQTv9GMToRvP");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn kusama_whale_works() {
		let mut client = new_achainable_client();
		let res = client.kusama_whale("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn under_10_eth_holder_works() {
		let mut client = new_achainable_client();
		let res = client.under_10_eth_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn under_10_lit_holder_works() {
		let mut client = new_achainable_client();
		let res = client.under_10_lit_holder("0x2A038e100F8B85DF21e4d44121bdBfE0c288A869");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn over_100_eth_holder_works() {
		let mut client = new_achainable_client();
		let res = client.over_100_eth_holder("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn between_10_to_100_eth_holder_works() {
		let mut client = new_achainable_client();
		let res = client.between_10_to_100_eth_holder("0x082aB5505CdeA46caeF670754E962830Aa49ED2C");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn eth_millionaire_works() {
		let mut client = new_achainable_client();
		let res = client.eth_millionaire("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn eth2_validator_eligible_works() {
		let mut client = new_achainable_client();
		let res = client.eth2_validator_eligible("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn over_100_weth_holder_works() {
		let mut client = new_achainable_client();
		let res = client.over_100_weth_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn over_100_lit_bep20_amount_works() {
		let mut client = new_achainable_client();
		let res = client.over_100_lit_bep20_amount("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn native_lit_holder_works() {
		let mut client = new_achainable_client();
		let res = client.native_lit_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn erc20_lit_holder_works() {
		let mut client = new_achainable_client();
		let res = client.erc20_lit_holder("0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn bep20_lit_holder_works() {
		let mut client = new_achainable_client();
		let res = client.bep20_lit_holder("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_treasury_proposal_beneficiary_works() {
		let mut client = new_achainable_client();
		let res = client.is_polkadot_treasury_proposal_beneficiary(
			"5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW",
		);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_treasury_proposal_beneficiary_works() {
		let mut client = new_achainable_client();
		let res = client.is_kusama_treasury_proposal_beneficiary(
			"CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq",
		);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_tip_finder_works() {
		let mut client = new_achainable_client();
		let res = client.is_polkadot_tip_finder("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_tip_finder_works() {
		let mut client = new_achainable_client();
		let res = client.is_kusama_tip_finder("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_tip_beneficiary_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_polkadot_tip_beneficiary("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_tip_beneficiary_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_kusama_tip_beneficiary("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_opengov_proposer_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_polkadot_opengov_proposer("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_opengov_proposer_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_kusama_opengov_proposer("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_fellowship_proposer_works() {
		let mut client = new_achainable_client();
		let res = client
			.is_polkadot_fellowship_proposer("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_fellowship_proposer_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_kusama_fellowship_proposer("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_fellowship_member_works() {
		let mut client = new_achainable_client();
		let res = client
			.is_polkadot_fellowship_member("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_fellowship_member_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_kusama_fellowship_member("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_ex_councilor_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_polkadot_ex_councilor("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_ex_councilor_works() {
		let mut client = new_achainable_client();
		let res = client.is_kusama_ex_councilor("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_councilor_works() {
		let mut client = new_achainable_client();
		let res = client.is_polkadot_councilor("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_councilor_works() {
		let mut client = new_achainable_client();
		let res = client.is_kusama_councilor("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_polkadot_bounty_curator_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_polkadot_bounty_curator("5CAGKg1NAArpEgze7F7KEnw8T2uFVcAWf6mJNTWeg9PWkdVW");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn is_kusama_bounty_curator_works() {
		let mut client = new_achainable_client();
		let res =
			client.is_kusama_bounty_curator("CsCrPSvLBPSSxJuQmDr18FFZPAQCtKVmsMu9YRTe5VToGeq");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v2_user_works() {
		let mut client = new_achainable_client();
		let res = client.uniswap_v2_user("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v3_user_works() {
		let mut client = new_achainable_client();
		let res = client.uniswap_v3_user("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v2_lp_in_2022_works() {
		let mut client = new_achainable_client();
		let res = client.uniswap_v2_lp_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn uniswap_v3_lp_in_2022_works() {
		let mut client = new_achainable_client();
		let res = client.uniswap_v3_lp_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdc_uniswap_v2_lp_works() {
		let mut client = new_achainable_client();
		let res = client.usdc_uniswap_v2_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdc_uniswap_v3_lp_works() {
		let mut client = new_achainable_client();
		let res = client.usdc_uniswap_v3_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdt_uniswap_lp_works() {
		let mut client = new_achainable_client();
		let res = client.usdt_uniswap_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdt_uniswap_v2_lp_works() {
		let mut client = new_achainable_client();
		let res = client.usdt_uniswap_v2_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn usdt_uniswap_v3_lp_works() {
		let mut client = new_achainable_client();
		let res = client.usdt_uniswap_v3_lp("0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn aave_v2_lender_works() {
		let mut client = new_achainable_client();
		let res = client.aave_v2_lender("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn aave_v2_borrower_works() {
		let mut client = new_achainable_client();
		let res = client.aave_v2_borrower("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn aave_v3_lender_works() {
		let mut client = new_achainable_client();
		let res = client.aave_v3_lender("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn aave_v3_borrower_works() {
		let mut client = new_achainable_client();
		let res = client.aave_v3_borrower("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn curve_trader_works() {
		let mut client = new_achainable_client();
		let res = client.curve_trader("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn curve_trader_in_2022_works() {
		let mut client = new_achainable_client();
		let res = client.curve_trader_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn curve_liquidity_provider_works() {
		let mut client = new_achainable_client();
		let res = client.curve_liquidity_provider("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn curve_liquidity_provider_in_2022_works() {
		let mut client = new_achainable_client();
		let res =
			client.curve_liquidity_provider_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn swapped_with_metamask_in_2022_works() {
		let mut client = new_achainable_client();
		let res =
			client.swapped_with_metamask_in_2022("0x335c0552eb130f3Dfbe6efcB4D2895aED1E9938b");
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(res, true);
	}

	#[test]
	fn class_of_year_invalid_address_works() {
		use crate::achainable::{Params, ParamsBasicTypeWithClassOfYear};
		let invalid_address = "0x06e23f8209eCe9a33E24fd81440D46B08517adb5";

		let name = "Account created between {dates}".into();
		let network = &Web3Network::Ethereum;
		let date1 = "2015-01-01".into();
		let date2 = "2023-01-01".into();
		let p = ParamsBasicTypeWithClassOfYear::new(name, network, date1, date2);

		let mut client = new_achainable_client();
		let res =
			client.query_class_of_year(invalid_address, Params::ParamsBasicTypeWithClassOfYear(p));
		assert!(res.is_ok());

		let year = res.unwrap();
		assert_eq!(year, "Invalid".to_string());
	}

	#[test]
	fn get_balance_works() {
		let data = r#"{
			"name": "ERC20 balance over {amount}",
			"result": true,
			"display": [
				{
					"text": "Balance over 0 (Balance is 370)",
					"result": true
				}
			],
			"analyticsDisplay": [],
			"runningCost": 1
		}"#;
		let value: Value = serde_json::from_str(data).unwrap();
		let balance = AchainableClient::get_balance(value).unwrap();
		assert_eq!(balance, 370.0_f64);
	}
}
