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
	GenKey(GenKeyArgs),
	PrintMrEnclave(PrintMrEnclaveArgs),
}

#[derive(Args)]
pub struct RunArgs {
	pub parentchain_url: String,
	pub ethereum_url: String,
	pub solana_url: String,
	pub pumpx_signer_url: String,
	pub worker_url: String,
	pub bsc_url: String,
	pub bsc_testnet_url: Option<String>,
	#[arg(long, short = 'b', default_value = "0", help = "Start block to sync from parentchain")]
	pub start_block: u64,
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
	#[arg(long, value_name = "should sync with parentchain")]
	pub parentchain_sync: bool,
	#[arg(short, long, default_value = "9090", value_name = "metrics port")]
	pub metrics_port: String,
}

#[derive(Args)]
pub struct GenKeyArgs {
	#[arg(short, long, default_value = "local", value_name = "local directory path")]
	pub local_directory_path: String,
}

#[derive(Args)]
pub struct PrintMrEnclaveArgs {
	#[arg(short, long, default_value = "local", value_name = "local directory path")]
	pub local_directory_path: String,
}
