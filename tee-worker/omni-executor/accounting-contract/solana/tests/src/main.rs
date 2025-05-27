use anchor_client::anchor_lang::prelude::{Pubkey, UpgradeableLoaderState};
use anchor_client::anchor_lang::{AccountDeserialize, AnchorDeserialize};
use anchor_client::solana_sdk::bpf_loader_upgradeable::ID as BPF_LOADER_UPGRADEABLE_ID;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use anchor_client::{Client, Cluster, Program};
use clap::{Parser, Subcommand};
use std::env;
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
	#[arg(long, help = "Path to keypair file of the payer")]
	pub wallet_path: String,

	#[arg(long, help = "Deploy program_id of the solana program")]
	pub program_id: String,

	#[arg(long, help = "Admin pubkey to be set")]
	pub admin: Option<String>,

	#[arg(long, help = "Worker pubkey to be set")]
	pub worker: Option<String>,

	#[arg(long, help = "Amount to be used")]
	pub amount: Option<u64>,

	#[arg(long, help = "Beneficiary pubkey")]
	pub beneficiary: Option<String>,

	#[command(subcommand)]
	pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	#[command(about = "Deposit funds into the treasury account")]
	DepositFunds,

	#[command(about = "Withdraw funds from the treasury account to the payer")]
	WithdrawFunds,

	#[command(about = "Set a new admin public key")]
	SetAdmin,

	#[command(about = "Set a new worker public key")]
	SetWorker,

	#[command(about = "Create a payout request for a beneficiary")]
	CreatePayRequest,

	#[command(about = "Check the current balance of the treasury account")]
	ProgramBalance,

	#[command(about = "List all payout requests made for a beneficiary")]
	UserPayouts,

	#[command(about = "Check the current nonce for a beneficiary")]
	UserNonce,

	#[command(about = "Verify if the provided admin pubkey is set")]
	IsAdmin,

	#[command(about = "Verify if the provided worker pubkey is set")]
	IsWorker,
}

