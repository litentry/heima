// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::cli::Cli;
use accounting_contract_client::AccountingContractClient;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use binance_api::BinanceApi;
use clap::Parser;
use cli::*;
use cross_chain_intent_executor::{Chain, CrossChainIntentExecutor, RpcEndpointRegistry};
use ethereum_intent_executor::EthereumIntentExecutor;
use executor_core::ecdsa_key_store::EcdsaKeyStore;
use executor_core::key_store::KeyStore;
use executor_core::shielding_key_store::ShieldingKeyStore;
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey};
use executor_crypto::{ecdsa, PairTrait};
use executor_primitives::AccountId;
use executor_storage::{init_storage, StorageDB};
use intent_core::IntentIdStore;
use intent_core::StorageDbIntentIdStore;
use log::{error, info};
use native_task_handler::{
	run_native_task_handler, Aes256KeyStore, TaskHandlerContext, MAX_CONCURRENT_TASKS,
};
use parentchain_attestation::perform_attestation;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::{
	CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClientFactory,
	ToPrimitiveType,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TxSigner};
use pumpx::pubkey_to_evm_address;
use pumpx::signer_client::SignerClient;
use pumpx::PumpxApi;
use rpc_server::{start_server as start_rpc_server, AuthTokenKeyStore};
use solana::SolanaClient;
use solana_intent_executor::SolanaIntentExecutor;
use std::env;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tokio::runtime::Handle;
use tokio::signal;
use tokio::sync::oneshot;

mod cli;

