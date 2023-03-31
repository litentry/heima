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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

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
use crate::{add_now_to_from_dates, Error, Result};
use itp_stf_primitives::types::ShardIdentifier;
use itp_types::AccountId;
use itp_utils::stringify::account_id_to_string;
use lc_credentials::{format_assertion_to_date, Credential};
use lc_data_providers::graphql::{
	GraphQLClient, VerifiedCredentialsIsHodlerIn, VerifiedCredentialsNetwork,
};
use litentry_primitives::{Assertion, Identity, ParentchainBalance, ParentchainBlockNumber};
use log::*;
use std::{
	collections::{HashMap, HashSet},
	string::{String, ToString},
	vec::Vec,
};

// ERC20 LIT token address
const LIT_TOKEN_ADDRESS: &str = "0xb59490aB09A0f526Cc7305822aC65f2Ab12f9723";
const VC_SUBJECT_DESCRIPTION: &str =
	"Check whether any of the linked accounts hold a minimum amount of LIT NOW";
const VC_SUBJECT_TYPE: &str = "LIT Holder";

pub fn build(
	identities: Vec<Identity>,
	min_balance: ParentchainBalance,
	shard: &ShardIdentifier,
	who: &AccountId,
	bn: ParentchainBlockNumber,
) -> Result<Credential> {
	debug!(
		"Assertion A4 build, who: {:?}, bn: {}, identities: {:?}",
		account_id_to_string(&who),
		bn,
		identities
	);

	let evm_min_balance = (min_balance / (10 ^ 18)) as f64;
	let substrate_min_balance = (min_balance / (10 ^ 12)) as f64;

	let mut client = GraphQLClient::new();
	let mut networks: HashMap<VerifiedCredentialsNetwork, HashSet<String>> = HashMap::new();

	identities.iter().for_each(|identity| {
		match identity {
			Identity::Substrate { network, address } => {
				let mut address = account_id_to_string(address.as_ref());
				address.insert_str(0, "0x");

				if_match_networks_collect_address(&mut networks, (*network).into(), address);
			},
			Identity::Evm { network, address } => {
				let mut address = account_id_to_string(address.as_ref());
				address.insert_str(0, "0x");

				if_match_networks_collect_address(&mut networks, (*network).into(), address);
			},
			_ => {},
		};
	});

	// Including 8 items from NOW -> 2017-01-01
	// Because it is necessary to determine the continuous holding, the reverse order query is started from NOW.
	// As long as there are terminals, this interval is taken as the result.
	// query from NOW -> 2023 -> ... -> 2017
	let now = format_assertion_to_date();
	let mut hold_from_date = now.clone();

	let dates = add_now_to_from_dates(&now);
	debug!("Assertion A4 dates: {:?}", dates);

	// is_hold default value is true
	let mut is_hold = networks.is_empty();
	for (verified_network, addresses) in networks {
		// If find a satisfying interval from a network, do not need to query the remaining networks.
		if hold_from_date != now {
			break
		}

		let addresses: Vec<String> = addresses.into_iter().collect();
		let token_address = if verified_network == VerifiedCredentialsNetwork::Ethereum {
			LIT_TOKEN_ADDRESS
		} else {
			""
		};
		let q_min_balance = if verified_network == VerifiedCredentialsNetwork::Litentry
			|| verified_network == VerifiedCredentialsNetwork::Litmus
			|| verified_network == VerifiedCredentialsNetwork::LitentryRococo
		{
			substrate_min_balance
		} else {
			// Ethereum network
			evm_min_balance
		};

		for from_date in dates.iter() {
			let vch = VerifiedCredentialsIsHodlerIn::new(
				addresses.clone(),
				from_date.to_string(),
				verified_network.clone(),
				token_address.to_string(),
				q_min_balance,
			);
			match client.check_verified_credentials_is_hodler(vch) {
				Ok(is_hodler_out) => {
					for hodler in is_hodler_out.verified_credentials_is_hodler.iter() {
						is_hold = is_hold && hodler.is_hodler;
					}
				},
				Err(e) => error!(
					"Assertion A4 request check_verified_credentials_is_hodler error: {:?}",
					e
				),
			}

			// If Continuous holding， update hold_from_date, or just break
			if is_hold {
				hold_from_date = from_date.to_string();
			} else {
				break
			}
		}
	}

	match Credential::new_default(who, &shard.clone(), bn) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.add_subject_info(VC_SUBJECT_DESCRIPTION, VC_SUBJECT_TYPE);
			credential_unsigned.update_holder(is_hold, min_balance, &hold_from_date, &now);

			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::A4(min_balance), e.to_error_detail()))
		},
	}
}

fn if_match_networks_collect_address(
	networks: &mut HashMap<VerifiedCredentialsNetwork, HashSet<String>>,
	verified_network: VerifiedCredentialsNetwork,
	address: String,
) {
	if matches!(
		verified_network,
		VerifiedCredentialsNetwork::Litentry
			| VerifiedCredentialsNetwork::Litmus
			| VerifiedCredentialsNetwork::LitentryRococo
			| VerifiedCredentialsNetwork::Ethereum
	) {
		match networks.get_mut(&verified_network) {
			Some(set) => {
				set.insert(address);
			},
			None => {
				let mut set = HashSet::new();
				set.insert(address);

				networks.insert(verified_network.clone(), set);
			},
		}
	}
}
