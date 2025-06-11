use anchor_client::anchor_lang::Space;
use std::error::Error;
use std::str::FromStr;

use accounting_contract::accounting_contract::set_worker;
use anchor_client::anchor_lang::AnchorDeserialize;
use anchor_client::solana_sdk::instruction::InstructionError::Custom;
use anchor_client::solana_sdk::program_error::ProgramError;
use anchor_client::solana_sdk::signature::SeedDerivable;
use anchor_client::solana_sdk::transaction::TransactionError;
use anchor_client::solana_sdk::transaction::TransactionError::InstructionError;
use anchor_client::ClientError::SolanaClientError;
use anchor_client::{
	anchor_lang::{AccountDeserialize, ProgramData},
	solana_sdk::{
		bpf_loader_upgradeable::{UpgradeableLoaderState, ID as BPF_LOADER_UPGRADEABLE_ID},
		commitment_config::CommitmentConfig,
		pubkey::Pubkey,
		signature::{read_keypair_file, Keypair, Signer},
	},
	Client, ClientError, Cluster, Program,
};
use anchor_client::solana_sdk::rent::Rent;

// PK - 8QQds7P14EL1ZFjPLsTg2AHZaQHfNGH1EDW8wMhJmxaX
pub fn setup_program<'a>(payer: &'a Keypair) -> Program<&'a Keypair> {
	let program_id = "D3S1ZTrFNkfeoHaLSTAjMXZVXnRJvsNnbwh9k5mRYqqV";
	let client = setup_client(program_id, payer);

	// Ensure the payer has tokens
	let program_id = Pubkey::from_str(program_id).unwrap();
	let program = client.program(program_id).unwrap();
	program
}

pub fn setup_client<'a>(program_id: &str, payer: &'a Keypair) -> Client<&'a Keypair> {
	Client::new_with_options(Cluster::Localnet, payer, CommitmentConfig::confirmed())
}

pub fn get_account_data<T: AccountDeserialize + AnchorDeserialize + serde::de::DeserializeOwned>(
	program: &Program<&Keypair>,
	seeds: &[&[u8]],
) -> T {
	let account_admin = program
		.rpc()
		.get_account(&Pubkey::find_program_address(seeds, &program.id()).0)
		.unwrap();

	bincode::deserialize::<T>(&account_admin.data[8..]).unwrap()
}

pub fn set_admin(program: &Program<&Keypair>, payer: &Keypair) -> Result<(), ()> {
	let program_pubkey = program.id();
	// Fetch program_data address
	let account = program.rpc().get_account(&program_pubkey).unwrap();
	if account.owner != BPF_LOADER_UPGRADEABLE_ID {
		println!("Not an upgradable program")
	}

	let upgradeable_state: UpgradeableLoaderState =
		bincode::deserialize(&account.data).ok().unwrap();

	let address: Pubkey =
		if let UpgradeableLoaderState::Program { programdata_address } = upgradeable_state {
			programdata_address
		} else {
			program_pubkey
		};

	program
		.request()
		.accounts(accounting_contract::accounts::SetAdmin {
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			signer: payer.pubkey(),
			program_account: address,
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::SetAdmin { new_admin: payer.pubkey() })
		.send()
		.map_err(|_| ())?;

	Ok(())
}

pub fn set_worker_account(program: &Program<&Keypair>, payer: &Keypair) -> Result<(), ()> {
	program
		.request()
		.accounts(accounting_contract::accounts::SetWorker {
			worker_account: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			signer: payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::SetWorker { new_worker: payer.pubkey() })
		.send()
		.map_err(|_| ())?;

	Ok(())
}

pub fn request_airdrop(program: &Program<&Keypair>, payer: &Keypair) {
	let balance = program.rpc().get_balance(&payer.pubkey()).expect("Failed to fetch balance");
	if balance == 0 {
		program
			.rpc()
			.request_airdrop(&payer.pubkey(), 1_000_000_000)
			.expect("Failed to request airdrop");
	}
}

#[test]
fn test_set_admin() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);

	// Request airdrop for payer
	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	let admin = get_account_data::<accounting_contract::AdminAccount>(&program, &[b"admin"]);

	assert_eq!(payer.pubkey(), admin.admin);
}