#[tokio::main]
async fn main() -> Result<(), ()> {
	env_logger::builder()
		.format(|buf, record| {
			let ts = buf.timestamp_micros();
			writeln!(
				buf,
				"{} [{}][{}][{}]: {}",
				ts,
				record.level(),
				std::thread::current().name().unwrap_or("none"),
				record.target(),
				record.args(),
			)
		})
		.init();

	let cli = Cli::parse();

	match cli.cmd {
		Commands::Run(args) => {
			let auth_token_key_store = AuthTokenKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/auth_token_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);
			let jwt_rsa_private_key = auth_token_key_store.read().expect("Could not read jwt key");

			let pumpx_auth_key_store = pumpx::auth_key_store::AuthKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/pumpx_auth_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let pumpx_signer_key =
				pumpx_auth_key_store.read().expect("Could not read PumpX signer key");

			let pumpx_signer_pair = ecdsa::Pair::from_seed_slice(&pumpx_signer_key).unwrap();
			info!("PumpX auth public key: {:?}", pumpx_signer_pair.public());

			let accounting_ecdsa_signer_key = EcdsaKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/accounting_ecdsa_signer_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let accounting_ecdsa_signer_key =
				accounting_ecdsa_signer_key.read().expect("Could not read accounting ecsa key");
			let accounting_ecdsa_signer_key_pair =
				ecdsa::Pair::from_seed_slice(&accounting_ecdsa_signer_key).unwrap();

			info!(
				"Accounting ecdsa signer address: {:?}",
				pubkey_to_evm_address(accounting_ecdsa_signer_key_pair.public().as_ref()).unwrap()
			);

			let storage_db =
				init_storage(&args.parentchain_url).await.expect("Could not initialize storage");

			let client_factory = SubxtClientFactory::<CustomConfig>::new(&args.parentchain_url);
			let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
			let parentchain_rpc_client_factory = Arc::new(client_factory);

			let substrate_key_store = Arc::new(SubstrateKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/substrate_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			));
			let parentchain_signer = parentchain_signer::get_signer(substrate_key_store.clone());
			let signer_account_id: AccountId =
				parentchain_signer.public_key().to_account_id().to_primitive_type();
			let mut parentchain_rpc_client = parentchain_rpc_client_factory
				.new_client()
				.await
				.expect("Could not create RPC client");
			let signer_account_nonce = parentchain_rpc_client
				.get_account_nonce(&signer_account_id)
				.await
				.expect("Could not get signer account nonce");

			let tx_signer = Arc::new(TxSigner::new(
				metadata_provider,
				parentchain_rpc_client_factory.clone(),
				parentchain_signer.clone(),
				signer_account_nonce,
			));
			let aes256_key_store = Aes256KeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/aes_256_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);
			let aes256_key = aes256_key_store.read().expect("Could not read aes256 key");

			let pumpx_signer_client: Arc<Box<dyn SignerClient>> =
				Arc::new(Box::new(pumpx::signer_client::PumpxSignerClient::new(
					args.pumpx_signer_url.clone(),
					pumpx_signer_pair,
				)));

			let ethereum_intent_executor =
				EthereumIntentExecutor::new(&args.ethereum_url, &args.delegation_contract_address)?;
			let solana_intent_executor = SolanaIntentExecutor::new(&args.solana_url)?;

			let mut rpc_endpoint_registry = RpcEndpointRegistry::new();
			rpc_endpoint_registry.insert(Chain::Solana, args.solana_url.clone());

			if let Some(ref bsc_url) = args.bsc_url {
				rpc_endpoint_registry.insert(Chain::Ethereum(56), bsc_url.to_owned());
			}

			if let Some(ref bsc_testnet_url) = args.bsc_testnet_url {
				rpc_endpoint_registry.insert(Chain::Ethereum(97), bsc_testnet_url.to_owned());
			}

			let pumpx_api_base_url = std::env::var("OE_PUMPX_API_BASE_URL").ok();
			let pumpx_api = Arc::new(PumpxApi::new(pumpx_api_base_url));

			let binance_api_key = env::var("OE_BINANCE_API_KEY").unwrap_or("".to_string());
			let binance_api_secret = env::var("OE_BINANCE_API_SECRET").unwrap_or("".to_string());
			let binance_api_base_url = env::var("OE_BINANCE_API_BASE_URL").ok();
			let binance_api = Arc::new(BinanceApi::new(
				binance_api_key,
				binance_api_secret,
				binance_api_base_url,
			));

			let solana_client = Arc::new(SolanaClient::new(&args.solana_url));

			let accounting_contract_signer =
				PrivateKeySigner::from_slice(&accounting_ecdsa_signer_key_pair.seed())
					.expect("Could not create accounting contract signer");
			let accounting_contract_wallet = EthereumWallet::from(accounting_contract_signer);

			let ethereum_rpc_provider = ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&args.ethereum_url,
				accounting_contract_wallet,
			);
			let accounting_contract_client = AccountingContractClient::new(
				ethereum_rpc_provider,
				args.accounting_contract_address.parse().unwrap(),
			);

			let cross_chain_intent_executor = CrossChainIntentExecutor::new(
				rpc_endpoint_registry,
				pumpx_signer_client.clone(),
				pumpx_api.clone(),
				storage_db.clone(),
				binance_api,
				solana_client,
				Arc::new(accounting_contract_client),
			)?;

			let intent_id_store: Arc<Box<dyn IntentIdStore>> =
				Arc::new(Box::new(StorageDbIntentIdStore::new(storage_db.clone())));

			let task_handler_context = TaskHandlerContext::new(
				parentchain_rpc_client_factory.clone(),
				tx_signer.clone(),
				storage_db.clone(),
				jwt_rsa_private_key.clone(),
				aes256_key,
				Arc::new(ethereum_intent_executor),
				Arc::new(solana_intent_executor),
				Arc::new(cross_chain_intent_executor),
				pumpx_api.clone(),
				pumpx_signer_client.clone(),
				intent_id_store.clone(),
			);
			// TODO: make buffer size configurable
			let native_task_sender =
				run_native_task_handler(MAX_CONCURRENT_TASKS, Arc::new(task_handler_context)).await;

			log::info!("worker url: {:?}", args.worker_url);
			let worker_url = url::Url::parse(&args.worker_url).expect("Invalid worker url");

			let shielding_key_store = ShieldingKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/shielding_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let shielding_key = shielding_key_store.read().expect("Could not read shielding key");
			let shielding_pubkey = shielding_key.public_key();
			let shielding_pubkey_vec = serde_json::to_vec(&Rsa3072PubKey {
				n: shielding_pubkey.n().to_bytes_le(),
				e: shielding_pubkey.e().to_bytes_le(),
			})
			.expect("Could not serialize shielding public key");

			let mrenclave = perform_attestation(
				parentchain_rpc_client_factory,
				parentchain_signer,
				tx_signer.clone(),
				worker_url.as_str(),
				shielding_pubkey_vec,
			)
			.await
			.map_err(|_| {
				error!("Could not perform attestation");
			})?;

			start_rpc_server(
				worker_url.port().expect("Missing worker port"),
				shielding_key,
				Arc::new(native_task_sender),
				pumpx_api,
				storage_db.clone(),
				mrenclave,
				jwt_rsa_private_key,
				intent_id_store,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

			listen_to_parentchain(*args, storage_db).await.unwrap();

			match signal::ctrl_c().await {
				Ok(()) => {},
				Err(err) => {
					eprintln!("Unable to listen for shutdown signal: {}", err);
					// we also shut down in case of error
				},
			}
		},
		Commands::GenKey(args) => {
			let key_store = Arc::new(SubstrateKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/substrate_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			));
			let _ = parentchain_signer::get_signer(key_store);
		},
	}

	Ok(())
}

async fn listen_to_parentchain(
	args: RunArgs,
	storage_db: Arc<StorageDB>,
) -> Result<JoinHandle<()>, ()> {
	let (_sub_stop_sender, sub_stop_receiver) = oneshot::channel();

	let mut parentchain_listener = parentchain_listener::create_listener(
		"heima",
		Handle::current(),
		&args.parentchain_url,
		sub_stop_receiver,
		storage_db,
		&Path::new(&args.local_directory_path)
			.join("log/parentchain_last_log.bin")
			.into_os_string()
			.into_string()
			.unwrap(),
	)
	.await?;

	Ok(thread::Builder::new()
		.name("heima_sync".to_string())
		.spawn(move || parentchain_listener.sync(args.start_block))
		.unwrap())
}
