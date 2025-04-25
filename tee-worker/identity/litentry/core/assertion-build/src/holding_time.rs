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

use crate::*;
use lc_credentials::{
	achainable::amount_holding_time::AchainableAmountHoldingTimeUpdate, IssuerRuntimeVersion,
};
use lc_data_providers::{
	achainable::{AchainableClient, AchainableHolder, ParamsBasicTypeWithAmountHolding},
	vec_to_string, DataProviderConfig, LIT_TOKEN_ADDRESS, WBTC_TOKEN_ADDRESS,
};
use litentry_primitives::AmountHoldingTimeType;
use std::string::ToString;

/// Here's an example of different assertions in this VC:
///
/// Imagine:
/// ALICE holds 100 LITs since 2018-03-02
/// BOB   holds 100 LITs since 2023-03-07
/// CAROL holds 0.1 LITs since 2020-02-22
///
/// min_amount is 1 LIT
///
/// If they all request A4, these are the received assertions:
/// ALICE:
/// [
///    from_date: < 2019-01-01
///    to_date: >= 2023-03-30 (now)
///    value: true
/// ]
///
/// BOB:
/// [
///    from_date: < 2017-01-01
///    to_date: >= 2023-03-30 (now)
///    value: false
/// ]
///
/// CAROL:
/// [
///    from_date: < 2017-01-01
///    to_date: >= 2023-03-30 (now)
///    value: false
/// ]
///
/// So just from the assertion results you can't distinguish between:
/// BOB, who just started to hold recently,
/// and CAROL, who has been holding for 3 years, but with too little amount
///
/// This is because the data provider doesn't provide more information, it only
/// takes the query with from_date and min_ammount, and returns true or false.
///
/// Please note:
/// the operators are mainly for IDHub's parsing, we will **NEVER** have:
/// - `from_date` with >= op, nor
/// - `value` is false but the `from_date` is something other than 2017-01-01.
///  

pub fn build(
	req: &AssertionBuildRequest,
	htype: AmountHoldingTimeType,
	min_balance: ParameterString,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("Assertion A4 build, who: {:?}", account_id_to_string(&req.who));

	let q_min_balance = prepare_min_balance(&htype, &min_balance)?;
	let accounts = prepare_accounts(&req.identities, &htype);

	// Redundant check in principle, but better safe than sorry :)
	if accounts.is_empty() {
		return Err(emit_error(&htype, &min_balance, ErrorDetail::NoEligibleIdentity));
	}

	let holding_date = search_holding_date(data_provider_config, accounts, &q_min_balance)
		.map_err(|e| emit_error(&htype, &min_balance, e))?;

	generate_vc(req, &htype, &q_min_balance, holding_date)
		.map_err(|e| emit_error(&htype, &min_balance, e))
}

fn prepare_min_balance(
	htype: &AmountHoldingTimeType,
	min_balance: &ParameterString,
) -> Result<String> {
	vec_to_string(min_balance.to_vec())
		.map_err(|_| emit_error(htype, min_balance, ErrorDetail::ParseError))
}

#[derive(Clone)]
// Represents an individual account that may or not be holding the desired token amount
struct Account {
	network: Web3Network,
	token: Option<String>,
	address: String,
}

// TODO: unit test?
fn prepare_accounts(
	identities: &Vec<IdentityNetworkTuple>,
	htype: &AmountHoldingTimeType,
) -> Vec<Account> {
	transpose_identity(identities)
		.into_iter()
		.flat_map(|(network, addresses)| -> Vec<Account> {
			let token = match_token_address(htype, &network);
			addresses
				.into_iter()
				.map(move |address| Account { network, token: token.clone(), address })
				.collect()
		})
		.collect()
}

// Represents the outcome of a holding query for a given date
//  Ok(true)  => positive: user/account is holding (uninterrupted) since the given date
//  Ok(false) => negative: user/account did not hold (at some point) since the given date
//  Err(...)  => inconclusive: query failed
type QueryOutcome = core::result::Result<bool, ErrorDetail>;

fn is_positive(outcome: &QueryOutcome) -> bool {
	matches!(outcome, Ok(true))
}

fn is_negative(outcome: &QueryOutcome) -> bool {
	matches!(outcome, Ok(false))
}

fn is_inconclusive(outcome: &QueryOutcome) -> bool {
	matches!(outcome, Err(_))
}

