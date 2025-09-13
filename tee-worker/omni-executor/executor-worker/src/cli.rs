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
}

#[derive(Args)]
pub struct RunArgs {
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
	#[arg(
		short,
		long,
		default_value = "31weKQJQA9ZFYaVdnjtAXUoEGPW8UUFC5TEvgPJugAub",
		value_name = "solana accounting contract address"
	)]
	pub solana_accounting_contract_address: String,
	#[arg(long, value_name = "should sync with parentchain")]
	pub parentchain_sync: bool,
	#[arg(short, long, default_value = "9090", value_name = "metrics port")]
	pub metrics_port: String,
	#[arg(long, default_value = "0", value_name = "threshold value in usdt")]
	pub instant_payout_threshold: String,
	#[arg(long, value_name = "enable mock server for testing")]
	pub enable_mock_server: bool,
	#[arg(long, default_value = "3456", value_name = "mock server port")]
	pub mock_server_port: u16,
	#[arg(long, value_name = "enable jaeger tracing")]
	pub enable_jaeger: bool,
	#[arg(long, default_value = "4317", value_name = "jaeger port")]
	pub jaeger_port: u16,
	#[arg(long, default_value = "localhost", value_name = "jaeger host")]
	pub jaeger_host: String,
	#[arg(long, value_name = "output traces to JSON file for later upload to Jaeger")]
	pub json_trace_output: bool,
	#[arg(long, default_value = "./traces.json", value_name = "JSON trace output file path")]
	pub json_trace_file: String,
}

#[derive(Args)]
pub struct GenKeyArgs {
	#[arg(short, long, default_value = "local", value_name = "local directory path")]
	pub local_directory_path: String,
}
