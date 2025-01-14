use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
	pub parentchain_url: String,
	pub ethereum_url: String,
	pub solana_url: String,
	pub worker_rpc_port: String,
	pub start_block: u64,
}
