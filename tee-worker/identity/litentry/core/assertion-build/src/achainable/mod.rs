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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use self::{
	amount::build_amount, amount_holding::build_amount_holding, amount_token::build_amount_token,
	amounts::build_amounts, basic::build_basic, between_percents::build_between_percents,
	class_of_year::build_class_of_year, date::build_date, date_interval::build_date_interval,
	date_percent::build_date_percent, mirror::build_on_mirror, token::build_token,
};
use crate::*;
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_data_providers::{
	achainable::{
		AchainableClient, AchainableTagDeFi, HoldingAmount, Params, ParamsBasicTypeWithAmountToken,
	},
	achainable_names::{AchainableNameAmountToken, GetAchainableName},
	DataProviderConfig, Error as DataProviderError, LIT_TOKEN_ADDRESS,
};
use litentry_primitives::{AchainableParams, AssertionBuildRequest};
use std::string::ToString;

pub mod amount;
pub mod amount_holding;
pub mod amount_token;
pub mod amounts;
pub mod basic;
pub mod between_percents;
pub mod class_of_year;
pub mod date;
pub mod date_interval;
pub mod date_percent;
pub mod mirror;
pub mod token;

pub fn build(
	req: &AssertionBuildRequest,
	param: AchainableParams,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	match param {
		AchainableParams::AmountHolding(param) =>
			build_amount_holding(req, param, data_provider_config),
		AchainableParams::AmountToken(param) =>
			build_amount_token(req, param, data_provider_config),
		AchainableParams::Amount(param) => build_amount(req, param, data_provider_config),
		AchainableParams::Amounts(param) => build_amounts(req, param, data_provider_config),
		AchainableParams::Basic(param) => build_basic(req, param, data_provider_config),
		AchainableParams::BetweenPercents(param) =>
			build_between_percents(req, param, data_provider_config),
		AchainableParams::ClassOfYear(param) =>
			build_class_of_year(req, param, data_provider_config),
		AchainableParams::DateInterval(param) =>
			build_date_interval(req, param, data_provider_config),
		AchainableParams::DatePercent(param) =>
			build_date_percent(req, param, data_provider_config),
		AchainableParams::Date(param) => build_date(req, param, data_provider_config),
		AchainableParams::Token(param) => build_token(req, param, data_provider_config),
		AchainableParams::Mirror(param) => build_on_mirror(req, param, data_provider_config),
	}
}

