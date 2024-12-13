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
	trusted_cli::TrustedCli,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_operation::perform_trusted_operation,
	Cli, CliResult, CliResultOk,
};
use clap::Parser;
use ita_stf::{Index, TrustedCall};
use itp_stf_primitives::{traits::TrustedCallSigning, types::KeyPair};
use sp_core::Pair;

// usage exmaple:
//
// ./bin/litentry-cli trusted --mrenclave <mrenclave> --direct clean-id-graphs

#[derive(Parser)]
pub struct CleanIDGraphsCommand {}

impl CleanIDGraphsCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_cli: &TrustedCli) -> CliResult {
		let alice = get_pair_from_str(trusted_cli, "//Alice", cli);

		let (mrenclave, shard) = get_identifiers(trusted_cli, cli);
		let nonce = get_layer_two_nonce!(alice, cli, trusted_cli);

		let top = TrustedCall::clean_id_graphs(alice.public().into())
			.sign(&KeyPair::Sr25519(Box::new(alice)), nonce, &mrenclave, &shard)
			.into_trusted_operation(trusted_cli.direct);
		Ok(perform_trusted_operation::<()>(cli, trusted_cli, &top).map(|_| CliResultOk::None)?)
	}
}
