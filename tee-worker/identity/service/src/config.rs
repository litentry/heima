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

use clap::ArgMatches;
use itc_rest_client::rest_client::Url;
use itp_types::{parentchain::ParentchainId, ShardIdentifier};
use parse_duration::parse;
use serde::{Deserialize, Serialize};
use std::{
	fs,
	path::{Path, PathBuf},
	time::Duration,
};

static DEFAULT_NODE_URL: &str = "ws://127.0.0.1";
static DEFAULT_NODE_PORT: &str = "9944";
static DEFAULT_TRUSTED_PORT: &str = "2000";
static DEFAULT_UNTRUSTED_PORT: &str = "2001";
static DEFAULT_MU_RA_PORT: &str = "3443";
static DEFAULT_METRICS_PORT: &str = "8787";
static DEFAULT_UNTRUSTED_HTTP_PORT: &str = "4545";
static DEFAULT_MOCK_SERVER_PORT: &str = "19527";
static DEFAULT_PARENTCHAIN_START_BLOCK: &str = "0";
static DEFAULT_FAIL_AT: &str = "0";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
	pub litentry_rpc_url: String,
	pub litentry_rpc_port: String,
	pub target_a_parentchain_rpc_url: Option<String>,
	pub target_a_parentchain_rpc_port: Option<String>,
	pub target_b_parentchain_rpc_url: Option<String>,
	pub target_b_parentchain_rpc_port: Option<String>,
	pub worker_ip: String,
	/// Trusted worker address that will be advertised on the parentchain.
	pub trusted_external_worker_address: Option<String>,
	/// Port to directly communicate with the trusted tls server inside the enclave.
	pub trusted_worker_port: String,
	/// Untrusted worker address that will be returned by the dedicated trusted ws rpc call.
	pub untrusted_external_worker_address: Option<String>,
	/// Port to the untrusted ws of the validateer.
	pub untrusted_worker_port: String,
	/// Mutual remote attestation address that will be returned by the dedicated trusted ws rpc call.
	pub mu_ra_external_address: Option<String>,
	/// Port for mutual-remote attestation requests.
	pub mu_ra_port: String,
	/// Enable the metrics server
	pub enable_metrics_server: bool,
	/// Port for the metrics server
	pub metrics_server_port: String,
	/// Port for the untrusted HTTP server (e.g. for `is_initialized`)
	pub untrusted_http_port: String,
	/// Data directory used by all the services.
	pub data_dir: PathBuf,
	/// Config of the 'run' subcommand
	pub run_config: Option<RunConfig>,

	/// whether to enable the HTTP mock server for testing
	pub enable_mock_server: bool,
	/// the mock server port
	pub mock_server_port: String,
	/// the parentchain block number to start syncing with
	pub parentchain_start_block: String,
	/// mode to use for failing sidechain slot
	pub fail_slot_mode: Option<String>,
	/// slot number to fail at
	pub fail_at: u64,
}

#[allow(clippy::too_many_arguments)]
impl Config {
	pub fn new(
		litentry_rpc_url: String,
		litentry_rpc_port: String,
		target_a_parentchain_rpc_url: Option<String>,
		target_a_parentchain_rpc_port: Option<String>,
		target_b_parentchain_rpc_url: Option<String>,
		target_b_parentchain_rpc_port: Option<String>,
		worker_ip: String,
		trusted_external_worker_address: Option<String>,
		trusted_worker_port: String,
		untrusted_external_worker_address: Option<String>,
		untrusted_worker_port: String,
		mu_ra_external_address: Option<String>,
		mu_ra_port: String,
		enable_metrics_server: bool,
		metrics_server_port: String,
		untrusted_http_port: String,
		data_dir: PathBuf,
		run_config: Option<RunConfig>,
		enable_mock_server: bool,
		mock_server_port: String,
		parentchain_start_block: String,
		fail_slot_mode: Option<String>,
		fail_at: u64,
	) -> Self {
		Self {
			litentry_rpc_url,
			litentry_rpc_port,
			target_a_parentchain_rpc_url,
			target_a_parentchain_rpc_port,
			target_b_parentchain_rpc_url,
			target_b_parentchain_rpc_port,
			worker_ip,
			trusted_external_worker_address,
			trusted_worker_port,
			untrusted_external_worker_address,
			untrusted_worker_port,
			mu_ra_external_address,
			mu_ra_port,
			enable_metrics_server,
			metrics_server_port,
			untrusted_http_port,
			data_dir,
			run_config,
			enable_mock_server,
			mock_server_port,
			parentchain_start_block,
			fail_slot_mode,
			fail_at,
		}
	}

