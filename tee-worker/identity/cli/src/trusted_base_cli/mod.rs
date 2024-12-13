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

#[cfg(feature = "development")]
use crate::trusted_base_cli::commands::litentry::clean_id_graphs::CleanIDGraphsCommand;
#[cfg(feature = "development")]
use crate::trusted_base_cli::commands::litentry::remove_identity::RemoveIdentityCommand;
use crate::{
	trusted_base_cli::commands::{
		balance::BalanceCommand,
		get_shard::GetShardCommand,
		litentry::{
			get_storage::GetStorageCommand, link_identity::LinkIdentityCommand,
			request_vc::RequestVcCommand,
			send_erroneous_parentchain_call::SendErroneousParentchainCallCommand,
		},
		nonce::NonceCommand,
		set_balance::SetBalanceCommand,
		transfer::TransferCommand,
	},
	trusted_cli::TrustedCli,
	trusted_command_utils::get_keystore_path,
	Cli, CliResult, CliResultOk, ED25519_KEY_TYPE, SR25519_KEY_TYPE,
};
use log::*;
use sp_core::crypto::Ss58Codec;
use sp_keystore::Keystore;
use substrate_client_keystore::LocalKeystore;

use self::commands::litentry::id_graph::IDGraphCommand;

pub mod commands;

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

	/// gets the nonce of a given account, taking the pending trusted calls
	/// in top pool in consideration
	Nonce(NonceCommand),

	/// get shard for this worker
	GetShard(GetShardCommand),

	// Litentry's commands below
	/// retrieve the sidechain's raw storage - should only work for non-prod
	GetStorage(GetStorageCommand),

	/// send an erroneous parentchain call intentionally, only used in tests
	SendErroneousParentchainCall(SendErroneousParentchainCallCommand),

	/// Link the given identity to the prime identity, with specified networks
	LinkIdentity(LinkIdentityCommand),

	/// The IDGraph for the given identity
	IDGraph(IDGraphCommand),

	/// Request VC
	RequestVc(RequestVcCommand),

	/// Remove Identity from the prime identity
	#[cfg(feature = "development")]
	RemoveIdentity(RemoveIdentityCommand),

	// Remove all id_graph from storage
	#[cfg(feature = "development")]
	CleanIDGraphs(CleanIDGraphsCommand),
}

impl TrustedBaseCommand {
	pub fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		match self {
			TrustedBaseCommand::NewAccount => new_account(trusted_cli, cli),
			TrustedBaseCommand::ListAccounts => list_accounts(trusted_cli, cli),
			TrustedBaseCommand::Transfer(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::SetBalance(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::Balance(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::Nonce(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::GetShard(cmd) => cmd.run(cli, trusted_cli),
			// Litentry's commands below
			TrustedBaseCommand::GetStorage(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::SendErroneousParentchainCall(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::LinkIdentity(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::IDGraph(cmd) => cmd.run(cli, trusted_cli),
			TrustedBaseCommand::RequestVc(cmd) => cmd.run(cli, trusted_cli),
			#[cfg(feature = "development")]
			TrustedBaseCommand::RemoveIdentity(cmd) => cmd.run(cli, trusted_cli),
			#[cfg(feature = "development")]
			TrustedBaseCommand::CleanIDGraphs(cmd) => cmd.run(cli, trusted_cli),
		}
	}
}

fn new_account(trusted_args: &TrustedCli, cli: &Cli) -> CliResult {
	let store = LocalKeystore::open(get_keystore_path(trusted_args, cli), None).unwrap();
	let key = LocalKeystore::sr25519_generate_new(&store, SR25519_KEY_TYPE, None).unwrap();
	drop(store);
	info!("new account {}", key.to_ss58check());
	let key_str = key.to_ss58check();
	println!("{}", key_str);

	Ok(CliResultOk::PubKeysBase58 { pubkeys_sr25519: Some(vec![key_str]), pubkeys_ed25519: None })
}

fn list_accounts(trusted_args: &TrustedCli, cli: &Cli) -> CliResult {
	let store = LocalKeystore::open(get_keystore_path(trusted_args, cli), None).unwrap();
	info!("sr25519 keys:");
	for pubkey in store.sr25519_public_keys(SR25519_KEY_TYPE).into_iter() {
		println!("{}", pubkey.to_ss58check());
	}
	info!("ed25519 keys:");
	let pubkeys: Vec<String> = store
		.ed25519_public_keys(ED25519_KEY_TYPE)
		.into_iter()
		.map(|pubkey| pubkey.to_ss58check())
		.collect();
	for pubkey in &pubkeys {
		println!("{}", pubkey);
	}
	drop(store);

	Ok(CliResultOk::PubKeysBase58 { pubkeys_sr25519: None, pubkeys_ed25519: Some(pubkeys) })
}