fn main() {
	let cli = Cli::parse();

	let wallet_path = cli.wallet_path.clone();
	let program_id = cli.program_id.clone();

	println!("Using wallet at path: {}", wallet_path);
	println!("Interacting with program: {}", program_id);

	let payer = read_keypair_file(wallet_path).expect("Failed to read keypair file");
	let client = Client::new_with_options(Cluster::Devnet, &payer, CommitmentConfig::confirmed());

	// Ensure the payer has tokens
	let program = client.program(Pubkey::from_str(&program_id).unwrap()).unwrap();

	match cli.cmd {
		Commands::SetAdmin => {
			let admin =
				Pubkey::from_str(cli.admin.as_ref().expect("admin must be passed as argument"))
					.unwrap();

			let program_pubkey = program.id();
			// Fetch program_data address
			let account = program.rpc().get_account(&program_pubkey).unwrap();
			if account.owner != BPF_LOADER_UPGRADEABLE_ID {
				println!("Not an upgradable program")
			}

			let upgradeable_state: UpgradeableLoaderState =
				bincode::deserialize(&account.data).ok().unwrap();

			let address: Pubkey = if let UpgradeableLoaderState::Program { programdata_address } =
				upgradeable_state
			{
				programdata_address
			} else {
				program_pubkey
			};

			let tx = program
				.request()
				.accounts(accounting_contract::accounts::SetAdmin {
					admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
					signer: payer.pubkey(),
					program_account: address,
					system_program: anchor_client::solana_sdk::system_program::ID,
				})
				.args(accounting_contract::instruction::SetAdmin { new_admin: admin })
				.send()
				.unwrap();

			println!("Transaction confirmed, Signature: {}", tx);
		},
		Commands::SetWorker => {
			let worker =
				Pubkey::from_str(cli.worker.as_ref().expect("worker must be passed as argument"))
					.unwrap();

			let tx = program
				.request()
				.accounts(accounting_contract::accounts::SetWorker {
					worker_account: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
					signer: payer.pubkey(),
					admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
					system_program: anchor_client::solana_sdk::system_program::ID,
				})
				.args(accounting_contract::instruction::SetWorker { new_worker: worker })
				.send()
				.unwrap();

			println!("Transaction is confirmed, signature: {}", tx);
		},
		Commands::IsWorker => {
			let worker =
				Pubkey::from_str(cli.worker.as_ref().expect("worker must be passed as argument"))
					.unwrap();

			let worker_account =
				get_account_data::<accounting_contract::WorkerAccount>(&program, &[b"worker"])
					.unwrap();
			if worker_account.worker == worker {
				println!("{} is Worker", worker);
			} else {
				println!("{} is not the worker, the worker is: {}", worker, worker_account.worker);
			}
		},
		Commands::IsAdmin => {
			let admin =
				Pubkey::from_str(cli.admin.as_ref().expect("admin must be passed as argument"))
					.unwrap();

			let admin_account =
				get_account_data::<accounting_contract::AdminAccount>(&program, &[b"admin"])
					.unwrap();
			if admin_account.admin == admin {
				println!("{} Is Admin", admin);
			} else {
				println!("{} is not the admin, the admin is: {}", admin, admin_account.admin);
			}
		},
		Commands::DepositFunds => {
			let amount: u64 = cli.amount.expect("amount must be passed as argument");

			let tx = program
				.request()
				.accounts(accounting_contract::accounts::DepositFunds {
					treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
					signer: payer.pubkey(),
					system_program: anchor_client::solana_sdk::system_program::ID,
				})
				.args(accounting_contract::instruction::DepositFunds { amount })
				.send()
				.unwrap();

			println!("The transaction is confirmed, signature: {}", tx);

			let treasury_account = program
				.rpc()
				.get_account(&Pubkey::find_program_address(&[b"treasury"], &program.id()).0)
				.unwrap();

			println!("The balance of the treasury account: {}", treasury_account.lamports);
		},
		Commands::ProgramBalance => {
			let treasury_account = program
				.rpc()
				.get_account(&Pubkey::find_program_address(&[b"treasury"], &program.id()).0)
				.unwrap();

			println!("The balaance of the program is: {}", treasury_account.lamports);
		},
		Commands::WithdrawFunds => {
			let amount: u64 = cli.amount.expect("amount must be passed as argument");

			let tx = program
				.request()
				.accounts(accounting_contract::accounts::WithdrawFunds {
					signer: payer.pubkey(),
					admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
					treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
					beneficiary: payer.pubkey(),
					system_program: anchor_client::solana_sdk::system_program::ID,
				})
				.args(accounting_contract::instruction::WithdrawFunds { amount })
				.send()
				.unwrap();

			println!("The transaction is confirmed, signature: {}", tx);
		},
		Commands::CreatePayRequest => {
			let amount: u64 = cli.amount.expect("amount must be passed as argument");
			let beneficiary = Pubkey::from_str(
				cli.beneficiary.as_ref().expect("beneficiary must be passed as argument"),
			)
			.unwrap();

			let nonce_account = get_account_data::<accounting_contract::Nonce>(
				&program,
				&[beneficiary.to_bytes().as_ref(), b"nonce"],
			)
			.unwrap_or_else(|e| accounting_contract::Nonce { nonce: 0 });
			let nonce = nonce_account.nonce + 1;

			let tx = program
				.request()
				.accounts(accounting_contract::accounts::CreatePayRequest {
					pay_out_request: Pubkey::find_program_address(
						&[beneficiary.to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
						&program.id(),
					)
					.0,
					account_nonce: Pubkey::find_program_address(
						&[beneficiary.to_bytes().as_ref(), b"nonce"],
						&program.id(),
					)
					.0,
					signer: payer.pubkey(),
					worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
					treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
					beneficiary,
					system_program: anchor_client::solana_sdk::system_program::ID,
				})
				.args(accounting_contract::instruction::CreatePayRequest { amount, nonce })
				.send()
				.expect("Failed to send transaction");

			println!("Transaction has been confirmed, signature: {}", tx);
		},
		Commands::UserNonce => {
			let beneficiary = Pubkey::from_str(
				cli.beneficiary.as_ref().expect("beneficiary must be passed as argument"),
			)
			.unwrap();

			let nonce_account = get_account_data::<accounting_contract::Nonce>(
				&program,
				&[beneficiary.to_bytes().as_ref(), b"nonce"],
			)
			.unwrap();
			let nonce = nonce_account.nonce;

			println!("The stored nonce is: {}", nonce);
		},
		Commands::UserPayouts => {
			let beneficiary = Pubkey::from_str(
				cli.beneficiary.as_ref().expect("beneficiary must be passed as argument"),
			)
			.unwrap();

			let nonce_account = get_account_data::<accounting_contract::Nonce>(
				&program,
				&[beneficiary.to_bytes().as_ref(), b"nonce"],
			)
			.unwrap();
			let nonce = nonce_account.nonce;

			for x in 1..nonce as usize + 1 {
				let current_nonce = x as u64;
				let payout = get_account_data::<accounting_contract::PayoutRequest>(
					&program,
					&[
						beneficiary.to_bytes().as_ref(),
						current_nonce.to_le_bytes().as_ref(),
						b"payout_request",
					],
				)
				.unwrap();
				println!("Payout Request No. {}, Amount: {}", payout.nonce, payout.amount);
			}

			if nonce == 0 {
				println!("There have been no payouts for this user");
			}
		},
	}
}

pub fn get_account_data<T: AccountDeserialize + AnchorDeserialize + serde::de::DeserializeOwned>(
	program: &Program<&Keypair>,
	seeds: &[&[u8]],
) -> Result<T, ()> {
	let account_admin = program
		.rpc()
		.get_account(&Pubkey::find_program_address(seeds, &program.id()).0)
		.map_err(|_| ())?;

	bincode::deserialize::<T>(&account_admin.data[8..]).map_err(|_| ())
}