	/// Integritee RPC endpoint (including ws://).
	pub fn litentry_rpc_endpoint(&self) -> String {
		format!("{}:{}", self.litentry_rpc_url, self.litentry_rpc_port)
	}

	pub fn target_a_parentchain_rpc_endpoint(&self) -> Option<String> {
		if self.target_a_parentchain_rpc_url.is_some()
			&& self.target_a_parentchain_rpc_port.is_some()
		{
			return Some(format!(
				"{}:{}",
				// Can be done better, but this code is obsolete anyhow with clap v4.
				self.target_a_parentchain_rpc_url.clone().unwrap(),
				self.target_a_parentchain_rpc_port.clone().unwrap()
			));
		};

		None
	}

	pub fn target_b_parentchain_rpc_endpoint(&self) -> Option<String> {
		if self.target_b_parentchain_rpc_url.is_some()
			&& self.target_b_parentchain_rpc_port.is_some()
		{
			return Some(format!(
				"{}:{}",
				// Can be done better, but this code is obsolete anyhow with clap v4.
				self.target_b_parentchain_rpc_url.clone().unwrap(),
				self.target_b_parentchain_rpc_port.clone().unwrap()
			));
		};

		None
	}

	pub fn trusted_worker_url_internal(&self) -> String {
		// use the same scheme as `trusted_worker_url_external`
		let url = url::Url::parse(self.trusted_worker_url_external().as_str()).unwrap();
		format!("{}://{}:{}", url.scheme(), self.worker_ip, self.trusted_worker_port)
	}

	/// Returns the trusted worker url that should be addressed by external clients.
	pub fn trusted_worker_url_external(&self) -> String {
		match &self.trusted_external_worker_address {
			Some(external_address) => ensure_ws_or_wss(external_address),
			None => format!("wss://{}:{}", self.worker_ip, self.trusted_worker_port), // fallback to wss
		}
	}

	pub fn untrusted_worker_url(&self) -> String {
		// use the same scheme as `untrusted_worker_url_external`
		let url = url::Url::parse(self.untrusted_worker_url_external().as_str()).unwrap();
		format!("{}://{}:{}", url.scheme(), self.worker_ip, self.untrusted_worker_port)
	}

	/// Returns the untrusted worker url that should be addressed by external clients.
	pub fn untrusted_worker_url_external(&self) -> String {
		match &self.untrusted_external_worker_address {
			Some(external_address) => ensure_ws_or_wss(external_address),
			None => format!("ws://{}:{}", self.worker_ip, self.untrusted_worker_port), // fallback to ws
		}
	}

	pub fn mu_ra_url(&self) -> String {
		format!("{}:{}", self.worker_ip, self.mu_ra_port)
	}

	/// Returns the mutual remote attestion worker url that should be addressed by external workers.
	pub fn mu_ra_url_external(&self) -> String {
		match &self.mu_ra_external_address {
			Some(external_address) => external_address.to_string(),
			None => format!("{}:{}", self.worker_ip, self.mu_ra_port),
		}
	}

	pub fn data_dir(&self) -> &Path {
		self.data_dir.as_path()
	}

	pub fn run_config(&self) -> &Option<RunConfig> {
		&self.run_config
	}

	pub fn enable_metrics_server(&self) -> bool {
		self.enable_metrics_server
	}

	pub fn try_parse_metrics_server_port(&self) -> Option<u16> {
		self.metrics_server_port.parse::<u16>().ok()
	}

	pub fn try_parse_untrusted_http_server_port(&self) -> Option<u16> {
		self.untrusted_http_port.parse::<u16>().ok()
	}

	pub fn try_parse_mock_server_port(&self) -> Option<u16> {
		self.mock_server_port.parse::<u16>().ok()
	}

