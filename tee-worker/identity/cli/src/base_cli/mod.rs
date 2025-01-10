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
	base_cli::commands::{
		balance::BalanceCommand, faucet::FaucetCommand, listen::ListenCommand, litentry::*,
		register_tcb_info::RegisterTcbInfoCommand, system_version::SystemVersionCommand,
		transfer::TransferCommand,
	},
	command_utils::*,
	Cli, CliResult, CliResultOk, ED25519_KEY_TYPE, SR25519_KEY_TYPE,
};
use base58::ToBase58;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use itc_rpc_client::direct_client::DirectApi;
use itp_node_api::api_client::PalletTeebagApi;
use itp_types::WorkerType;
use sp_core::crypto::Ss58Codec;
use sp_keystore::Keystore;
use std::{
	path::PathBuf,
	time::{Duration, UNIX_EPOCH},
};
use substrate_client_keystore::LocalKeystore;

mod commands;

#[derive(Subcommand)]
pub enum BaseCommand {
	/// query parentchain balance for AccountId
	Balance(BalanceCommand),

	/// generates a new account for the integritee chain in your local keystore
	NewAccount,

	/// lists all accounts in your local keystore for the integritee chain
	ListAccounts,

	/// query node metadata and print it as json to stdout
	PrintMetadata,

	/// query sgx-runtime metadata and print it as json to stdout
	PrintSgxMetadata,

	/// send some bootstrapping funds to supplied account(s)
	Faucet(FaucetCommand),

	/// transfer funds from one parentchain account to another
	Transfer(TransferCommand),

	/// query enclave registry and list all workers
	ListWorkers,

	/// listen to parentchain events
	Listen(ListenCommand),

	/// Register TCB info for FMSPC
	RegisterTcbInfo(RegisterTcbInfoCommand),

	/// System version
	SystemVersion(SystemVersionCommand),

	// Litentry's commands below
	/// query sgx-runtime metadata and print the raw (hex-encoded) metadata to stdout
	/// we could have added a parameter like `--raw` to `PrintSgxMetadata`, but
	/// we want to keep our changes isolated
	PrintSgxMetadataRaw,

	/// create idenity graph
	LinkIdentity(LinkIdentityCommand),

	/// get the IDGraph hash of the given identity
	IDGraphHash(IDGraphHashCommand),

	/// Deactivate Identity
	DeactivateIdentity(DeactivateIdentityCommand),

	/// Activate identity
	ActivateIdentity(ActivateIdentityCommand),

	/// Shield text
	ShieldText(ShieldTextCommand),
}

impl BaseCommand {
	pub fn run(&self, cli: &Cli) -> CliResult {
		match self {
			BaseCommand::Balance(cmd) => cmd.run(cli),
			BaseCommand::NewAccount => new_account(),
			BaseCommand::ListAccounts => list_accounts(),
			BaseCommand::PrintMetadata => print_metadata(cli),
			BaseCommand::PrintSgxMetadata => print_sgx_metadata(cli),
			BaseCommand::Faucet(cmd) => cmd.run(cli),
			BaseCommand::Transfer(cmd) => cmd.run(cli),
			BaseCommand::ListWorkers => list_workers(cli),
			BaseCommand::Listen(cmd) => cmd.run(cli),
			BaseCommand::RegisterTcbInfo(cmd) => cmd.run(cli),
			BaseCommand::SystemVersion(cmd) => cmd.run(cli),
			// Litentry's commands below
			BaseCommand::PrintSgxMetadataRaw => print_sgx_metadata_raw(cli),
			BaseCommand::LinkIdentity(cmd) => cmd.run(cli),
			BaseCommand::IDGraphHash(cmd) => cmd.run(cli),
			BaseCommand::DeactivateIdentity(cmd) => cmd.run(cli),
			BaseCommand::ActivateIdentity(cmd) => cmd.run(cli),
			BaseCommand::ShieldText(cmd) => cmd.run(cli),
		}
	}
}

fn new_account() -> CliResult {
	let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
	let key: sp_core::sr25519::Public =
		LocalKeystore::sr25519_generate_new(&store, SR25519_KEY_TYPE, None).unwrap();
	let key_base58 = key.to_ss58check();
	drop(store);
	println!("0x{}", hex::encode(key.0));
	Ok(CliResultOk::PubKeysBase58 {
		pubkeys_sr25519: Some(vec![key_base58]),
		pubkeys_ed25519: None,
	})
}

fn list_accounts() -> CliResult {
	let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
	println!("sr25519 keys:");
	let mut keys_sr25519 = vec![];
	for pubkey in store.sr25519_public_keys(SR25519_KEY_TYPE).into_iter() {
		let key_ss58 = pubkey.to_ss58check();
		println!("0x{}", hex::encode(pubkey.0));
		keys_sr25519.push(key_ss58);
	}
	println!("ed25519 keys:");
	let mut keys_ed25519 = vec![];
	for pubkey in store.ed25519_public_keys(ED25519_KEY_TYPE).into_iter() {
		let key_ss58 = pubkey.to_ss58check();
		println!("{}", key_ss58);
		keys_ed25519.push(key_ss58);
	}
	drop(store);

	Ok(CliResultOk::PubKeysBase58 {
		pubkeys_sr25519: Some(keys_sr25519),
		pubkeys_ed25519: Some(keys_ed25519),
	})
}

fn print_metadata(cli: &Cli) -> CliResult {
	let api = get_chain_api(cli);
	let meta = api.metadata();
	println!("Metadata:\n {}", &meta.pretty_format().unwrap());
	Ok(CliResultOk::Metadata { metadata: meta.clone() })
}
fn print_sgx_metadata(cli: &Cli) -> CliResult {
	let worker_api_direct = get_worker_api_direct(cli);
	let metadata = worker_api_direct.get_state_metadata().unwrap();
	println!("Metadata:\n {}", metadata.pretty_format().unwrap());
	Ok(CliResultOk::Metadata { metadata })
}

fn print_sgx_metadata_raw(cli: &Cli) -> CliResult {
	let worker_api_direct = get_worker_api_direct(cli);
	let metadata = worker_api_direct.get_state_metadata_raw().unwrap();
	println!("{metadata}");
	Ok(CliResultOk::None)
}

fn list_workers(cli: &Cli) -> CliResult {
	let api = get_chain_api(cli);
	let enclaves = api.all_enclaves(WorkerType::Identity, None).unwrap();
	println!("number of enclaves registered: {}", enclaves.len());

	let mr_enclaves = enclaves
		.iter()
		.map(|enclave| {
			println!("Enclave");
			println!("   MRENCLAVE: {}", enclave.mrenclave.to_base58());
			let timestamp = DateTime::<Utc>::from(
				UNIX_EPOCH + Duration::from_millis(enclave.last_seen_timestamp),
			);
			println!("   Last seen: {}", timestamp);
			println!("   URL: {}", String::from_utf8_lossy(enclave.url.as_slice()));
			enclave.mrenclave.to_base58()
		})
		.collect();

	Ok(CliResultOk::MrEnclaveBase58 { mr_enclaves })
}