#[test]
fn test_set_worker() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);

	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let worker = get_account_data::<accounting_contract::WorkerAccount>(&program, &[b"worker"]);
	assert_eq!(worker.worker, payer.pubkey())
}

// Caller of the set_worker_function is not the admin
#[test]
fn test_set_worker_unauthorized() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let seed_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
	let seed_bytes = hex::decode(seed_hex).unwrap();
	let secret: [u8; 32] = seed_bytes.clone().try_into().unwrap();

	let second_payer = Keypair::from_seed(secret.as_ref()).unwrap();
	let program = setup_program(&payer);
	let second_program = setup_program(&second_payer);

	assert_eq!(program.id(), second_program.id());
	request_airdrop(&program, &payer);
	request_airdrop(&second_program, &second_payer);

	set_admin(&program, &payer).unwrap();

	let result = second_program
		.request()
		.accounts(accounting_contract::accounts::SetWorker {
			worker_account: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			signer: second_payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::SetWorker { new_worker: payer.pubkey() })
		.send();

	if let Err(SolanaClientError(e)) = result {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6000))));
	}
}

#[test]
fn test_deposit_funds() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);

	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let _ = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program.id());
	let initial_treasury_balance = program.rpc().get_balance(&treasury_pda).unwrap();

	let _ = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let final_treasury_balance = program.rpc().get_balance(&treasury_pda).unwrap();
	assert_eq!(final_treasury_balance - initial_treasury_balance, 1_000_000_000);
}

#[test]
fn test_create_pay_request() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);
	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let nonce: u64 = 1;
	let keypair = Keypair::new();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: keypair.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 1_000_000,
			nonce: 1_u64,
		})
		.send()
		.expect("Failed to send transaction");
}

#[test]
fn test_create_pay_request_invalid_nonce() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);
	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let nonce: u64 = 1;
	let keypair = Keypair::new();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: keypair.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 1_000_000,
			nonce: 1_u64,
		});

	tx.send().unwrap();
	let nonce: u64 = 3;
	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: keypair.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 1_000_000,
			nonce: 3_u64,
		})
		.send();

	if let Err(SolanaClientError(e)) = tx {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6004))));
	}
}

#[test]
fn create_pay_request_out_of_balance() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);
	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();
	let nonce: u64 = 1;
	let keypair = Keypair::new();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: keypair.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 2_000_000_000,
			nonce: 1_u64,
		})
		.send();

	if let Err(SolanaClientError(e)) = tx {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6003))));
	}
}

#[test]
fn create_pay_request_unauthorized() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let seed_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
	let seed_bytes = hex::decode(seed_hex).unwrap();
	let secret: [u8; 32] = seed_bytes.clone().try_into().unwrap();

	let payer = Keypair::from_seed(secret.as_ref()).unwrap();
	let program = setup_program(&payer);

	request_airdrop(&program, &payer);

	let nonce: u64 = 1;
	let keypair = Keypair::new();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 1_000_000_000,
			nonce: 1_u64,
		})
		.send();

	if let Err(SolanaClientError(e)) = tx {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6000))));
	}
}

#[test]
fn test_withdraw() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::WithdrawFunds {
			signer: payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::WithdrawFunds { amount: 1_000_000 })
		.send()
		.expect("Failed to send transaction");
}

#[test]
fn test_withdraw_out_of_funds() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::WithdrawFunds {
			signer: payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::WithdrawFunds { amount: 2_000_000_000 })
		.send();

	if let Err(SolanaClientError(e)) = tx {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6003))));
	}
}

