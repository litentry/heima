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
	#[arg(short, long, default_value = "0", value_name = "start block to sync from parentchain")]
	pub start_block: u64,
	#[arg(
		short,
		long,
		default_value = "local/keystore/substrate_alice.bin",
		value_name = "keystore file path"
	)]
	pub keystore_path: String,
	#[arg(
		short,
		long,
		default_value = "local/log/parentchain_last_log.bin",
		value_name = "log file path"
	)]
	pub log_path: String,
}

#[derive(Args)]
pub struct GenKeyArgs {
	#[arg(
		short,
		long,
		default_value = "local/keystore/substrate_alice.bin",
		value_name = "keystore file path"
	)]
	pub keystore_path: String,
}
