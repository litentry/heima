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

use crate::{chain_specs, evm_tracing_types::EthApiOptions};
use clap::Parser;
use std::path::PathBuf;

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Export the genesis state of the parachain.
	#[clap(name = "export-genesis-state")]
	ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	#[clap(name = "export-genesis-wasm")]
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// Sub-commands concerned with benchmarking.
	/// The pallet benchmarking moved to the `pallet` sub-command.
	#[command(subcommand)]
	Benchmark(Box<frame_benchmarking_cli::BenchmarkCmd>),

	/// Try some testing command against a specified runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// Errors since the binary was not build with `--features try-runtime`.
	#[cfg(not(feature = "try-runtime"))]
	TryRuntime,
}

#[derive(Debug, Parser)]
#[command(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[command(flatten)]
	pub run: cumulus_client_cli::RunCmd,

	/// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[arg(long)]
	pub no_hardware_benchmarks: bool,

	/// Relay chain arguments
	#[arg(raw = true, conflicts_with = "relay-chain-rpc-url")]
	pub relay_chain_args: Vec<String>,

	#[clap(flatten)]
	pub eth_api_options: EthApiOptions,

	/// Enable Ethereum compatible JSON-RPC servers (disabled by default).
	#[clap(name = "enable-evm-rpc", long)]
	pub enable_evm_rpc: bool,

	/// Proposer's maximum block size limit in bytes
	#[clap(long, default_value = sc_basic_authorship::DEFAULT_BLOCK_SIZE_LIMIT.to_string())]
	pub proposer_block_size_limit: usize,

	/// Proposer's soft deadline in percents of block size
	#[clap(long, default_value = "50")]
	pub proposer_soft_deadline_percent: u8,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = chain_specs::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.path().join("polkadot");
		Self { base_path: Some(base_path), chain_id, base: Parser::parse_from(relay_chain_args) }
	}
}