#[test]
fn test_withdraw_unauthorized() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	request_airdrop(&program, &payer);

	set_admin(&program, &payer).unwrap();
	set_worker_account(&program, &payer).unwrap();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let seed_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
	let seed_bytes = hex::decode(seed_hex).unwrap();
	let secret: [u8; 32] = seed_bytes.clone().try_into().unwrap();

	let payer = Keypair::from_seed(secret.as_ref()).unwrap();
	let program = setup_program(&payer);

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::WithdrawFunds {
			signer: payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::WithdrawFunds { amount: 1_000_000_000 })
		.send();

	if let Err(SolanaClientError(e)) = tx {
		assert_eq!(e.get_transaction_error(), Some(InstructionError(0, Custom(6000))));
	}
}

#[test]
fn test_full_workerflow() {
	let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
	let program = setup_program(&payer);
	let program_pubkey = program.id();

	let balance = program.rpc().get_balance(&payer.pubkey()).expect("Failed to fetch balance");

	if balance == 0 {
		program
			.rpc()
			.request_airdrop(&payer.pubkey(), 1_000_000_000)
			.expect("Failed to airdrop SOL to payer");
	}

	let account = program.rpc().get_account(&program_pubkey).unwrap();
	if account.owner != BPF_LOADER_UPGRADEABLE_ID {
		println!("Not an upgradable program")
	}

	let upgradeable_state: UpgradeableLoaderState =
		bincode::deserialize(&account.data).ok().unwrap();

	let address: Pubkey =
		if let UpgradeableLoaderState::Program { programdata_address } = upgradeable_state {
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
		.args(accounting_contract::instruction::SetAdmin { new_admin: payer.pubkey() })
		.send()
		.expect("Failed to send transaction");

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::SetWorker {
			worker_account: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			signer: payer.pubkey(),
			admin_account: Pubkey::find_program_address(&[b"admin"], &program.id()).0,
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::SetWorker { new_worker: payer.pubkey() })
		.send()
		.expect("Failed to send transaction");

	let account_admin = program
		.rpc()
		.get_account(&Pubkey::find_program_address(&[b"admin"], &program.id()).0)
		.unwrap();
	let admin_struct: accounting_contract::AdminAccount =
		bincode::deserialize(&account_admin.data[8..]).unwrap();

	let payer_account = program.rpc().get_account(&payer.pubkey()).unwrap();

	assert_eq!(payer.pubkey(), admin_struct.admin);

	let account_worker = program
		.rpc()
		.get_account(&Pubkey::find_program_address(&[b"worker"], &program.id()).0)
		.unwrap();
	let worker_struct: accounting_contract::WorkerAccount =
		bincode::deserialize(&account_worker.data[8..]).unwrap();

	assert_eq!(payer.pubkey(), worker_struct.worker);

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program.id());
	let initial_treasury_balance = program.rpc().get_balance(&treasury_pda).unwrap();

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::DepositFunds {
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			signer: payer.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::DepositFunds { amount: 1_000_000_000 })
		.send()
		.expect("Failed to send transaction");

	let (treasury_pda, _) = Pubkey::find_program_address(&[b"treasury"], &program.id());
	let final_treasury_balance = program.rpc().get_balance(&treasury_pda).unwrap();

	assert_eq!(final_treasury_balance - initial_treasury_balance, 1_000_000_000);

	let payer_account = program.rpc().get_account(&payer.pubkey()).unwrap();
	let initial_balance = payer_account.lamports;

	let keypair = Keypair::new();
	let nonce: u64 = 1;

	let tx = program
		.request()
		.accounts(accounting_contract::accounts::CreatePayRequest {
			pay_out_request: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
				&program.id(),
			)
			.0,
			account_nonce: Pubkey::find_program_address(
				&[keypair.pubkey().to_bytes().as_ref(), b"nonce"],
				&program.id(),
			)
			.0,
			signer: payer.pubkey(),
			worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
			treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
			beneficiary: keypair.pubkey(),
			system_program: anchor_client::solana_sdk::system_program::ID,
		})
		.args(accounting_contract::instruction::CreatePayRequest {
			amount: 1_000_000,
			nonce: 1_u64,
		})
		.send()
		.expect("Failed to send transaction");

	let payer_account = program.rpc().get_account(&keypair.pubkey()).unwrap();
	let final_balance = payer_account.lamports;

	assert_eq!(final_balance, 1_000_000);
}
