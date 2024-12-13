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
	command_utils::get_worker_api_direct, trusted_operation::get_id_graph_hash, Cli, CliResult,
	CliResultOk, H256,
};
use codec::Decode;
use itc_rpc_client::direct_client::DirectApi;
use itp_types::ShardIdentifier;
use litentry_primitives::Identity;

// usage: ./bin/litentry-cli id-graph-hash did-identity
//
// this command prints the id graph hash of the given identity in did form

#[derive(Parser)]
pub struct IDGraphHashCommand {
	/// identity to query, in did form
	did: String,
}

impl IDGraphHashCommand {
	pub(crate) fn run(&self, cli: &Cli) -> CliResult {
		let direct_api = get_worker_api_direct(cli);
		let mrenclave = direct_api.get_state_mrenclave().unwrap();
		let shard = ShardIdentifier::decode(&mut &mrenclave[..]).unwrap();
		let identity = Identity::from_did(self.did.as_str()).unwrap();
		let id_graph_hash = get_id_graph_hash::<H256>(&direct_api, &shard, &identity).unwrap();
		println!("{:?}", id_graph_hash);

		Ok(CliResultOk::None)
	}
}
