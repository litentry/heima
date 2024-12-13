/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use crate::error::{Error, ServiceResult};
use codec::Encode;
use itp_node_api::api_client::{AccountApi, ParentchainApi};
use itp_settings::worker::REGISTERING_FEE_FACTOR_FOR_INIT_FUNDS;
use itp_types::{
	parentchain::{AccountId, Balance, ParentchainId},
	Moment,
};
use log::*;
use sp_core::{
	crypto::{AccountId32, Ss58Codec},
	Pair,
};
use sp_keyring::AccountKeyring;
use sp_runtime::{MultiAddress, Saturating};
use std::{thread, time::Duration};
use substrate_api_client::{
	ac_compose_macros::compose_extrinsic, ac_primitives::Bytes, extrinsic::BalancesExtrinsics,
	GetBalance, GetStorage, GetTransactionPayment, SubmitAndWatch, XtStatus,
};

const SGX_RA_PROOF_MAX_LEN: usize = 5000;
const MAX_URL_LEN: usize = 256;
/// Information about the enclave on-chain account.
pub trait EnclaveAccountInfo {
	fn free_balance(&self) -> ServiceResult<Balance>;
}

pub struct EnclaveAccountInfoProvider {
	node_api: ParentchainApi,
	account_id: AccountId32,
}

impl EnclaveAccountInfo for EnclaveAccountInfoProvider {
	fn free_balance(&self) -> ServiceResult<Balance> {
		self.node_api.get_free_balance(&self.account_id).map_err(|e| e.into())
	}
}

impl EnclaveAccountInfoProvider {
	pub fn new(node_api: ParentchainApi, account_id: AccountId32) -> Self {
		EnclaveAccountInfoProvider { node_api, account_id }
	}
}

/// evaluate if the enclave should have more funds and how much more
/// in --dev mode: let Alice pay for missing funds
/// in production mode: wait for manual transfer before continuing
pub fn setup_reasonable_account_funding(
	api: &ParentchainApi,
	accountid: &AccountId32,
	parentchain_id: ParentchainId,
	is_development_mode: bool,
) -> ServiceResult<()> {
	loop {
		let needed = estimate_funds_needed_to_run_for_a_while(api, accountid, parentchain_id)?;
		let free = api.get_free_balance(accountid)?;
		let missing_funds = needed.saturating_sub(free);

		if missing_funds < needed * 2 / 3 {
			return Ok(())
		}

		if is_development_mode {
			info!("[{:?}] Alice will grant {:?} to {:?}", parentchain_id, missing_funds, accountid);
			bootstrap_funds_from_alice(api, accountid, missing_funds)?;
		} else {
			error!(
				"[{:?}] Enclave account needs funding. please send at least {:?} to {:?}",
				parentchain_id, missing_funds, accountid
			);
			thread::sleep(Duration::from_secs(10));
		}
	}
}

fn estimate_funds_needed_to_run_for_a_while(
	api: &ParentchainApi,
	accountid: &AccountId32,
	parentchain_id: ParentchainId,
) -> ServiceResult<Balance> {
	let existential_deposit = api.get_existential_deposit()?;
	info!("[{:?}] Existential deposit is = {:?}", parentchain_id, existential_deposit);

	let mut min_required_funds: Balance = existential_deposit;

	let transfer_fee = estimate_transfer_fee(api)?;
	info!("[{:?}] a single transfer costs {:?}", parentchain_id, transfer_fee);
	min_required_funds += 1000 * transfer_fee;

	// TODO(Litentry P-628): shall we charge RA fee?
	info!("[{:?}] not adding RA fees for now", parentchain_id);

	info!(
		"[{:?}] we estimate the funding requirement for the primary validateer (worst case) to be {:?}",
		parentchain_id,
		min_required_funds
	);
	Ok(min_required_funds)
}

pub fn estimate_fee(api: &ParentchainApi, encoded_extrinsic: Vec<u8>) -> Result<u128, Error> {
	let reg_fee_details = api.get_fee_details(&encoded_extrinsic.into(), None)?;
	match reg_fee_details {
		Some(details) => match details.inclusion_fee {
			Some(fee) => Ok(fee.inclusion_fee()),
			None => Err(Error::Custom(
				"Inclusion fee for the registration of the enclave is None!".into(),
			)),
		},
		None =>
			Err(Error::Custom("Fee Details for the registration of the enclave is None !".into())),
	}
}

/// Alice sends some funds to the account. only for dev chains testing
fn bootstrap_funds_from_alice(
	api: &ParentchainApi,
	accountid: &AccountId32,
	funding_amount: u128,
) -> Result<(), Error> {
	let alice = AccountKeyring::Alice.pair();
	let alice_acc = AccountId32::from(*alice.public().as_array_ref());

	let alice_free = api.get_free_balance(&alice_acc)?;
	info!("    Alice's free balance = {:?}", alice_free);
	let nonce = api.get_account_next_index(&alice_acc)?;
	info!("    Alice's Account Nonce is {}", nonce);

	if funding_amount > alice_free {
		println!(
            "funding amount is too high: please change EXISTENTIAL_DEPOSIT_FACTOR_FOR_INIT_FUNDS ({:?})",
            funding_amount
        );
		return Err(Error::ApplicationSetup)
	}

	let mut alice_signer_api = api.clone();
	alice_signer_api.set_signer(alice.into());

	println!("[+] send extrinsic: bootstrap funding Enclave from Alice's funds");
	let xt = alice_signer_api
		.balance_transfer_allow_death(MultiAddress::Id(accountid.clone()), funding_amount);
	let xt_report = alice_signer_api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized)?;
	info!(
		"[<] L1 extrinsic success. extrinsic hash: {:?} / status: {:?}",
		xt_report.extrinsic_hash, xt_report.status
	);
	// Verify funds have arrived.
	let free_balance = alice_signer_api.get_free_balance(accountid);
	trace!("TEE's NEW free balance = {:?}", free_balance);

	Ok(())
}

/// precise estimation of a single transfer fee
pub fn estimate_transfer_fee(api: &ParentchainApi) -> Result<Balance, Error> {
	let encoded_xt: Bytes = api
		.balance_transfer_allow_death(AccountId::from([0u8; 32]).into(), 1000000000000)
		.encode()
		.into();
	let tx_fee = api.get_fee_details(&encoded_xt, None).unwrap().unwrap().inclusion_fee.unwrap();
	let transfer_fee = tx_fee.base_fee + tx_fee.len_fee + tx_fee.adjusted_weight_fee;
	Ok(transfer_fee)
}
