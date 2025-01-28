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
use ethereum_intent_executor::EthereumIntentExecutor;
use executor_storage::{init_storage, StorageDB};
use log::error;
use native_task_handler::{run_native_task_handler, ParentchainTxSigner, TaskHandlerContext};
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
use parentchain_signer::key_store::SubstrateKeyStore;
use parentchain_signer::TransactionSigner;
use rpc_server::{start_server as start_rpc_server, ShieldingKey};
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
			// TODO: move to config
			let jwt_secret = env::var("JWT_SECRET").unwrap_or("secret".to_string());
			let storage_db =
				init_storage(&args.parentchain_url).await.expect("Could not initialize storage");

			let client_factory = SubxtClientFactory::<CustomConfig>::new(&args.parentchain_url);
			let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
			let key_store = Arc::new(SubstrateKeyStore::new(args.keystore_path.clone()));
			let parentchain_rpc_client_factory = Arc::new(client_factory);
			let transaction_signer = Arc::new(TransactionSigner::new(
				metadata_provider,
				parentchain_rpc_client_factory.clone(),
				key_store.clone(),
			));
			let task_handler_context = TaskHandlerContext::new(
				parentchain_rpc_client_factory.clone(),
				transaction_signer.clone(),
				storage_db.clone(),
				jwt_secret.clone(),
			);
			// TODO: make buffer size configurable
			let buffer = 1024;
			let native_task_sender =
				run_native_task_handler(buffer, Arc::new(task_handler_context)).await;
			// TODO: get mrenclave from quote
			let mrenclave = [0u8; 32];

			start_rpc_server(
				&args.worker_rpc_port,
				parentchain_rpc_client_factory,
				ShieldingKey::new(),
				Arc::new(native_task_sender),
				storage_db.clone(),
				mrenclave,
				jwt_secret,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

			listen_to_parentchain(args, storage_db, transaction_signer, key_store)
				.await
				.unwrap();

			match signal::ctrl_c().await {
				Ok(()) => {},
				Err(err) => {
					eprintln!("Unable to listen for shutdown signal: {}", err);
					// we also shut down in case of error
				},
			}
		},
		Commands::GenKey(args) => {
			let key_store = Arc::new(SubstrateKeyStore::new(args.keystore_path));
			let _ = parentchain_signer::get_signer(key_store);
		},
	}

	Ok(())
}

async fn listen_to_parentchain(
	args: RunArgs,
	storage_db: Arc<StorageDB>,
	parentchain_tx_signer: Arc<ParentchainTxSigner>,
	key_store: Arc<SubstrateKeyStore>,
) -> Result<JoinHandle<()>, ()> {
	let (_sub_stop_sender, sub_stop_receiver) = oneshot::channel();
	let ethereum_intent_executor =
		EthereumIntentExecutor::new(&args.ethereum_url).map_err(|e| log::error!("{:?}", e))?;
	let solana_intent_executor =
		SolanaIntentExecutor::new(args.solana_url).map_err(|e| log::error!("{:?}", e))?;

	let mut parentchain_listener =
		parentchain_listener::create_listener::<EthereumIntentExecutor, SolanaIntentExecutor>(
			"litentry_rococo",
			Handle::current(),
			&args.parentchain_url,
			ethereum_intent_executor,
			solana_intent_executor,
			sub_stop_receiver,
			storage_db,
			parentchain_tx_signer,
			key_store,
			&args.log_path,
		)
		.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(args.start_block))
		.unwrap())
}
