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

use super::IMP;
use crate::{
	command_utils::{get_chain_api, *},
	Cli, CliResult, CliResultOk,
};
use codec::{Decode, Encode};
use ita_stf::{helpers::get_expected_raw_message, Web3Network};
use itc_rpc_client::direct_client::DirectApi;
use itp_sgx_crypto::ShieldingCryptoEncrypt;
use itp_stf_primitives::types::ShardIdentifier;
use itp_types::AccountId;
use litentry_primitives::{Identity, LitentryMultiSignature, Web3CommonValidationData};
use log::*;
use sp_application_crypto::Pair;
use sp_core::sr25519 as sr25519_core;
use substrate_api_client::{ac_compose_macros::compose_extrinsic, SubmitAndWatch, XtStatus};

#[derive(Parser)]
pub struct LinkIdentityCommand {
	/// AccountId in ss58check format
	account: String,
	/// Identity to be created, in did form
	did: String,
	/// Delegate signer for the account
	#[clap(short = 'd')]
	delegate: Option<String>,
	/// Web3 networks for the linking
	#[clap(num_args = 0.., value_delimiter = ',')]
	networks: Vec<String>,
}

impl LinkIdentityCommand {
	pub(crate) fn run(&self, cli: &Cli) -> CliResult {
		let mut chain_api = get_chain_api(cli);

		let direct_api = get_worker_api_direct(cli);
		let mrenclave = direct_api.get_state_mrenclave().unwrap();
		let shard = ShardIdentifier::decode(&mut &mrenclave[..]).unwrap();
		let nonce = direct_api
			.get_next_nonce(&shard, &self.get_primary_account_id().into())
			.unwrap();

		let signer = self.get_signer();
		let who: sr25519_core::Pair = get_pair_from_str(&self.account).into();
		chain_api.set_signer(signer.into());

		let (identity, encrypted_identity) = self.encrypt_identity(cli);
		let (encrypted_web3network, encrypted_validation_data) =
			self.prepare_validation_data(&identity, cli, nonce);

		let xt = compose_extrinsic!(
			chain_api,
			IMP,
			"link_identity",
			shard,
			who.public().0,
			encrypted_identity,
			encrypted_validation_data,
			encrypted_web3network
		);

		let tx_hash = chain_api.submit_and_watch_extrinsic_until(xt, XtStatus::Broadcast).unwrap();
		info!("[+] LinkIdentityCommand TrustedOperation got finalized. Hash: {:?}", tx_hash);

		Ok(CliResultOk::None)
	}

	fn get_primary_account_id(&self) -> AccountId {
		let account: sr25519_core::Pair = get_pair_from_str(&self.account).into();
		account.public().into()
	}

	fn get_signer(&self) -> sr25519_core::Pair {
		let account = self.delegate.as_ref().unwrap_or(&self.account);
		get_pair_from_str(account).into()
	}

	fn encrypt_identity(&self, cli: &Cli) -> (Identity, Vec<u8>) {
		let identity = Identity::from_did(&self.did).unwrap();
		let tee_shielding_key = get_shielding_key(cli).unwrap();
		let encrypted_identity = tee_shielding_key.encrypt(&identity.encode()).unwrap();
		(identity, encrypted_identity)
	}

	fn prepare_validation_data(
		&self,
		identity: &Identity,
		cli: &Cli,
		nonce: u32,
	) -> (Vec<u8>, Vec<u8>) {
		if identity.is_web3() {
			let who_identity = Identity::from(self.get_signer().public());
			let validation_payload = get_expected_raw_message(&who_identity, identity, nonce);
			let web3network: Vec<Web3Network> = self
				.networks
				.iter()
				.map(|n| n.as_str().try_into().expect("cannot convert to Web3Network"))
				.collect();
			let tee_shielding_key = get_shielding_key(cli).unwrap();
			let encrypted_web3network = tee_shielding_key.encrypt(&web3network.encode()).unwrap();

			let identity_account_id = identity.to_native_account().unwrap();
			let identity_public_key = format!("{}", identity_account_id);
			let identity_pair: sr25519_core::Pair = get_pair_from_str(&identity_public_key).into();

			let signature: LitentryMultiSignature = identity_pair.sign(&validation_payload).into();
			let web3common = Web3CommonValidationData {
				message: validation_payload.try_into().unwrap(),
				signature,
			};
			let validation_data = litentry_primitives::ValidationData::Web3(
				litentry_primitives::Web3ValidationData::Substrate(web3common),
			);
			let encrypted_validation_data =
				tee_shielding_key.encrypt(&validation_data.encode()).unwrap();

			(encrypted_web3network, encrypted_validation_data)
		} else {
			(vec![0], vec![0])
		}
	}
}
