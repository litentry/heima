use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
	#[command(subcommand)]
	pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	Run(Box<RunArgs>),
	ExportBundlerKey(ExportBundlerKeyArgs),
}

#[derive(Args)]
pub struct RunArgs {
	#[arg(short, long, default_value = "local", value_name = "local directory path")]
	pub local_directory_path: String,
	#[arg(
		short,
		long,
		default_value = "0xc07cb79754cf3b252038e2713a138363d55df9e0",
		value_name = "delegation contract address"
	)]
	pub delegation_contract_address: String,
	#[arg(
		short,
		long,
		default_value = "0xb0830ef478a215ed393c20a0c97aa69869a0beea",
		value_name = "accounting contract address"
	)]
	pub accounting_contract_address: String,
	#[arg(
		short,
		long,
		default_value = "31weKQJQA9ZFYaVdnjtAXUoEGPW8UUFC5TEvgPJugAub",
		value_name = "solana accounting contract address"
	)]
	pub solana_accounting_contract_address: String,
	#[arg(short, long, default_value = "9090", value_name = "metrics port")]
	pub metrics_port: String,
	#[arg(long, default_value = "0", value_name = "threshold value in usdt")]
	pub instant_payout_threshold: String,
	#[arg(long, value_name = "enable mock server for testing")]
	pub enable_mock_server: bool,
	#[arg(long, default_value = "3456", value_name = "mock server port")]
	pub mock_server_port: u16,
}

#[derive(Args)]
pub struct ExportBundlerKeyArgs {
	#[arg(short, long, value_name = "path to authorized ECDSA private key file (32-byte seed)")]
	pub authorized_key_path: String,
	#[arg(short, long, default_value = "ws://127.0.0.1:3456", value_name = "worker WebSocket URL")]
	pub worker_url: String,
}
