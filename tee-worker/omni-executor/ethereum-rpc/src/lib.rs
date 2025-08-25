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
pub mod error;
pub mod signer;

use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::hex;
use alloy::network::Ethereum;
use alloy::network::EthereumWallet;
use alloy::network::NetworkWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::state::AccountOverride;
use alloy::rpc::types::TransactionRequest;
use alloy::transports::RpcError;
use async_trait::async_trait;
use executor_core::wallet_metrics::WalletBalanceFetcher;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::log::error;

pub use error::RpcProviderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eip1559FeeEstimate {
	pub max_fee_per_gas: u128,
	pub max_priority_fee_per_gas: u128,
}

impl Eip1559FeeEstimate {
	pub fn new(max_fee_per_gas: u128, max_priority_fee_per_gas: u128) -> Self {
		Self { max_fee_per_gas, max_priority_fee_per_gas }
	}

	pub fn base_fee(&self) -> u128 {
		self.max_fee_per_gas.saturating_sub(self.max_priority_fee_per_gas)
	}
}

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

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, RpcProviderError>;
	async fn get_pending_nonce(&self, address: Self::Addr) -> Result<u64, ()>;
	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, RpcProviderError>;
	async fn send_transaction(&self, tx: Self::Transaction) -> Result<String, RpcProviderError>;
	async fn send_transaction_with_wallet(
		&self,
		wallet: &EthereumWallet,
		tx: Self::Transaction,
	) -> Result<String, RpcProviderError>;
	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, RpcProviderError>;
	async fn get_gas_price(&self) -> Result<u128, RpcProviderError>;
	async fn estimate_eip1559_fees(&self) -> Result<Eip1559FeeEstimate, RpcProviderError>;

	async fn get_code_at(&self, address: Self::Addr) -> Result<Vec<u8>, RpcProviderError>;

	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, RpcProviderError>;
	async fn call_with_state_override(
		&self,
		tx: Self::Transaction,
		state_override: HashMap<Address, AccountOverride>,
	) -> Result<Vec<u8>, RpcProviderError>;
	async fn get_wallet_address(&self) -> Result<Address, RpcProviderError>;
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

	async fn get_balance(&self, address: Self::Addr) -> Result<U256, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);
		provider.get_balance(address).await.map_err(RpcProviderError::from_alloy_error)
	}

	async fn get_pending_nonce(&self, address: Self::Addr) -> Result<u64, ()> {
		let provider = ProviderBuilder::new().connect_http(
			self.url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?,
		);

		provider
			.get_transaction_count(address)
			.block_id(BlockId::Number(BlockNumberOrTag::Pending))
			.await
			.map_err(|e| error!("Could not get pending nonce: {:?}", e))
	}

	async fn get_transaction_count(&self, address: Self::Addr) -> Result<u64, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);
		provider
			.get_transaction_count(address)
			.await
			.map_err(RpcProviderError::from_alloy_error)
	}

	async fn send_transaction(
		&self,
		raw_tx: Self::Transaction,
	) -> Result<String, RpcProviderError> {
		let Some(ref wallet) = self.wallet else {
			return Err(RpcProviderError::NoWallet);
		};

		self.send_transaction_with_wallet(wallet, raw_tx).await
	}

	async fn send_transaction_with_wallet(
		&self,
		wallet: &EthereumWallet,
		raw_tx: Self::Transaction,
	) -> Result<String, RpcProviderError> {
		let provider = ProviderBuilder::new().wallet(wallet.clone()).connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		let signer_address = wallet.default_signer().address();
		let mut tx = raw_tx.from(signer_address);
		// Add a 10% buffer to gas to make it safer
		tx.gas = Some(self.estimate_gas(tx.clone()).await? * 110 / 100);

		let pending_tx = provider
			.send_transaction(tx)
			.await
			.map_err(RpcProviderError::from_alloy_error)?;

		// Get transaction hash and return immediately without waiting for confirmation
		let tx_hash = pending_tx.tx_hash().to_string();

		Ok(tx_hash)
	}

	async fn estimate_gas(&self, tx: Self::Transaction) -> Result<u64, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		provider
			.estimate_gas(tx.clone())
			.await
			.map_err(RpcProviderError::from_alloy_error)
	}

	async fn get_gas_price(&self) -> Result<u128, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		provider.get_gas_price().await.map_err(RpcProviderError::from_alloy_error)
	}

	async fn estimate_eip1559_fees(&self) -> Result<Eip1559FeeEstimate, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		let estimation = provider
			.estimate_eip1559_fees()
			.await
			.map_err(RpcProviderError::from_alloy_error)?;

		Ok(Eip1559FeeEstimate {
			max_fee_per_gas: estimation.max_fee_per_gas,
			max_priority_fee_per_gas: estimation.max_priority_fee_per_gas,
		})
	}

	async fn get_code_at(&self, address: Self::Addr) -> Result<Vec<u8>, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		provider
			.get_code_at(address)
			.await
			.map_err(RpcProviderError::from_alloy_error)
			.map(|b| b.to_vec())
	}

	async fn call(&self, tx: Self::Transaction) -> Result<Vec<u8>, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);
		let result = provider.call(tx).await.map_err(|e| {
			// Special handling for contract reverts
			if let RpcError::ErrorResp(resp) = &e {
				if let Some(data) = &resp.data {
					// Try to extract revert data
					if let Ok(value) = serde_json::from_str::<String>(data.get()) {
						if value.starts_with("0x") {
							let decoded = match hex::decode(value) {
								Ok(bytes) => Some(bytes),
								Err(e) => {
									error!("Could not decode rpc response: {:?}", e);
									None
								},
							};
							// This is likely revert data, return as execution reverted
							return RpcProviderError::ExecutionReverted {
								reason: resp.message.to_string(),
								data: decoded,
							};
						}
					}
				}
			}
			RpcProviderError::from_alloy_error(e)
		})?;
		Ok(result.to_vec())
	}

	async fn call_with_state_override(
		&self,
		tx: Self::Transaction,
		state_override: HashMap<Address, AccountOverride>,
	) -> Result<Vec<u8>, RpcProviderError> {
		let provider = ProviderBuilder::new().connect_http(
			self.url
				.parse()
				.map_err(|e: url::ParseError| RpcProviderError::InvalidUrl(e.to_string()))?,
		);

		// Use the raw_request method to make eth_call with state override
		let params = serde_json::json!([tx, "latest", state_override]);

		let result =
			provider
				.raw_request::<_, String>("eth_call".into(), params)
				.await
				.map_err(|e| {
					// Special handling for contract reverts with state override
					if let RpcError::ErrorResp(resp) = &e {
						if let Some(data) = &resp.data {
							// Try to extract revert data
							if let Ok(value) = serde_json::from_str::<String>(data.get()) {
								if value.starts_with("0x") {
									let decoded = match hex::decode(value) {
										Ok(bytes) => Some(bytes),
										Err(e) => {
											error!("Could not decode rpc response: {:?}", e);
											None
										},
									};
									// This is likely revert data, return as execution reverted
									return RpcProviderError::ExecutionReverted {
										reason: resp.message.to_string(),
										data: decoded,
									};
								}
							}
						}
					}
					RpcProviderError::from_alloy_error(e)
				})?;

		let decoded = hex::decode(&result[2..]).map_err(|e| {
			RpcProviderError::Generic(format!("Failed to decode hex response: {}", e))
		})?;

		Ok(decoded)
	}

	async fn get_wallet_address(&self) -> Result<Address, RpcProviderError> {
		if let Some(ref wallet) = self.wallet {
			Ok(<EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(wallet))
		} else {
			Err(RpcProviderError::NoWallet)
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
	use crate::Eip1559FeeEstimate;
	use crate::RpcProvider as RpcProviderTrait;
	use crate::RpcProviderError;
	use crate::RpcProviderFactory;
	use alloy::network::EthereumWallet;
	use alloy::primitives::Address;
	use alloy::primitives::U256;
	use alloy::rpc::types::state::AccountOverride;
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

			async fn get_balance(&self, address: Address) -> Result<U256, RpcProviderError>;
			async fn get_transaction_count(&self, address: Address) -> Result<u64, RpcProviderError>;
			async fn send_transaction(&self, tx: TransactionRequest) -> Result<String, RpcProviderError>;
			async fn send_transaction_with_wallet(&self, wallet: &EthereumWallet, tx: TransactionRequest) -> Result<String, RpcProviderError>;
			async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, RpcProviderError>;
			async fn get_gas_price(&self) -> Result<u128, RpcProviderError>;
			async fn estimate_eip1559_fees(&self) -> Result<Eip1559FeeEstimate, RpcProviderError>;
			async fn get_code_at(&self, address: Address) -> Result<Vec<u8>, RpcProviderError>;
			async fn call(&self, tx: TransactionRequest) -> Result<Vec<u8>, RpcProviderError>;
			async fn call_with_state_override(&self, tx: TransactionRequest, state_override: HashMap<Address, AccountOverride>) -> Result<Vec<u8>, RpcProviderError>;
			async fn get_wallet_address(&self) -> Result<Address, RpcProviderError>;
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
