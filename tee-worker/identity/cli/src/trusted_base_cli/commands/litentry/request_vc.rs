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

use crate::{
	get_layer_two_nonce,
	trusted_base_cli::commands::litentry::request_vc_subcommands::Command,
	trusted_cli::TrustedCli,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_operation::prepare_request_data_and_send_direct_vc_request,
	Cli, CliResult, CliResultOk,
};
use clap::Parser;
use codec::Decode;
use ita_stf::{trusted_call_result::RequestVCResult, Index, TrustedCall};
use itp_stf_primitives::{traits::TrustedCallSigning, types::KeyPair};
use litentry_primitives::{
	aes_decrypt, Assertion, BoundedWeb3Network, Identity, ParameterString, RequestAesKey,
	Web3Network, AES_KEY_LEN,
};
use sp_core::Pair;

// usage example below
//
// Basically, the assertion subcommand needs to be quoted to signal the value group for certain assertion.
// You can specifiy `-a "<value>"` multiple times to pass in a batched vc request
//
// Printing `--help` give some information but clap doesn't know anything about the value specifiction.
// However, if you put mismatched parameters for subcommands you'll get an error hint during the parsing.
// For example:
// -a "a2 p1 p2" will give you:
//   error: unexpected argument 'p2'
//   Usage: placeholder a2 <GUILD_ID>
// as a2 expects A2Arg which only has one field `guild_id`
//
// single a8:
// ./bin/litentry-cli trusted -d request-vc \
//   did:litentry:substrate:0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48 -a "a8 litentry,litmus"
//
// single OneBlock:
// ./bin/litentry-cli trusted -d request-vc \
//   did:litentry:substrate:0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48 -a "one-block completion"
//
// batched a1 + a2 + a3:
// ./bin/litentry-cli trusted -d request-vc \
//   did:litentry:substrate:0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48 \
//   -a "a1" -a "a2 gid" -a "a3 gid cid rid"
//
// batched achainable + vip3:
// ./bin/litentry-cli trusted -d request-vc \
//   did:litentry:substrate:0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48 \
//   -a "achainable amount-holding a -c=litentry 1 2014-05-01" \
//   -a "vip3-membership-card gold"

pub fn to_para_str<T>(s: T) -> ParameterString
where
	T: AsRef<[u8]>,
{
	ParameterString::truncate_from(s.as_ref().to_vec())
}

pub fn to_chains<T, U>(networks: U) -> BoundedWeb3Network
where
	T: AsRef<str>,
	U: IntoIterator<Item = T>,
{
	let networks: Vec<Web3Network> =
		networks.into_iter().map(|n| n.as_ref().try_into().unwrap()).collect();

	networks.try_into().unwrap()
}

#[derive(Debug, Parser)]
pub struct RequestVcCommand {
	// did account to whom the vc will be issued
	did: String,
	// the assertion itself, can be specified more than once
	// the value will be passed into the parser as a whole string
	#[clap(short, long, num_args = 1..)]
	assertion: Vec<String>,
}

fn print_vc(key: &RequestAesKey, mut vc: RequestVCResult) {
	let decrypted = aes_decrypt(key, &mut vc.vc_payload).unwrap();
	let credential_str = String::from_utf8(decrypted).expect("Found invalid UTF-8");
	println!("----Generated VC-----");
	println!("{}", credential_str);
	if let Some(mut vc_logs) = vc.vc_logs {
		let decrypted_logs = aes_decrypt(key, &mut vc_logs).unwrap();
		if !decrypted_logs.is_empty() {
			let logs_str = String::from_utf8(decrypted_logs).expect("Found invalid UTF-8");
			println!("----VC Logging-----");
			println!("{}", logs_str);
		}
	}
}

impl RequestVcCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		let alice = get_pair_from_str(trusted_cli, "//Alice", cli);
		let identity = Identity::from_did(self.did.as_str()).unwrap();
		println!(">>> identity: {:?}", identity);

		let (mrenclave, shard) = get_identifiers(trusted_cli, cli);
		let nonce = get_layer_two_nonce!(alice, cli, trusted_cli);
		println!(">>> nonce: {}", nonce);

		let assertions: Vec<Assertion> = self
			.assertion
			.iter()
			.map(|a| {
				let mut s = vec!["placeholder"];
				s.extend(a.as_str().split(' '));
				AssertionCommand::parse_from(s).command.to_assertion()
			})
			.collect::<Result<Vec<_>, _>>()?;

		println!(">>> assertions: {:?}", assertions);

		let key = Self::random_aes_key();

		let top = TrustedCall::request_batch_vc(
			alice.public().into(),
			identity,
			assertions.try_into().unwrap(),
			Some(key),
			Default::default(),
		)
		.sign(&KeyPair::Sr25519(Box::new(alice)), 0, &mrenclave, &shard)
		.into_trusted_operation(trusted_cli.direct);

		match prepare_request_data_and_send_direct_vc_request(cli, trusted_cli, &top, key) {
			Ok(result) =>
				for res in result {
					match res.result {
						Err(err) => {
							println!("received one error: {:?}", err);
						},
						Ok(payload) => {
							let vc = RequestVCResult::decode(&mut payload.as_slice()).unwrap();
							print_vc(&key, vc);
						},
					}
				},
			Err(e) => {
				println!("{:?}", e);
			},
		}

		Ok(CliResultOk::None)
	}

	fn random_aes_key() -> RequestAesKey {
		let random: Vec<u8> = (0..AES_KEY_LEN).map(|_| rand::random::<u8>()).collect();
		random[0..AES_KEY_LEN].try_into().unwrap()
	}
}

#[derive(Debug, clap::Parser)]
// the wrapper to the underlying `subcommand` type
pub struct AssertionCommand {
	/// subcommand to define the vc type requested
	#[clap(subcommand)]
	pub command: Command,
}
