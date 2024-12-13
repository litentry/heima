use crate::{
	command_utils::{get_shielding_key, get_worker_api_direct},
	trusted_base_cli::commands::litentry::request_vc_subcommands::Command,
	trusted_cli::TrustedCli,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_operation::send_direct_vc_request,
	Cli, CliResult, CliResultOk,
};
use clap::Parser;
use core::time::Duration;
use ita_sgx_runtime::pallet_identity_management_tee::Identity;
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itc_rpc_client::direct_client::DirectClient;
use itp_stf_primitives::{
	traits::TrustedCallSigning,
	types::{KeyPair, TrustedOperation},
};
use litentry_primitives::{RequestAesKey, ShardIdentifier, AES_KEY_LEN};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use sp_core::Pair;
use std::time::Instant;

//	Command to perform request_vc benchmarking. It creates a given number of threads and sends
//	`requests_per_thread` requests one by one recording time to get response. It uses Alice as
//	an IdGraph root. If bench case requires any identities to be linked, the preparation should be done
//	using other CLI commands.
//	Example usage:
//	./litentry-cli trusted benchmark-request-vc 8 400 \
//		did:litentry:substrate:0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48 \
//		a1
#[derive(Parser)]
pub struct BenchmarkRequestVcCommand {
	threads: u32,
	requests_per_thread: u32,
	// did account to whom the vc will be issued
	did: String,
	#[clap(subcommand)]
	assertion: Command,
}

struct ActionData<'a> {
	client: &'a DirectClient,
	request_aes_key: &'a RequestAesKey,
	shard_identifier: &'a ShardIdentifier,
	shielding_key: &'a Rsa3072PubKey,
	top: &'a TrustedOperation<TrustedCallSigned, Getter>,
}

struct Client<'a> {
	client_api: DirectClient,
	action: fn(&ActionData) -> Result<Duration, ()>,
	triggers: u32,
	aes_key: &'a RequestAesKey,
	shard: &'a ShardIdentifier,
	shielding_key: &'a Rsa3072PubKey,
	top: &'a TrustedOperation<TrustedCallSigned, Getter>,
}

impl BenchmarkRequestVcCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		println!("Running request_vc benchmarking");
		let identity = Identity::from_did(self.did.as_str()).unwrap();
		let alice = get_pair_from_str(trusted_cli, "//Alice", cli);
		let (mrenclave, shard) = get_identifiers(trusted_cli, cli);
		let encryption_key = get_shielding_key(cli).unwrap();

		let threads = self.threads;

		let mut clients = Vec::with_capacity(threads as usize);

		let assertion = self.assertion.to_assertion().unwrap();
		let aes_key = random_aes_key();

		let top = TrustedCall::request_vc(
			alice.public().into(),
			identity,
			assertion,
			Some(aes_key),
			Default::default(),
		)
		.sign(&KeyPair::Sr25519(Box::new(alice)), 1, &mrenclave, &shard)
		.into_trusted_operation(true);

		for _ in 0..threads {
			let worker_api_direct = get_worker_api_direct(cli);
			clients.push(Client {
				client_api: worker_api_direct,
				action: |action_data| {
					let action_start: Instant = Instant::now();
					match send_direct_vc_request(
						action_data.client,
						*action_data.shard_identifier,
						action_data.shielding_key,
						*action_data.request_aes_key,
						action_data.top,
					) {
						Ok(mut result) => {
							let result = result.remove(0);
							match result.result {
								Err(_) => Err(()),
								Ok(_) => Ok(action_start.elapsed()),
							}
						},
						Err(_) => Err(()),
					}
				},
				triggers: self.requests_per_thread,
				aes_key: &aes_key,
				shard: &shard,
				shielding_key: &encryption_key,
				top: &top,
			});
		}

		rayon::ThreadPoolBuilder::new()
			.num_threads(threads as usize)
			.build_global()
			.unwrap();

		let overall_start: Instant = Instant::now();

		let results: Vec<Vec<Result<Duration, ()>>> = clients
			.into_par_iter()
			.map(move |client| {
				let mut results = Vec::with_capacity(client.triggers as usize);
				for _ in 0..client.triggers {
					let action_data = ActionData {
						client: &client.client_api,
						request_aes_key: client.aes_key,
						shard_identifier: client.shard,
						shielding_key: client.shielding_key,
						top: client.top,
					};
					results.push((client.action)(&action_data));
				}
				results
			})
			.collect();

		let mut time_sum = 0_f64;
		let mut time_count = 0;
		let mut errors_count = 0;

		for result in results {
			for result in result {
				match result {
					Ok(time) => {
						time_sum += time.as_secs_f64();
						time_count += 1;
					},
					Err(_) => errors_count += 1,
				}
			}
		}
		println!("Total time: {:.4}s", overall_start.elapsed().as_secs_f64());
		println!("Average request vc time is: {:.4}s", time_sum / (time_count as f64));
		println!("Errors count: {}", errors_count);
		Ok(CliResultOk::None)
	}
}

fn random_aes_key() -> RequestAesKey {
	let random: Vec<u8> = (0..AES_KEY_LEN).map(|_| rand::random::<u8>()).collect();
	random[0..AES_KEY_LEN].try_into().unwrap()
}
