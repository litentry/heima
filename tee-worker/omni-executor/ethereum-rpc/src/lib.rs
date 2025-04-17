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

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use log::error;

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
pub trait RpcProvider {
	type Addr;
	type Transaction;

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, ()>;
	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, ()>;
	async fn send_transaction(&self, tx: Self::Transaction) -> Result<(), ()>;
	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, ()>;
	async fn get_gas_price(&self) -> Result<u128, ()>;
	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, ()>;
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
		let provider = ProviderBuilder::new()
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);
		provider
			.get_balance(address)
			.await
			.map_err(|e| error!("Could not get balance: {:?}", e))
	}

	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, ()> {
		let provider = ProviderBuilder::new()
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);
		provider
			.get_transaction_count(address)
			.await
			.map_err(|e| error!("Could not get transaction count: {:?}", e))
	}

	async fn send_transaction(&self, tx: Self::Transaction) -> Result<(), ()> {
		if self.wallet.is_none() {
			return Err(());
		}

		let provider = ProviderBuilder::new()
			.wallet(
				self.wallet
					.clone()
					.ok_or(error!("Provider without a wallet cannot send transactions"))?,
			)
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

		let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
			error!("Could not send transaction: {:?}", e);
		})?;
		println!("tx hash is: {}", pending_tx.tx_hash().to_string());
		// wait for transaction to be included
		let _ = pending_tx.get_receipt().await.map_err(|e| {
			error!("Could not get transaction receipt: {:?}", e);
		})?;
		Ok(())
	}

	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, ()> {
		let provider = ProviderBuilder::new()
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

		provider
			.estimate_gas(tx.clone())
			.await
			.map_err(|e| error!("Could not estimate gas: {:?}", e))
	}

	async fn get_gas_price(&self) -> Result<u128, ()> {
		let provider = ProviderBuilder::new()
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

		provider
			.get_gas_price()
			.await
			.map_err(|e| error!("Could not get gas price: {:?}", e))
	}

	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, ()> {
		let provider = ProviderBuilder::new()
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

		let result = provider.call(tx).await.map_err(|e| error!("Could not call: {:?}", e))?;

		Ok(result.to_vec())
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
			async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, ()>;
			async fn get_gas_price(&self) -> Result<u128, ()>;
			async fn call(&self, tx: TransactionRequest) -> Result<Vec<u8>, ()>;
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

#[cfg(test)]
pub mod tests {
	use crate::AlloyRpcProvider;
	use crate::RpcProvider;
	use alloy::network::{EthereumWallet, TransactionBuilder};
	use alloy::primitives::{Address, U256};
	use alloy::rpc::types::TransactionRequest;
	use alloy::signers::local::PrivateKeySigner;
	use std::str::FromStr;

	#[tokio::test]
	async fn test_tx() {
		let private_key = "0b7ff5a0daaaefa75487e1495bff5729aeace7c3ce8d97c0d444a62842de86af";
		// https://hoodi-faucet.pk910.de
		// address 0x3c174bdca218d8fde527baa822672128975a8eae
		let signer = PrivateKeySigner::from_str(private_key).expect("Invalid private key");
		let wallet = EthereumWallet::from(signer);

		let to_address = Address::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
		let amount = U256::from(1_000_000_000_000_000u64); // 0.001 ETH = 10^15 Wei

		let provider = AlloyRpcProvider::new_with_wallet("https://rpc.hoodi.ethpandaops.io", wallet);
		let tx: TransactionRequest = TransactionRequest::default().with_to(to_address).with_value(amount);

		provider.send_transaction(tx).await.expect("Failed to send transaction");
	}
}
