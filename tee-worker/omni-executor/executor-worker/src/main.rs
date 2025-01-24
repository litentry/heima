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
use ethereum_intent_executor::EthereumIntentExecutor;
use log::error;
use native_task_handler::{run_native_task_handler, TaskHandlerContext};
use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
use rpc_server::{start_server as start_rpc_server, ShieldingKey};
use solana_intent_executor::SolanaIntentExecutor;
use std::env;
use std::io::Write;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{fs, thread};
use storage::{init_storage, StorageDB};
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

	fs::create_dir_all("data/").map_err(|e| {
		error!("Could not create data dir: {:?}", e);
	})?;

	// TODO: move to config
	let jwt_secret = env::var("JWT_SECRET").unwrap_or("secret".to_string());
	let storage_db =
		init_storage(&cli.parentchain_url).await.expect("Could not initialize storage");
	let client_factory = Arc::new(SubxtClientFactory::<CustomConfig>::new(&cli.parentchain_url));
	let rpc_client = client_factory.new_client_until_connected().await;
	let parentchain_rpc_client = Arc::new(rpc_client);

	let task_handler_context = TaskHandlerContext {
		parentchain_rpc_client: parentchain_rpc_client.clone(),
		storage_db: storage_db.clone(),
		jwt_secret: jwt_secret.clone(),
	};

	// TODO: make buffer size configurable
	let buffer = 1024;
	let native_task_sender = run_native_task_handler(buffer, Arc::new(task_handler_context)).await;
	// TODO: get mrenclave from quote
	let mrenclave = [0u8; 32];

	start_rpc_server(
		&cli.worker_rpc_port,
		parentchain_rpc_client,
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

	listen_to_parentchain(
		cli.parentchain_url,
		cli.ethereum_url,
		cli.solana_url,
		cli.start_block,
		storage_db,
	)
	.await
	.unwrap();

	match signal::ctrl_c().await {
		Ok(()) => {},
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
			// we also shut down in case of error
		},
	}

	Ok(())
}

async fn listen_to_parentchain(
	parentchain_url: String,
	ethereum_url: String,
	solana_url: String,
	start_block: u64,
	storage_db: Arc<StorageDB>,
) -> Result<JoinHandle<()>, ()> {
	let (_sub_stop_sender, sub_stop_receiver) = oneshot::channel();
	let ethereum_intent_executor =
		EthereumIntentExecutor::new(&ethereum_url).map_err(|e| log::error!("{:?}", e))?;
	let solana_intent_executor =
		SolanaIntentExecutor::new(solana_url).map_err(|e| log::error!("{:?}", e))?;

	let mut parentchain_listener =
		parentchain_listener::create_listener::<EthereumIntentExecutor, SolanaIntentExecutor>(
			"litentry_rococo",
			Handle::current(),
			&parentchain_url,
			ethereum_intent_executor,
			solana_intent_executor,
			sub_stop_receiver,
			storage_db,
		)
		.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(start_block))
		.unwrap())
}