	pub fn try_parse_parentchain_start_block(&self) -> Option<u32> {
		self.parentchain_start_block.parse::<u32>().ok()
	}
}

impl From<&ArgMatches<'_>> for Config {
	fn from(m: &ArgMatches<'_>) -> Self {
		let trusted_port = m.value_of("trusted-worker-port").unwrap_or(DEFAULT_TRUSTED_PORT);
		let untrusted_port = m.value_of("untrusted-worker-port").unwrap_or(DEFAULT_UNTRUSTED_PORT);
		let mu_ra_port = m.value_of("mu-ra-port").unwrap_or(DEFAULT_MU_RA_PORT);
		let is_metrics_server_enabled = m.is_present("enable-metrics");
		let metrics_server_port = m.value_of("metrics-port").unwrap_or(DEFAULT_METRICS_PORT);
		let untrusted_http_port =
			m.value_of("untrusted-http-port").unwrap_or(DEFAULT_UNTRUSTED_HTTP_PORT);

		let data_dir = match m.value_of("data-dir") {
			Some(d) => {
				let p = PathBuf::from(d);
				if !p.exists() {
					log::info!("Creating new data-directory for the service {}.", p.display());
					fs::create_dir_all(p.as_path()).unwrap();
				} else {
					log::info!("Starting service in existing directory {}.", p.display());
				}
				p
			},
			None => {
				log::warn!("[Config] defaulting to data-dir = PWD because it was previous behaviour. This might change soon.\
				Please pass the data-dir explicitly to ensure nothing breaks in your setup.");
				pwd()
			},
		};

		let run_config = m.subcommand_matches("run").map(RunConfig::from);

		let is_mock_server_enabled = m.is_present("enable-mock-server");
		let mock_server_port = m.value_of("mock-server-port").unwrap_or(DEFAULT_MOCK_SERVER_PORT);
		let parentchain_start_block =
			m.value_of("parentchain-start-block").unwrap_or(DEFAULT_PARENTCHAIN_START_BLOCK);
		let fail_slot_mode = m.value_of("fail-slot-mode").map(|v| v.to_string());
		let fail_at = m.value_of("fail-at").unwrap_or(DEFAULT_FAIL_AT).parse().unwrap();
		Self::new(
			m.value_of("node-url").unwrap_or(DEFAULT_NODE_URL).into(),
			m.value_of("node-port").unwrap_or(DEFAULT_NODE_PORT).into(),
			m.value_of("target-a-parentchain-rpc-url").map(Into::into),
			m.value_of("target-a-parentchain-rpc-port").map(Into::into),
			m.value_of("target-b-parentchain-rpc-url").map(Into::into),
			m.value_of("target-b-parentchain-rpc-port").map(Into::into),
			if m.is_present("ws-external") { "0.0.0.0".into() } else { "127.0.0.1".into() },
			m.value_of("trusted-external-address")
				.map(|url| add_port_if_necessary(url, trusted_port)),
			trusted_port.to_string(),
			m.value_of("untrusted-external-address")
				.map(|url| add_port_if_necessary(url, untrusted_port)),
			untrusted_port.to_string(),
			m.value_of("mu-ra-external-address")
				.map(|url| add_port_if_necessary(url, mu_ra_port)),
			mu_ra_port.to_string(),
			is_metrics_server_enabled,
			metrics_server_port.to_string(),
			untrusted_http_port.to_string(),
			data_dir,
			run_config,
			is_mock_server_enabled,
			mock_server_port.to_string(),
			parentchain_start_block.to_string(),
			fail_slot_mode,
			fail_at,
		)
	}
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunConfig {
	/// Skip remote attestation. Set this flag if running enclave in SW mode
	skip_ra: bool,
	/// Set this flag if running in development mode to bootstrap enclave account on parentchain via //Alice.
	dev: bool,
	/// Shard identifier base58 encoded. Defines the shard that this worker operates on. Default is mrenclave.
	shard: Option<String>,
	/// Marblerun's Prometheus endpoint base URL
	marblerun_base_url: Option<String>,
	/// parentchain which should be used for shielding/unshielding the stf's native token
	pub shielding_target: Option<ParentchainId>,
}

impl RunConfig {
	pub fn skip_ra(&self) -> bool {
		self.skip_ra
	}

