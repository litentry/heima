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

use metrics::{describe_gauge, gauge};
use std::{collections::HashMap, sync::Arc, thread, time::Duration};
use tokio::{runtime::Handle, task::JoinHandle};
use tracing::error;

pub type WalletName = String;
pub type WalletAddress = String;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct WalletId {
	pub address: WalletAddress,
	pub network_type: WalletNetworkType,
}

pub struct Wallet {
	pub id: WalletId,
	pub name: WalletName,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum WalletNetworkType {
	Ethereum(u32),
	Solana,
}

#[async_trait::async_trait]
pub trait WalletBalanceFetcher: Send + Sync {
	async fn fetch(&self, address: &str) -> Result<f64, ()>;
}

struct WalletMetric {
	name: WalletName,
	wallet_id: WalletId,
}

impl WalletMetric {
	pub fn new(name: WalletName, wallet_id: WalletId) -> Self {
		describe_gauge!(wallet_metrics_name(&name), "Wallet balance");
		Self { name, wallet_id }
	}

	pub fn update(&self, balance: f64) {
		gauge!(wallet_metrics_name(&self.name)).set(balance);
	}
}

pub struct WalletMetrics {
	metrics: Vec<WalletMetric>,
	balance_fetchers: HashMap<WalletNetworkType, Arc<Box<dyn WalletBalanceFetcher>>>,
}

impl WalletMetrics {
	pub fn new(
		balance_fetchers: HashMap<WalletNetworkType, Arc<Box<dyn WalletBalanceFetcher>>>,
	) -> Self {
		Self { metrics: Vec::new(), balance_fetchers }
	}

	pub fn register(&mut self, wallet: Wallet) {
		let metric = WalletMetric::new(wallet.name, wallet.id);
		self.metrics.push(metric);
	}

	async fn update(&mut self) {
		for metric in self.metrics.iter() {
			if let Some(fetcher) = self.balance_fetchers.get(&metric.wallet_id.network_type) {
				match fetcher.fetch(&metric.wallet_id.address).await {
					Ok(balance) => metric.update(balance),
					Err(e) => error!(
						"Error while fetching balance for: {:?}, reason: {:?}",
						metric.wallet_id, e
					),
				};
			} else {
				error!("There is not fetcher for {:?}", metric.wallet_id);
			}
		}
	}
}

pub fn start_wallet_metrics(handle: Handle, mut metrics: WalletMetrics) -> JoinHandle<()> {
	handle.clone().spawn_blocking(move || loop {
		handle.block_on(metrics.update());
		// update once per hour
		thread::sleep(Duration::from_secs(60 * 60));
	})
}

fn wallet_metrics_name(wallet_name: &str) -> String {
	format!("{}_balance", wallet_name)
}