// Check against the data provider whether a single account has been holding since the given date.
fn account_is_holding(
	client: &mut AchainableClient,
	q_min_balance: &String,
	account: &Account,
	date: &str,
) -> QueryOutcome {
	let holding = ParamsBasicTypeWithAmountHolding::new(
		&account.network,
		q_min_balance.to_string(),
		date.to_string(),
		account.token.clone(),
	);
	return client.is_holder(account.address.as_str(), holding).map_err(|e| {
		error!("Assertion HoldingTime request error: {:?}", e);
		e.into_error_detail()
	});
}

// Check against the data provider whether any of the given accounts has been holding since the given date.
// If at least one positive outcome is found, the accounts that yielded a (conclusive) negative outcome are eliminated.
fn holding_time_search_step(
	client: &mut AchainableClient,
	q_min_balance: &String,
	accounts: Vec<Account>,
	date: &str,
) -> (QueryOutcome, Vec<Account>) {
	// Check all remaining identities on the given date
	let outcomes: Vec<QueryOutcome> = accounts
		.iter()
		.map(|account| account_is_holding(client, q_min_balance, account, date))
		.collect();

	// If any positive result is found:
	//   - Discard all identities that yielded a _negative_ result
	//     - but KEEP the ones that yielded error; they may still be relevant!
	//   - Return the remaining identities with a positive value to continue the search
	if outcomes.iter().any(is_positive) {
		let new_accounts = accounts
			.into_iter()
			.zip(outcomes.iter())
			.filter_map(|(account, outcome)| (!is_negative(outcome)).then_some(account))
			.collect();
		return (Ok(true), new_accounts);
	}

	/*
	 * If any error is found:
	 *   - The search is stuck; bubble the error
	 *     TODO: retry?
	 *
	 * Otherwise (all results were negative):
	 *   - Keep all identities
	 *   - Return with a negative result and continue the search
	 */
	let outcome = match outcomes.into_iter().find(is_inconclusive) {
		Some(Err(e)) => Err(e),
		_ => Ok(false),
	};

	(outcome, accounts)
}

const ASSERTION_DATE_LEN: usize = 15;
const ASSERTION_FROM_DATE: [&str; ASSERTION_DATE_LEN] = [
	"2017-01-01",
	"2017-07-01",
	"2018-01-01",
	"2018-07-01",
	"2019-01-01",
	"2019-07-01",
	"2020-01-01",
	"2020-07-01",
	"2021-01-01",
	"2021-07-01",
	"2022-01-01",
	"2022-07-01",
	"2023-01-01",
	"2023-07-01",
	// In order to address the issue of the community encountering a false query for WBTC in
	// November, the product team feels that adding this date temporarily solves this problem.
	"2023-12-01",
];

// Search against the data provider for the holding time of the user's longest holding account.
// Return the date if successful, `None` if none of the accounts is currently holding.
fn search_holding_date(
	data_provider_config: &DataProviderConfig,
	mut accounts: Vec<Account>,
	q_min_balance: &String,
) -> core::result::Result<Option<&'static str>, ErrorDetail> {
	let mut client = AchainableClient::new(data_provider_config);

	let mut pred = |date: &&str| {
		let (outcome, new_accounts) =
			holding_time_search_step(&mut client, q_min_balance, accounts.clone(), date);
		accounts = new_accounts;
		outcome.map(|is_holding| !is_holding) // negated to match the partition_point API
	};

	partition_point(ASSERTION_FROM_DATE.as_ref(), &mut pred).map(|index| {
		if index < ASSERTION_DATE_LEN {
			Some(ASSERTION_FROM_DATE[index])
		} else {
			None
		}
	})
}

fn generate_vc(
	req: &AssertionBuildRequest,
	htype: &AmountHoldingTimeType,
	q_min_balance: &str,
	holding_date: Option<&str>,
) -> core::result::Result<Credential, ErrorDetail> {
	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.update_amount_holding_time_credential(
				htype,
				holding_date.is_some(),
				q_min_balance,
				holding_date.unwrap_or("2017-01-01"),
			);

			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(e.into_error_detail())
		},
	}
}

