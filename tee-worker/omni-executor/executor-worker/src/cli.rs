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
	Run(RunArgs),
	GenKey(GenKeyArgs),
}

#[derive(Args)]
pub struct RunArgs {
	pub parentchain_url: String,
	pub ethereum_url: String,
	pub solana_url: String,
	pub worker_url: String,
	#[arg(long, short = 'b', default_value = "0", help = "Start block to sync from parentchain")]
	pub start_block: u64,
	#[arg(
		short,
		long,
		default_value = "local/keystore/substrate_alice.bin",
		value_name = "keystore file path"
	)]
	pub substrate_keystore_path: String,
	#[arg(
		short,
		long,
		default_value = "local/keystore/aes_256_key.bin",
		value_name = "Aes256 keystore file path"
	)]
	pub aes256_key_store_path: String,
	#[arg(
		short,
		long,
		default_value = "local/log/parentchain_last_log.bin",
		value_name = "log file path"
	)]
	pub log_path: String,
	#[arg(
		short,
		long,
		default_value = "0xc07cb79754cf3b252038e2713a138363d55df9e0",
		value_name = "delegation contract address"
	)]
	pub delegation_contract_address: String,
}

#[derive(Args)]
pub struct GenKeyArgs {
	#[arg(
		short,
		long,
		default_value = "local/keystore/substrate_alice.bin",
		value_name = "keystore file path"
	)]
	pub substrate_keystore_path: String,
}