	pub fn dev(&self) -> bool {
		self.dev
	}

	pub fn shard(&self) -> Option<&str> {
		self.shard.as_deref()
	}

	pub fn marblerun_base_url(&self) -> &str {
		// This conflicts with the default port of a substrate node, but it is indeed the
		// default port of marblerun too:
		// https://github.com/edgelesssys/marblerun/blob/master/docs/docs/workflows/monitoring.md?plain=1#L26
		self.marblerun_base_url.as_deref().unwrap_or("http://localhost:9944")
	}
}

impl From<&ArgMatches<'_>> for RunConfig {
	fn from(m: &ArgMatches<'_>) -> Self {
		let skip_ra = m.is_present("skip-ra");
		let dev = m.is_present("dev");
		let shard = m.value_of("shard").map(|s| s.to_string());

		let marblerun_base_url = m.value_of("marblerun-url").map(|i| {
			Url::parse(i)
				.unwrap_or_else(|e| panic!("marblerun-url parsing error: {:?}", e))
				.to_string()
		});

		let shielding_target = m.value_of("shielding-target").map(|i| match i {
			"litentry" => ParentchainId::Litentry,
			"target_a" => ParentchainId::TargetA,
			"target_b" => ParentchainId::TargetB,
			_ => panic!(
				"failed to parse shielding-target: {} must be one of litentry|target_a|target_b",
				i
			),
		});

		Self { skip_ra, dev, shard, marblerun_base_url, shielding_target }
	}
}

fn add_port_if_necessary(url: &str, port: &str) -> String {
	// [Option("ws(s)"), ip, Option(port)]
	match url.split(':').count() {
		3 => url.to_string(),
		2 => {
			if url.contains("ws") {
				// url is of format ws://127.0.0.1, no port added
				format!("{}:{}", url, port)
			} else {
				// url is of format 127.0.0.1:4000, port was added
				url.to_string()
			}
		},
		1 => format!("{}:{}", url, port),
		_ => panic!("Invalid worker url format in url input {:?}", url),
	}
}

fn ensure_ws_or_wss(url_str: &str) -> String {
	let url = url::Url::parse(url_str)
		.map_err(|e| {
			println!("Parse url [{}] error: {}", url_str, e);
		})
		.unwrap();

	if url.scheme() != "wss" && url.scheme() != "ws" {
		panic!("Parse url [{}] error: expect ws or wss, but get {}", url_str, url.scheme());
	}
	url.into()
}

pub fn pwd() -> PathBuf {
	std::env::current_dir().expect("works on all supported platforms; qed.")
}

#[cfg(test)]
mod test {
	use super::*;
	use std::{assert_matches::assert_matches, collections::HashMap};

	#[test]
	fn check_correct_config_assignment_for_empty_input() {
		let empty_args = ArgMatches::default();
		let config = Config::from(&empty_args);
		let expected_worker_ip = "127.0.0.1";

		assert_eq!(config.litentry_rpc_url, DEFAULT_NODE_URL);
		assert_eq!(config.litentry_rpc_port, DEFAULT_NODE_PORT);
		assert_eq!(config.target_a_parentchain_rpc_url, None);
		assert_eq!(config.target_a_parentchain_rpc_port, None);
		assert_eq!(config.target_b_parentchain_rpc_url, None);
		assert_eq!(config.target_b_parentchain_rpc_port, None);
		assert_eq!(config.trusted_worker_port, DEFAULT_TRUSTED_PORT);
		assert_eq!(config.untrusted_worker_port, DEFAULT_UNTRUSTED_PORT);
		assert_eq!(config.mu_ra_port, DEFAULT_MU_RA_PORT);
		assert_eq!(config.worker_ip, expected_worker_ip);
		assert!(config.trusted_external_worker_address.is_none());
		assert!(config.untrusted_external_worker_address.is_none());
		assert!(config.mu_ra_external_address.is_none());
		assert!(!config.enable_metrics_server);
		assert_eq!(config.untrusted_http_port, DEFAULT_UNTRUSTED_HTTP_PORT);
		assert_eq!(config.data_dir, pwd());
		assert!(config.run_config.is_none());
		assert_eq!(config.mock_server_port, DEFAULT_MOCK_SERVER_PORT);
		assert_eq!(config.parentchain_start_block, DEFAULT_PARENTCHAIN_START_BLOCK);
		assert_matches!(config.fail_slot_mode, Option::None);
		assert_eq!(config.fail_at, DEFAULT_FAIL_AT.parse::<u64>().unwrap())
	}

