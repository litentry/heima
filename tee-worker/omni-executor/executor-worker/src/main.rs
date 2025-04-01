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
use clap::Parser;
use cli::*;
use cross_chain_intent_executor::{Chain, CrossChainIntentExecutor, RpcEndpointRegistry};
use ethereum_intent_executor::EthereumIntentExecutor;
use executor_core::key_store::KeyStore;
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey};
use executor_crypto::{ecdsa, PairTrait};
use executor_storage::{init_storage, StorageDB};
use log::{error, info};
use native_task_handler::{run_native_task_handler, Aes256KeyStore, TaskHandlerContext};
use parentchain_attestation::perform_attestation;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
use parentchain_signer::key_store::SubstrateKeyStore;
use parentchain_signer::{get_signer, TransactionSigner};
use pumpx::PumpxApi;
use rpc_server::{start_server as start_rpc_server, AuthTokenKeyStore, ShieldingKey};
use solana_intent_executor::SolanaIntentExecutor;
use std::env;
use std::io::Write;
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
				"{} [{}][{}]: {}",
				ts,
				record.level(),
				std::thread::current().name().unwrap_or("none"),
				record.args(),
			)
		})
		.init();

	let cli = Cli::parse();

	match cli.cmd {
		Commands::Run(args) => {
			let _binance_api_key = env::var("OE_BINANCE_API_KEY").unwrap_or("".to_string());
			let auth_token_key_store =
				AuthTokenKeyStore::new(args.auth_token_key_store_path.clone());
			let jwt_rsa_private_key = auth_token_key_store.read().expect("Could not read jwt key");

			let pumpx_auth_key_store =
				pumpx::auth_key_store::AuthKeyStore::new(args.pumpx_auth_key_store_path.clone());

			let pumpx_signer_key =
				pumpx_auth_key_store.read().expect("Could not read PumpX signer key");

			let pumpx_signer_pair = ecdsa::Pair::from_seed_slice(&pumpx_signer_key).unwrap();
			info!("PumpX auth public key: {:?}", pumpx_signer_pair.public());

			let storage_db =
				init_storage(&args.parentchain_url).await.expect("Could not initialize storage");

			let client_factory = SubxtClientFactory::<CustomConfig>::new(&args.parentchain_url);
			let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
			let substrate_key_store =
				Arc::new(SubstrateKeyStore::new(args.substrate_keystore_path.clone()));
			let parentchain_rpc_client_factory = Arc::new(client_factory);
			let transaction_signer = Arc::new(TransactionSigner::new(
				metadata_provider,
				parentchain_rpc_client_factory.clone(),
				substrate_key_store.clone(),
			));
			let aes256_key_store = Aes256KeyStore::new(args.aes256_key_store_path.clone());
			let aes256_key = aes256_key_store.read().expect("Could not read aes256 key");

			let pumpx_signer_client = Arc::new(pumpx::signer_client::SignerClient::new(
				args.pumpx_signer_url.clone(),
				pumpx_signer_pair,
			));

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

			let cross_chain_intent_executor = CrossChainIntentExecutor::new(
				parentchain_rpc_client_factory.clone(),
				transaction_signer.clone(),
				rpc_endpoint_registry,
				pumpx_signer_client.clone(),
			)?;

			let pumpx_api_base_url = std::env::var("OE_PUMPX_API_BASE_URL").ok();
			let pumpx_api = PumpxApi::new(pumpx_api_base_url);

			let task_handler_context = TaskHandlerContext::new(
				parentchain_rpc_client_factory.clone(),
				transaction_signer.clone(),
				storage_db.clone(),
				jwt_rsa_private_key.clone(),
				aes256_key,
				Arc::new(ethereum_intent_executor),
				Arc::new(solana_intent_executor),
				Arc::new(cross_chain_intent_executor),
				Arc::new(pumpx_api),
			);
			// TODO: make buffer size configurable
			let buffer = 1024;
			let native_task_sender =
				run_native_task_handler(buffer, Arc::new(task_handler_context)).await;

			let signer = get_signer(substrate_key_store.clone());

			log::info!("worker url: {:?}", args.worker_url);
			let worker_url = url::Url::parse(&args.worker_url).expect("Invalid worker url");

			let shielding_key = ShieldingKey::new();
			let shielding_pubkey = shielding_key.public_key();
			let shielding_pubkey_vec = serde_json::to_vec(&Rsa3072PubKey {
				n: shielding_pubkey.n().to_bytes_le(),
				e: shielding_pubkey.e().to_bytes_le(),
			})
			.expect("Could not serialize shielding public key");

			let mrenclave = perform_attestation(
				parentchain_rpc_client_factory.clone(),
				signer,
				transaction_signer.clone(),
				worker_url.as_str(),
				shielding_pubkey_vec,
			)
			.await
			.map_err(|_| {
				error!("Could not perform attestation");
			})?;

			start_rpc_server(
				worker_url.port().expect("Missing worker port"),
				parentchain_rpc_client_factory,
				shielding_key,
				Arc::new(native_task_sender),
				storage_db.clone(),
				mrenclave,
				jwt_rsa_private_key,
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
			let key_store = Arc::new(SubstrateKeyStore::new(args.substrate_keystore_path));
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
		"litentry_rococo",
		Handle::current(),
		&args.parentchain_url,
		sub_stop_receiver,
		storage_db,
		&args.log_path,
	)
	.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(args.start_block))
		.unwrap())
}
