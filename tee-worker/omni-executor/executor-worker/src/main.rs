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

use intention_executor::EthereumIntentionExecutor;
use log::error;
use parentchain_listener::CustomConfig;
use std::io::Write;
use std::thread::JoinHandle;
use std::{fs, thread};
use tokio::runtime::Handle;
use tokio::sync::oneshot;

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

	fs::create_dir_all("data/").map_err(|e| {
		error!("Could not create data dir: {:?}", e);
	})?;

	listen_to_parentchain().await.unwrap().join().unwrap();
	Ok(())
}

async fn listen_to_parentchain() -> Result<JoinHandle<()>, ()> {
	let (_sub_stop_sender, sub_stop_receiver) = oneshot::channel();

	let relayer = EthereumIntentionExecutor::new("http://ethereum-node:8545")
		.map_err(|e| log::error!("{:?}", e))?;

	let mut parentchain_listener = parentchain_listener::create_listener::<CustomConfig>(
		"litentry_rococo",
		Handle::current(),
		"ws://litentry-node:9944",
		Box::new(relayer),
		sub_stop_receiver,
	)
	.await?;

	Ok(thread::Builder::new()
		.name("litentry_rococo_sync".to_string())
		.spawn(move || parentchain_listener.sync(0))
		.unwrap())
}
