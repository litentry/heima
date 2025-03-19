use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use log::error;
#[cfg(test)]
use mockall::automock;

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
		AlloyRpcProvider::new(&self.url, ctx)
	}
}

#[async_trait]
#[cfg_attr(test, automock(type Addr=Address; type Transaction=TransactionRequest;))]
pub trait RpcProvider {
	type Addr;
	type Transaction;

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, ()>;
	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, ()>;
	async fn send_transaction(&self, tx: Self::Transaction) -> Result<(), ()>;
	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, ()>;
	async fn get_gas_price(&self) -> Result<u128, ()>;
}

pub struct AlloyRpcProvider {
	url: String,
	wallet: EthereumWallet,
}

impl AlloyRpcProvider {
	pub fn new(url: &str, wallet: EthereumWallet) -> Self {
		Self { url: url.to_string(), wallet }
	}
}

//todo: remove unwraps
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
		let provider = ProviderBuilder::new()
			.wallet(self.wallet.clone())
			.on_http(self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

		let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
			error!("Could not send transaction: {:?}", e);
		})?;
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
}

#[cfg(test)]
pub mod tests {
	use crate::rpc::MockRpcProvider;
	use crate::rpc::RpcProviderFactory;
	use alloy::network::EthereumWallet;
	use alloy::primitives::Address;
	use std::cell::RefCell;
	use std::collections::HashMap;

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