	#[test]
	fn worker_ip_is_set_correctly_for_set_ws_external_flag() {
		let expected_worker_ip = "0.0.0.0";

		let mut args = ArgMatches::default();
		args.args = HashMap::from([("ws-external", Default::default())]);
		let config = Config::from(&args);

		assert_eq!(config.worker_ip, expected_worker_ip);
	}

	#[test]
	fn check_correct_config_assignment_for_given_input() {
		let node_ip = "ws://12.1.58.1";
		let node_port = "111111";
		let trusted_ext_addr = "wss://1.1.1.2:700";
		let trusted_port = "7119";
		let untrusted_ext_addr = "ws://1.723.3.1:11";
		let untrusted_port = "9119";
		let mu_ra_ext_addr = "1.1.3.1:1000";
		let mu_ra_port = "99";
		let untrusted_http_port = "4321";

		let mock_server_port = "19527";
		let parentchain_start_block = "30";

		let mut args = ArgMatches::default();
		args.args = HashMap::from([
			("node-url", Default::default()),
			("node-port", Default::default()),
			("ws-external", Default::default()),
			("trusted-external-address", Default::default()),
			("untrusted-external-address", Default::default()),
			("mu-ra-external-address", Default::default()),
			("mu-ra-port", Default::default()),
			("untrusted-worker-port", Default::default()),
			("trusted-worker-port", Default::default()),
			("untrusted-http-port", Default::default()),
			("mock-server-port", Default::default()),
			("parentchain-start-block", Default::default()),
		]);
		// Workaround because MatchedArg is private.
		args.args.get_mut("node-url").unwrap().vals = vec![node_ip.into()];
		args.args.get_mut("node-port").unwrap().vals = vec![node_port.into()];
		args.args.get_mut("trusted-external-address").unwrap().vals = vec![trusted_ext_addr.into()];
		args.args.get_mut("untrusted-external-address").unwrap().vals =
			vec![untrusted_ext_addr.into()];
		args.args.get_mut("mu-ra-external-address").unwrap().vals = vec![mu_ra_ext_addr.into()];
		args.args.get_mut("mu-ra-port").unwrap().vals = vec![mu_ra_port.into()];
		args.args.get_mut("untrusted-worker-port").unwrap().vals = vec![untrusted_port.into()];
		args.args.get_mut("trusted-worker-port").unwrap().vals = vec![trusted_port.into()];
		args.args.get_mut("untrusted-http-port").unwrap().vals = vec![untrusted_http_port.into()];
		args.args.get_mut("mock-server-port").unwrap().vals = vec![mock_server_port.into()];
		args.args.get_mut("parentchain-start-block").unwrap().vals =
			vec![parentchain_start_block.into()];

		let config = Config::from(&args);

		assert_eq!(config.litentry_rpc_url, node_ip);
		assert_eq!(config.litentry_rpc_port, node_port);
		assert_eq!(config.trusted_worker_port, trusted_port);
		assert_eq!(config.untrusted_worker_port, untrusted_port);
		assert_eq!(config.mu_ra_port, mu_ra_port);
		assert_eq!(config.trusted_external_worker_address, Some(trusted_ext_addr.to_string()));
		assert_eq!(config.untrusted_external_worker_address, Some(untrusted_ext_addr.to_string()));
		assert_eq!(config.mu_ra_external_address, Some(mu_ra_ext_addr.to_string()));
		assert_eq!(config.untrusted_http_port, untrusted_http_port.to_string());
		assert_eq!(config.mock_server_port, mock_server_port.to_string());
		assert_eq!(config.parentchain_start_block, parentchain_start_block.to_string());
	}

	#[test]
	fn default_run_config_is_correct() {
		let empty_args = ArgMatches::default();
		let run_config = RunConfig::from(&empty_args);

		assert_eq!(run_config.dev, false);
		assert_eq!(run_config.skip_ra, false);
		assert!(run_config.shard.is_none());
	}