/// Utils Functions
fn emit_error(
	htype: &AmountHoldingTimeType,
	min_balance: &ParameterString,
	e: ErrorDetail,
) -> Error {
	let a = if *htype == AmountHoldingTimeType::LIT {
		Assertion::A4(min_balance.clone())
	} else if *htype == AmountHoldingTimeType::DOT {
		Assertion::A7(min_balance.clone())
	} else if *htype == AmountHoldingTimeType::WBTC {
		Assertion::A10(min_balance.clone())
	} else {
		Assertion::A11(min_balance.clone())
	};

	Error::RequestVCFailed(a, e)
}

fn match_token_address(htype: &AmountHoldingTimeType, network: &Web3Network) -> Option<String> {
	match htype {
		AmountHoldingTimeType::WBTC => Some(WBTC_TOKEN_ADDRESS.into()),
		AmountHoldingTimeType::LIT => {
			if *network == Web3Network::Ethereum {
				Some(LIT_TOKEN_ADDRESS.into())
			} else {
				None
			}
		},
		_ => None,
	}
}

fn partition_point<T, E, P>(vector: &[T], pred: &mut P) -> core::result::Result<usize, E>
where
	P: FnMut(&T) -> core::result::Result<bool, E>,
{
	let mut trapped_error: Option<E> = None;
	let wrapped_pred = |element: &T| -> bool {
		if trapped_error.is_some() {
			return true;
		}
		match pred(element) {
			Ok(result) => result,
			Err(error) => {
				trapped_error = Some(error);
				true
			},
		}
	};
	let index = vector.partition_point(wrapped_pred);
	match trapped_error {
		Some(error) => Err(error),
		None => Ok(index),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use lc_mock_server::run;
	use litentry_primitives::{AmountHoldingTimeType, Web3Network};

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();
		let mut data_provider_config = DataProviderConfig::new().unwrap();
		data_provider_config.set_achainable_url(url).unwrap();
		data_provider_config
	}

	#[test]
	fn do_build_lit_works() {
		let data_provider_config = init();

		let htype = AmountHoldingTimeType::LIT;
		let network = Web3Network::Litentry;
		let accounts: Vec<Account> = vec![Account {
			address: "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C11".to_string(),
			network,
			token: match_token_address(&htype, &network),
		}];

		let q_min_balance = "10".to_string();

		let holding_date =
			search_holding_date(&data_provider_config, accounts, &q_min_balance).unwrap();
		assert!(holding_date.is_some());
	}

	#[test]
	fn do_build_dot_works() {
		let data_provider_config = init();

		let htype = AmountHoldingTimeType::DOT;
		let network = Web3Network::Polkadot;
		let accounts = vec![Account {
			address: "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C13".to_string(),
			network,
			token: match_token_address(&htype, &network),
		}];
		let q_min_balance = "10".to_string();

		let holding_date =
			search_holding_date(&data_provider_config, accounts, &q_min_balance).unwrap();
		assert!(holding_date.is_some());
	}

	#[test]
	fn do_build_wbtc_works() {
		let data_provider_config = init();

		let htype = AmountHoldingTimeType::WBTC;
		let network = Web3Network::Ethereum;
		let accounts = vec![
			"0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C11",
			"0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C12",
		]
		.iter()
		.map(|address| Account {
			address: address.to_string(),
			network,
			token: match_token_address(&htype, &network),
		})
		.collect();

		let q_min_balance = "10".to_string();

		let holding_date =
			search_holding_date(&data_provider_config, accounts, &q_min_balance).unwrap();
		assert!(holding_date.is_some());
	}

	#[test]
	fn do_build_non_hold_works() {
		let data_provider_config = init();

		let htype = AmountHoldingTimeType::LIT;
		let network = Web3Network::Ethereum;
		let accounts = vec![Account {
			address: "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C14".to_string(),
			network,
			token: match_token_address(&htype, &network),
		}];

		let q_min_balance = "10".to_string();

		let holding_date =
			search_holding_date(&data_provider_config, accounts, &q_min_balance).unwrap();
		assert!(holding_date.is_none());
	}

	#[test]
	fn match_token_address_works() {
		let htype = AmountHoldingTimeType::WBTC;
		let network = Web3Network::Ethereum;
		let ret = match_token_address(&htype, &network);
		assert_eq!(ret, Some(WBTC_TOKEN_ADDRESS.into()));

		let htype = AmountHoldingTimeType::LIT;
		let ret = match_token_address(&htype, &network);
		assert_eq!(ret, Some(LIT_TOKEN_ADDRESS.into()));
	}
}