pub fn request_achainable(
	addresses: Vec<String>,
	param: AchainableParams,
	data_provider_config: &DataProviderConfig,
) -> Result<bool> {
	let request_param = Params::try_from(param.clone())?;

	let mut client: AchainableClient = AchainableClient::new(data_provider_config);

	let mut result = false;

	loop_with_abort_strategy::<fn(&_) -> bool, String, DataProviderError>(
		addresses,
		|address| {
			let ret = client.query_system_label(address, request_param.clone())?;

			if ret {
				result = true;
				Ok(LoopControls::Break)
			} else {
				Ok(LoopControls::Continue)
			}
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| {
		Error::RequestVCFailed(
			Assertion::Achainable(param.clone()),
			errors[0].clone().into_error_detail(),
		)
	})?;

	Ok(result)
}

pub fn request_uniswap_v2_or_v3_user(
	addresses: Vec<String>,
	param: AchainableParams,
	data_provider_config: &DataProviderConfig,
) -> Result<(bool, bool)> {
	let _request_param = Params::try_from(param.clone())?;

	let mut client: AchainableClient = AchainableClient::new(data_provider_config);

	let mut v2_user = false;
	let mut v3_user = false;

	loop_with_abort_strategy::<fn(&_) -> bool, String, DataProviderError>(
		addresses,
		|address| {
			if !v2_user {
				v2_user |= client.uniswap_v2_user(address)?;
			}

			if !v3_user {
				v3_user |= client.uniswap_v3_user(address)?;
			}

			if v2_user && v3_user {
				Ok(LoopControls::Break)
			} else {
				Ok(LoopControls::Continue)
			}
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| {
		Error::RequestVCFailed(
			Assertion::Achainable(param.clone()),
			errors[0].clone().into_error_detail(),
		)
	})?;

	Ok((v2_user, v3_user))
}

const INVALID_CLASS_OF_YEAR: &str = "Invalid";
pub fn request_achainable_classofyear(
	addresses: Vec<String>,
	param: AchainableParams,
	data_provider_config: &DataProviderConfig,
) -> Result<(bool, String)> {
	let request_param = Params::try_from(param.clone())?;
	let mut client: AchainableClient = AchainableClient::new(data_provider_config);

	let mut longest_created_year = INVALID_CLASS_OF_YEAR.into();

	loop_with_abort_strategy::<fn(&_) -> bool, String, DataProviderError>(
		addresses,
		|address| {
			let year = client.query_class_of_year(address, request_param.clone())?;

			// In some cases,the metadata field TDF will return null, so if there is a parsing error, we need to continue requesting the next address
			if year.parse::<u32>().is_err() {
				return Ok(LoopControls::Continue)
			}

			if year < longest_created_year {
				longest_created_year = year;
			}

			Ok(LoopControls::Continue)
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| {
		Error::RequestVCFailed(
			Assertion::Achainable(param.clone()),
			errors[0].clone().into_error_detail(),
		)
	})?;

	Ok((longest_created_year.parse::<u32>().is_ok(), longest_created_year))
}

pub fn request_achainable_balance(
	addresses: Vec<String>,
	param: AchainableParams,
	data_provider_config: &DataProviderConfig,
) -> Result<String> {
	let request_param = Params::try_from(param.clone())?;
	let mut client: AchainableClient = AchainableClient::new(data_provider_config);
	let balance = client.holding_amount(addresses, request_param).map_err(|e| {
		Error::RequestVCFailed(Assertion::Achainable(param.clone()), e.into_error_detail())
	})?;

	Ok(balance)
}

pub fn query_lit_holding_amount(
	aparam: &AchainableParams,
	identities: Vec<(Web3Network, Vec<String>)>,
	data_provider_config: &DataProviderConfig,
) -> Result<usize> {
	let mut total_lit_balance = 0_f64;
	let mut client: AchainableClient = AchainableClient::new(data_provider_config);

	loop_with_abort_strategy::<fn(&_) -> bool, (Web3Network, Vec<String>), ErrorDetail>(
		identities,
		|(network, addresses)| {
			let (q_name, q_network, q_token) = if *network == Web3Network::Ethereum {
				(
					AchainableNameAmountToken::ERC20BalanceOverAmount,
					Web3Network::Ethereum,
					Some(LIT_TOKEN_ADDRESS.to_string()),
				)
			} else if *network == Web3Network::Bsc {
				(
					AchainableNameAmountToken::BEP20BalanceOverAmount,
					Web3Network::Bsc,
					Some(LIT_TOKEN_ADDRESS.to_string()),
				)
			} else if *network == Web3Network::Litentry {
				(AchainableNameAmountToken::BalanceOverAmount, Web3Network::Litentry, None)
			} else {
				return Ok(LoopControls::Continue)
			};

			let q_param = ParamsBasicTypeWithAmountToken::new(
				q_name.name().to_string(),
				&q_network,
				"0".to_string(),
				q_token,
			);

			let params = Params::ParamsBasicTypeWithAmountToken(q_param);
			let balance = client
				.holding_amount(addresses.clone(), params)
				.map_err(|e| e.into_error_detail())?
				.parse::<f64>()
				.map_err(|_| ErrorDetail::ParseError)?;

			total_lit_balance += balance;

			Ok(LoopControls::Continue)
		},
		AbortStrategy::FailFast::<fn(&_) -> bool>,
	)
	.map_err(|errors| {
		Error::RequestVCFailed(Assertion::Achainable(aparam.clone()), errors[0].clone())
	})?;

	Ok(total_lit_balance as usize)
}
