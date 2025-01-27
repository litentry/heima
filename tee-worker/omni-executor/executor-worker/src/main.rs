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
use executor_storage::init_storage;
use log::error;
use native_task_handler::run_native_task_handler;
use rpc_server::{start_server as start_rpc_server, ShieldingKey};
use solana_intent_executor::SolanaIntentExecutor;
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
			init_storage(&args.parentchain_url).await.expect("Could not initialize storage");

			// TODO: make buffer size configurable
			let buffer = 1024;
			let native_task_sender = run_native_task_handler(buffer).await;
			// TODO: get mrenclave from quote
			let mrenclave = [0u8; 32];

			start_rpc_server(
				&args.worker_rpc_port,
				ShieldingKey::new(),
				Arc::new(native_task_sender),
				mrenclave,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

			listen_to_parentchain(args).await.unwrap();

			match signal::ctrl_c().await {
				Ok(()) => {},
				Err(err) => {
					eprintln!("Unable to listen for shutdown signal: {}", err);
					// we also shut down in case of error
				},
			}
		},
		Commands::GenKey(args) => {
			let _ = parentchain_listener::get_signer(&args.keystore_path);
		},
	}

	Ok(())
}

async fn listen_to_parentchain(args: RunArgs) -> Result<JoinHandle<()>, ()> {
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
			&args.keystore_path,
			&args.log_path,
		)
		.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(args.start_block))
		.unwrap())
}
