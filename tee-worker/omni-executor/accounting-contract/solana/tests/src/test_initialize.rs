use std::str::FromStr;

use anchor_client::{
    anchor_lang::{AccountDeserialize, ProgramData},
    solana_sdk::{
        bpf_loader_upgradeable::{UpgradeableLoaderState, ID as BPF_LOADER_UPGRADEABLE_ID},
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
    },
    Client, Cluster, Program,
};

#[test]
fn test_full_workerflow() {
    let program_id = "31weKQJQA9ZFYaVdnjtAXUoEGPW8UUFC5TEvgPJugAub";
    // PK - 8QQds7P14EL1ZFjPLsTg2AHZaQHfNGH1EDW8wMhJmxaX
    let payer = read_keypair_file("../test-account.json").expect("Failed to read keypair file");
    // let payer = Keypair::new();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());

    // Ensure the payer has tokens
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let program_pubkey = program.id();

    let balance = program
        .rpc()
        .get_balance(&payer.pubkey())
        .expect("Failed to fetch balance");

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

    let address: Pubkey = if let UpgradeableLoaderState::Program {
        programdata_address,
    } = upgradeable_state
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
        .args(accounting_contract::instruction::SetAdmin {
            new_admin: payer.pubkey(),
        })
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
        .args(accounting_contract::instruction::SetWorker {
            new_worker: payer.pubkey(),
        })
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
        .args(accounting_contract::instruction::DepositFunds {
            amount: 1_000_000_000,
        })
        .send()
        .expect("Failed to send transaction");

    let treasury_account = program
        .rpc()
        .get_account(&Pubkey::find_program_address(&[b"treasury"], &program.id()).0)
        .unwrap();
    let treasury_struct: accounting_contract::TreasuryAccount =
        bincode::deserialize(&treasury_account.data[8..]).unwrap();

    // 1002240 is the initial balance of the treasury account
    assert_eq!(treasury_account.lamports, 1000946560);
    // assert_eq!(treasury_struct.balance, 1_000_000_000);

    let payer_account = program.rpc().get_account(&payer.pubkey()).unwrap();
    let initial_balance = payer_account.lamports;

    let keypair = Keypair::new();
    let nonce: u64 = 1;

    let tx = program
        .request()
        .accounts(accounting_contract::accounts::CreatePayRequest {
            pay_out_request: Pubkey::find_program_address(
                &[
                    keypair.pubkey().to_bytes().as_ref(),
                    &nonce.to_le_bytes(),
                    b"payout_request",
                ],
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