	#[test]
	fn run_config_parsing_works() {
		let shard_identifier = "shard-identifier";

		let mut args = ArgMatches::default();
		args.args = HashMap::from([
			("dev", Default::default()),
			("skip-ra", Default::default()),
			("shard", Default::default()),
		]);
		// Workaround because MatchedArg is private.
		args.args.get_mut("shard").unwrap().vals = vec![shard_identifier.into()];

		let run_config = RunConfig::from(&args);

		assert_eq!(run_config.dev, true);
		assert_eq!(run_config.skip_ra, true);
		assert_eq!(run_config.shard.unwrap(), shard_identifier.to_string());
	}

	#[test]
	fn external_addresses_are_returned_correctly_if_not_set() {
		let trusted_port = "7119";
		let untrusted_port = "9119";
		let mu_ra_port = "99";
		let expected_worker_ip = "127.0.0.1";

		let mut args = ArgMatches::default();
		args.args = HashMap::from([
			("mu-ra-port", Default::default()),
			("untrusted-worker-port", Default::default()),
			("trusted-worker-port", Default::default()),
		]);
		// Workaround because MatchedArg is private.
		args.args.get_mut("mu-ra-port").unwrap().vals = vec![mu_ra_port.into()];
		args.args.get_mut("untrusted-worker-port").unwrap().vals = vec![untrusted_port.into()];
		args.args.get_mut("trusted-worker-port").unwrap().vals = vec![trusted_port.into()];

		let config = Config::from(&args);

		assert_eq!(
			config.trusted_worker_url_external(),
			format!("wss://{}:{}", expected_worker_ip, trusted_port)
		);
		assert_eq!(
			config.untrusted_worker_url_external(),
			format!("ws://{}:{}", expected_worker_ip, untrusted_port)
		);
		assert_eq!(config.mu_ra_url_external(), format!("{}:{}", expected_worker_ip, mu_ra_port));
	}

	#[test]
	fn external_addresses_are_returned_correctly_if_set() {
		let trusted_ext_addr = "wss://1.1.1.2:700/";
		let untrusted_ext_addr = "ws://1.123.3.1:11/";
		let mu_ra_ext_addr = "1.1.3.1:1000";

		let mut args = ArgMatches::default();
		args.args = HashMap::from([
			("trusted-external-address", Default::default()),
			("untrusted-external-address", Default::default()),
			("mu-ra-external-address", Default::default()),
		]);
		// Workaround because MatchedArg is private.
		args.args.get_mut("trusted-external-address").unwrap().vals = vec![trusted_ext_addr.into()];
		args.args.get_mut("untrusted-external-address").unwrap().vals =
			vec![untrusted_ext_addr.into()];
		args.args.get_mut("mu-ra-external-address").unwrap().vals = vec![mu_ra_ext_addr.into()];

		let config = Config::from(&args);

		assert_eq!(config.trusted_worker_url_external(), trusted_ext_addr);
		assert_eq!(config.untrusted_worker_url_external(), untrusted_ext_addr);
		assert_eq!(config.mu_ra_url_external(), mu_ra_ext_addr);
	}

	#[test]
	fn ensure_no_port_is_added_to_url_with_port() {
		let url = "ws://hello:4000";
		let port = "0";

		let resulting_url = add_port_if_necessary(url, port);

		assert_eq!(resulting_url, url);
	}

	#[test]
	fn ensure_port_is_added_to_url_without_port() {
		let url = "wss://hello";
		let port = "0";

		let resulting_url = add_port_if_necessary(url, port);

		assert_eq!(resulting_url, format!("{}:{}", url, port));
	}

	#[test]
	fn ensure_no_port_is_added_to_url_with_port_without_prefix() {
		let url = "hello:10001";
		let port = "012";

		let resulting_url = add_port_if_necessary(url, port);

		assert_eq!(resulting_url, url);
	}

	#[test]
	fn ensure_port_is_added_to_url_without_port_without_prefix() {
		let url = "hello_world";
		let port = "10";

		let resulting_url = add_port_if_necessary(url, port);

		assert_eq!(resulting_url, format!("{}:{}", url, port));
	}
}
