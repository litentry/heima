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
	trusted_base_cli::commands::bitacross::utils::send_direct_request,
	trusted_cli::TrustedCli,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	Cli, CliResult, CliResultOk,
};
use itp_rpc::{RpcResponse, RpcReturnValue};
use itp_stf_primitives::types::KeyPair;
use itp_utils::FromHexPrefixed;
use lc_direct_call::{DirectCall, PrehashedEthereumMessage};
use sp_core::Pair;

#[derive(Parser)]
pub struct RequestDirectCallSignEthereumCommand {
	payload: Vec<u8>,
}

impl RequestDirectCallSignEthereumCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		let alice = get_pair_from_str(trusted_cli, "//Alice", cli);
		let (mrenclave, shard) = get_identifiers(trusted_cli, cli);
		let msg: PrehashedEthereumMessage =
			self.payload.clone().try_into().expect("Unable to convert payload to [u8; 32]");

		let dc = DirectCall::SignEthereum(alice.public().into(), msg).sign(
			&KeyPair::Sr25519(Box::new(alice)),
			&mrenclave,
			&shard,
		);

		let result: String = send_direct_request(cli, trusted_cli, dc).unwrap();
		let response: RpcResponse = serde_json::from_str(&result).unwrap();
		if let Ok(return_value) = RpcReturnValue::from_hex(&response.result) {
			println!("Got return value: {:?}", return_value);
		} else {
			println!("Could not decode return value: {:?}", response.result);
		}
		println!("Got result: {:?}", result);

		Ok(CliResultOk::None)
	}
}
