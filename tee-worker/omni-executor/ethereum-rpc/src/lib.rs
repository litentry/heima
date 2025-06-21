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

pub mod client;
pub mod signer;

use std::str::FromStr;

use alloy::network::Ethereum;
use alloy::network::EthereumWallet;
use alloy::network::NetworkWallet;
use alloy::primitives::{Address};
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use executor_core::wallet_metrics::WalletBalanceFetcher;
use tracing::log::error;
use alloy::eips::{BlockId, BlockNumberOrTag};

pub trait RpcProviderFactory {
	type Provider;
	type Context;

	fn create(&self, ctx: Self::Context) -> Self::Provider;
}

pub struct AlloyRpcProviderFactory {
	pub url: String,
}

impl RpcProviderFactory for AlloyRpcProviderFactory {
	type Provider = AlloyRpcProvider;
	type Context = EthereumWallet;

	fn create(&self, ctx: Self::Context) -> Self::Provider {
		AlloyRpcProvider::new_with_wallet(&self.url, ctx)
	}
}

#[async_trait]
pub trait RpcProvider: Send + Sync {
	type Addr;
	type Transaction;

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, ()>;
	async fn get_pending_nonce(&self, address: Self::Addr) -> Result<u64, ()>;
	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, ()>;
	async fn send_transaction(&self, tx: Self::Transaction) -> Result<(), ()>;
	async fn send_transaction_with_wallet(
		&self,
		wallet: &EthereumWallet,
		tx: Self::Transaction,
	) -> Result<String, ()>;
	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, ()>;
	async fn get_gas_price(&self) -> Result<u128, ()>;
	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, ()>;
	async fn get_wallet_address(&self) -> Result<Address, ()>;
}

pub struct AlloyRpcProvider {
	url: String,
	wallet: Option<EthereumWallet>,
}

impl AlloyRpcProvider {
	pub fn new_with_wallet(url: &str, wallet: EthereumWallet) -> Self {
		Self { url: url.to_string(), wallet: Some(wallet) }
	}

	pub fn new(url: &str) -> Self {
		Self { url: url.to_string(), wallet: None }
	}
}

#[async_trait]
impl RpcProvider for AlloyRpcProvider {
	type Addr = Address;
	type Transaction = TransactionRequest;

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);
		provider
			.get_balance(address)
			.await
			.map_err(|e| error!("Could not get balance: {:?}", e))
	}

	async fn get_pending_nonce(&self, address: Self::Addr) -> Result<u64, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);
		let transaction_count = self.get_transaction_count(address).await.map_err(|e| error!("Could not get transaction count: {:?}", e))?;

		let pending_transaction_count = provider
			.get_transaction_count(address)
			.block_id(BlockId::Number(BlockNumberOrTag::Pending))
			.await
			.map_err(|e| error!("Could not get balance: {:?}", e))?;

		Ok(transaction_count + pending_transaction_count)
	}

	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);
		provider
			.get_transaction_count(address)
			.await
			.map_err(|e| error!("Could not get transaction count: {:?}", e))
	}

	async fn send_transaction(&self, raw_tx: Self::Transaction) -> Result<(), ()> {
		let Some(ref wallet) = self.wallet else {
			error!("Provider without a wallet cannot send transactions");
			return Err(());
		};

		self.send_transaction_with_wallet(wallet, raw_tx).await?;

		Ok(())
	}

	async fn send_transaction_with_wallet(
		&self,
		wallet: &EthereumWallet,
		raw_tx: Self::Transaction,
	) -> Result<String, ()> {
		let provider = ProviderBuilder::new().wallet(wallet.clone()).connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);

		let signer_address = wallet.default_signer().address();
		let mut tx = raw_tx.from(signer_address);
		// Bump the gas by 10%
		// TODO: Set gas fees via CLI
		tx.gas = Some(self.estimate_gas(tx.clone()).await? * 110 / 100);
		tx.gas_price = Some(self.get_gas_price().await?);

		let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
			error!("Could not send transaction: {:?}", e);
		})?;

		// Get transaction hash before waiting for receipt
		let tx_hash = pending_tx.tx_hash().to_string();

		// wait for transaction to be included
		let _ = pending_tx.get_receipt().await.map_err(|e| {
			error!("Could not get transaction receipt: {:?}", e);
		})?;

		Ok(tx_hash)
	}

	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);

		provider
			.estimate_gas(tx.clone())
			.await
			.map_err(|e| error!("Could not estimate gas: {:?}", e))
	}

	async fn get_gas_price(&self) -> Result<u128, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);

		provider
			.get_gas_price()
			.await
			.map_err(|e| error!("Could not get gas price: {:?}", e))
	}

	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);

		let result = provider.call(tx).await.map_err(|e| error!("Could not call: {:?}", e))?;

		Ok(result.to_vec())
	}

	async fn get_wallet_address(&self) -> Result<Address, ()> {
		if let Some(ref wallet) = self.wallet {
			Ok(<EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(wallet))
		} else {
			Err(())
		}
	}
}

#[async_trait]
impl WalletBalanceFetcher for AlloyRpcProvider {
	async fn fetch(&self, address: &str) -> Result<f64, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);
		let address =
			Address::from_str(address).map_err(|e| error!("Could not parse address: {:?}", e))?;
		provider
			.get_balance(address)
			.await
			.map_err(|e| error!("Could not fetch wallet balance: {:?}", e))
			.map(|b| b.to::<u64>() as f64)
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {
	use crate::RpcProvider as RpcProviderTrait;
	use crate::RpcProviderFactory;
	use alloy::network::EthereumWallet;
	use alloy::primitives::Address;
	use alloy::primitives::U256;
	use alloy::rpc::types::TransactionRequest;
	use async_trait::async_trait;
	use mockall::mock;
	use std::cell::RefCell;
	use std::collections::HashMap;

	mock! {
		pub RpcProvider {}


		#[async_trait]
		impl RpcProviderTrait for RpcProvider {
			type Addr = Address;
			type Transaction = TransactionRequest;

			async fn get_balance(&self, address: Address) -> Result<U256, ()>;
			async fn get_transaction_count(&self, address: Address) -> Result<u64, ()>;
			async fn send_transaction(&self, tx: TransactionRequest) -> Result<(), ()>;
			async fn send_transaction_with_wallet(&self, wallet: &EthereumWallet, tx: TransactionRequest) -> Result<String, ()>;
			async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, ()>;
			async fn get_gas_price(&self) -> Result<u128, ()>;
			async fn call(&self, tx: TransactionRequest) -> Result<Vec<u8>, ()>;
			async fn get_wallet_address(&self) -> Result<Address, ()>;
		}



	}

	pub struct MockedRpcProviderFactory {
		providers: RefCell<HashMap<Address, MockRpcProvider>>,
	}

	impl MockedRpcProviderFactory {
		pub fn new(providers: HashMap<Address, MockRpcProvider>) -> Self {
			Self { providers: RefCell::new(providers) }
		}
	}

	impl RpcProviderFactory for MockedRpcProviderFactory {
		type Provider = MockRpcProvider;
		type Context = EthereumWallet;

		fn create(&self, wallet: Self::Context) -> Self::Provider {
			self.providers.borrow_mut().remove(&wallet.default_signer().address()).unwrap()
		}
	}
}
