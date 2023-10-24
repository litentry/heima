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

use crate::{
	trusted_base_cli::commands::{
		balance::BalanceCommand,
		get_storage::GetStorageCommand,
		litentry::{
			id_graph_stats::IDGraphStats, link_identity::LinkIdentityCommand,
			request_vc::RequestVcCommand,
			send_erroneous_parentchain_call::SendErroneousParentchainCallCommand,
			set_user_shielding_key::SetUserShieldingKeyCommand,
			user_shielding_key::UserShieldingKeyCommand,
		},
		nonce::NonceCommand,
		set_balance::SetBalanceCommand,
		transfer::TransferCommand,
		unshield_funds::UnshieldFundsCommand,
	},
	trusted_cli::TrustedCli,
	trusted_command_utils::get_keystore_path,
	Cli, CliResult, CliResultOk,
};
use log::*;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, Pair};
use substrate_client_keystore::{KeystoreExt, LocalKeystore};

use self::commands::litentry::id_graph::IDGraphCommand;

mod commands;

#[derive(Subcommand)]
pub enum TrustedBaseCommand {
	/// generates a new incognito account for the given shard
	NewAccount,

	/// lists all incognito accounts in a given shard
	ListAccounts,

	/// send funds from one incognito account to another
	Transfer(TransferCommand),

	/// ROOT call to set some account balance to an arbitrary number
	SetBalance(SetBalanceCommand),

	/// query balance for incognito account in keystore
	Balance(BalanceCommand),

	/// Transfer funds from an incognito account to an parentchain account
	UnshieldFunds(UnshieldFundsCommand),

	/// gets the nonce of a given account, taking the pending trusted calls
	/// in top pool in consideration
	Nonce(NonceCommand),

	// Litentry's commands below
	/// query a given user's shielding key
	UserShieldingKey(UserShieldingKeyCommand),

	/// set a given user's shielding key
	SetUserShieldingKey(SetUserShieldingKeyCommand),

	/// retrieve the sidechain's raw storage - should only work for non-prod
	GetStorage(GetStorageCommand),

	/// send an erroneous parentchain call intentionally, only used in tests
	SendErroneousParentchainCall(SendErroneousParentchainCallCommand),

	/// Disabled for now: get count of all keys account + identity in the IDGraphs
	IDGraphStats(IDGraphStats),

	/// Link the given identity to the prime identity, with specified networks
	LinkIdentity(LinkIdentityCommand),

	/// The IDGraph for the given identity
	IDGraph(IDGraphCommand),

	/// Request VC
	RequestVc(RequestVcCommand),
}

impl TrustedBaseCommand {
	pub fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		match self {
			TrustedBaseCommand::NewAccount => new_account(trusted_cli, cli),
			TrustedBaseCommand::ListAccounts => list_accounts(trusted_cli, cli),
			TrustedBaseCommand::Transfer(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::SetBalance(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::Balance(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::UnshieldFunds(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::Nonce(cmd) => cmd.run(cli, trusted_cli),
			// Litentry's commands below
			TrustedBaseCommand::UserShieldingKey(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::SetUserShieldingKey(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::GetStorage(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::SendErroneousParentchainCall(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::IDGraphStats(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::LinkIdentity(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::IDGraph(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::RequestVc(cmd) => cmd.run(cli, trusted_cli),
		}
	}
}

fn new_account(trusted_args: &TrustedCli, cli: &Cli) -> CliResult {
	let store = LocalKeystore::open(get_keystore_path(trusted_args, cli), None).unwrap();
	let key: sr25519::AppPair = store.generate().unwrap();
	drop(store);
	info!("new account {}", key.public().to_ss58check());
	let key_str = key.public().to_ss58check();
	println!("{}", key_str);

	Ok(CliResultOk::PubKeysBase58 { pubkeys_sr25519: Some(vec![key_str]), pubkeys_ed25519: None })
}

fn list_accounts(trusted_args: &TrustedCli, cli: &Cli) -> CliResult {
	let store = LocalKeystore::open(get_keystore_path(trusted_args, cli), None).unwrap();
	info!("sr25519 keys:");
	for pubkey in store.public_keys::<sr25519::AppPublic>().unwrap().into_iter() {
		println!("{}", pubkey.to_ss58check());
	}
	info!("ed25519 keys:");
	let pubkeys: Vec<String> = store
		.public_keys::<ed25519::AppPublic>()
		.unwrap()
		.into_iter()
		.map(|pubkey| pubkey.to_ss58check())
		.collect();
	for pubkey in &pubkeys {
		println!("{}", pubkey);
	}
	drop(store);

	Ok(CliResultOk::PubKeysBase58 { pubkeys_sr25519: None, pubkeys_ed25519: Some(pubkeys) })
}
