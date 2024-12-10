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
use solana_intent_executor::SolanaIntentExecutor;
use std::io::Write;
use std::thread::JoinHandle;
use std::{fs, thread};
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

	listen_to_parentchain(cli.parentchain_url, cli.ethereum_url, cli.solana_url, cli.start_block)
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
		)
		.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(start_block))
		.unwrap())
}
