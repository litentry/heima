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
fn test_set_admin() {
    let program_id = "31weKQJQA9ZFYaVdnjtAXUoEGPW8UUFC5TEvgPJugAub";
    let payer = Keypair::new();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let program_pubkey = program.id();

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
        .accounts(first::accounts::SetAdmin {
            admin_account: Pubkey::find_program_address(&[b"admin"], &program_id).0,
            caller: payer.pubkey(),
            program_account: address,
            system_program: anchor_client::solana_sdk::system_program::ID,
        })
        .args(first::instruction::SetAdmin {
            new_admin: payer.pubkey(),
        })
        .send()
        .expect("Failed to send transaction");

    let tx = program
        .request()
        .accounts(first::accounts::SetWorker {
            worker_account: Pubkey::find_program_address(&[b"worker"], &program_id).0,
            caller: payer.pubkey(),
            admin_account: Pubkey::find_program_address(&[b"admin"], &program_id).0,
            system_program: anchor_client::solana_sdk::system_program::ID,
        })
        .args(first::instruction::SetWorker {
            new_worker: payer.pubkey(),
        })
        .send()
        .expect("Failed to send transaction");

    println!("Your transaction signature: {}", tx);
}
