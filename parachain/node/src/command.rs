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

#![allow(clippy::borrowed_box)]

use crate::{
	chain_specs,
	cli::{Cli, RelayChainCli, Subcommand},
	service::*,
};
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_runtime::traits::AccountIdConversion;
use std::net::SocketAddr;

const UNSUPPORTED_CHAIN_MESSAGE: &str = "Unsupported chain spec, please use litentry*";

trait IdentifyChain {
	fn is_litentry(&self) -> bool;
	fn is_paseo(&self) -> bool;
	fn is_standalone(&self) -> bool;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn is_litentry(&self) -> bool {
		// we need the combined condition as the id in our paseo spec starts with `litentry-paseo`
		// simply renaming `litentry-paseo` to `paseo` everywhere would have an impact on the
		// existing litentry-paseo chain
		self.id().starts_with("litentry") && !self.id().starts_with("litentry-paseo")
	}
	fn is_paseo(&self) -> bool {
		self.id().starts_with("litentry-paseo")
	}
	fn is_standalone(&self) -> bool {
		self.id().eq("standalone") || self.id().eq("dev")
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyChain for T {
	fn is_litentry(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_litentry(self)
	}
	fn is_paseo(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_paseo(self)
	}
	fn is_standalone(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_standalone(self)
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		// `--chain=standalone or --chain=dev` to start a standalone node with paseo-dev chain spec
		// mainly based on Acala's `dev` implementation
		"dev" | "standalone" => Box::new(chain_specs::paseo::get_chain_spec_dev(true)),
		// Litentry
		"litentry-dev" => Box::new(chain_specs::litentry::get_chain_spec_dev()),
		"litentry-staging" => Box::new(chain_specs::litentry::get_chain_spec_staging()),
		"litentry" => Box::new(chain_specs::ChainSpec::from_json_bytes(
			&include_bytes!("../res/chain_specs/litentry.json")[..],
		)?),
		// Paseo
		"paseo-dev" => Box::new(chain_specs::paseo::get_chain_spec_dev(false)),
		"paseo" => Box::new(chain_specs::ChainSpec::from_json_bytes(
			&include_bytes!("../res/chain_specs/paseo.json")[..],
		)?),
		// Generate res/chain_specs/litentry.json
		"generate-litentry" => Box::new(chain_specs::litentry::get_chain_spec_prod()),
		// Generate res/chain_specs/paseo.json
		"generate-paseo" => Box::new(chain_specs::paseo::get_chain_spec_prod()),
		path => {
			let chain_spec = chain_specs::ChainSpec::from_json_file(path.into())?;
			if chain_spec.is_paseo() {
				Box::new(chain_specs::ChainSpec::from_json_file(path.into())?)
			} else {
				// Fallback: use Litentry chain spec
				Box::new(chain_spec)
			}
		},
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Litentry node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Litentry node\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		litentry-collator <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/litentry/litentry-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Litentry node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Litentry node\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		litentry-collator <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/litentry/litentry-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}
}

/// Creates partial components for the runtimes that are supported by the benchmarks.
macro_rules! construct_benchmark_partials {
	($config:expr, |$partials:ident| $code:expr) => {
		if $config.chain_spec.is_litentry() {
			let $partials = new_partial::<_>(&$config, build_import_queue, false, true)?;
			$code
		} else if $config.chain_spec.is_paseo() {
			let $partials = new_partial::<_>(&$config, build_import_queue, false, true)?;
			$code
		} else {
			panic!("{}", UNSUPPORTED_CHAIN_MESSAGE)
		}
	};
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;

		if runner.config().chain_spec.is_litentry() {
			runner.async_run(|$config| {
				let $components = new_partial::<
					_
				>(
					&$config,
					build_import_queue,
					false,
					$cli.delayed_best_block,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		} else if runner.config().chain_spec.is_paseo() {
			runner.async_run(|$config| {
				let $components = new_partial::<
					_
				>(
					&$config,
					build_import_queue,
					false,
					$cli.delayed_best_block,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		}
		else {
			panic!("{}", UNSUPPORTED_CHAIN_MESSAGE)
		}
	}}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();
	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
				);
				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::Revert(cmd)) => construct_async_run!(|components, cli, cmd, config| {
			Ok(cmd.run(components.client, components.backend, None))
		}),

		Some(Subcommand::Key(cmd)) => cmd.run(&cli),

		Some(Subcommand::ExportGenesisHead(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			if runner.config().chain_spec.is_litentry() {
				runner.sync_run(|config| {
					let sc_service::PartialComponents { client, .. } = new_partial::<_>(
						&config,
						build_import_queue,
						false,
						cli.delayed_best_block,
					)?;
					cmd.run(client)
				})
			} else if runner.config().chain_spec.is_paseo() {
				runner.sync_run(|config| {
					let sc_service::PartialComponents { client, .. } = new_partial::<_>(
						&config,
						build_import_queue,
						false,
						cli.delayed_best_block,
					)?;
					cmd.run(client)
				})
			} else {
				panic!("{}", UNSUPPORTED_CHAIN_MESSAGE)
			}
		},
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			match cmd {
				BenchmarkCmd::Pallet(cmd) => {
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							cmd.run_with_spec::<sp_runtime::traits::HashingFor<crate::service::Block>, ()>(
								Some(config.chain_spec),
							)
						})
					} else {
						Err(sc_cli::Error::Input(
							"Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
								.into(),
						))
					}
				},
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| cmd.run(partials.client))
				}),
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) => Err(sc_cli::Error::Input(
					"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
						.into(),
				)),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Machine(cmd) => {
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
				},
				// NOTE: this allows the Client to leniently implement
				// new benchmark commands without requiring a companion MR.
				#[allow(unreachable_patterns)]
				_ => Err("Benchmarking sub-command unsupported".into()),
			}
		},

		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();
			let is_standalone = runner.config().chain_spec.is_standalone();

			let evm_tracing_config = crate::evm_tracing_types::EvmTracingConfig {
				ethapi: cli.eth_api_options.ethapi,
				ethapi_max_permits: cli.eth_api_options.ethapi_max_permits,
				ethapi_trace_max_count: cli.eth_api_options.ethapi_trace_max_count,
				ethapi_trace_cache_duration: cli.eth_api_options.ethapi_trace_cache_duration,
				eth_log_block_cache: cli.eth_api_options.eth_log_block_cache,
				eth_statuses_cache: cli.eth_api_options.eth_statuses_cache,
				max_past_logs: cli.eth_api_options.max_past_logs,
				tracing_raw_max_memory_usage: cli.eth_api_options.tracing_raw_max_memory_usage,
			};

			runner.run_node_until_exit(|config| async move {
				if is_standalone {
					return start_standalone_node(config, evm_tracing_config)
						.await
						.map_err(Into::into);
				}

				let hwbench = if !cli.no_hardware_benchmarks {
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})
				} else {
					None
				};

				let para_id = ParaId::from(
					chain_specs::Extensions::try_get(&*config.chain_spec)
						.map(|e| e.para_id)
						.ok_or("Could not find ParaId in chain spec")?,
				);

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
				);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(
						&para_id,
					);

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", para_id);
				info!("Parachain Account: {}", parachain_account);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				let additional_config =
					AdditionalConfig { evm_tracing_config, enable_evm_rpc: cli.enable_evm_rpc };

				if config.chain_spec.is_litentry() {
					start_node(
						config,
						polkadot_config,
						collator_options,
						para_id,
						hwbench,
						additional_config,
						cli.delayed_best_block,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				} else if config.chain_spec.is_paseo() {
					start_node(
						config,
						polkadot_config,
						collator_options,
						para_id,
						hwbench,
						additional_config,
						cli.delayed_best_block,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				} else {
					Err(UNSUPPORTED_CHAIN_MESSAGE.into())
				}
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()?
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_addr(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_addr(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
		self.base.base.trie_cache_maximum_size()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_max_connections(&self) -> Result<u32> {
		self.base.base.rpc_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
