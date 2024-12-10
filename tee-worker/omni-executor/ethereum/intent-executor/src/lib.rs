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

use std::str::FromStr;

use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use async_trait::async_trait;
use executor_core::intent_executor::IntentExecutor;
use executor_core::primitives::Intent;
use log::{error, info};

/// Executes intents on Ethereum network.
pub struct EthereumIntentExecutor {
	rpc_url: String,
}

impl EthereumIntentExecutor {
	pub fn new(rpc_url: &str) -> Result<Self, ()> {
		Ok(Self { rpc_url: rpc_url.to_string() })
	}
}

#[async_trait]
impl IntentExecutor for EthereumIntentExecutor {
	async fn execute(&self, intent: Intent) -> Result<(), ()> {
		info!("Executing intent: {:?}", intent);
		// todo: this should be retrieved from key_store
		let signer = PrivateKeySigner::from_str(
			"0x59c6995e998f97a5a0044964f0945389dc9e86dae86c7a8412f4603b6b78690d",
		)
		.unwrap();
		let wallet = EthereumWallet::from(signer);
		let provider = ProviderBuilder::new()
			.with_recommended_fillers()
			.wallet(wallet)
			.on_http(self.rpc_url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);
		let nonce = provider
			.get_transaction_count(provider.signer_addresses().next().unwrap())
			.await
			.unwrap();
		let gas_price = provider.get_gas_price().await.unwrap();

		match intent {
			Intent::TransferEthereum(to, value) => {
				let mut tx = TransactionRequest::default()
					.to(Address::from(to))
					.nonce(nonce)
					.value(U256::from_be_bytes(value));

				tx.set_gas_price(gas_price);
				let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
					error!("Could not send transaction: {:?}", e);
				})?;
				// wait for transaction to be included
				pending_tx.get_receipt().await.map_err(|e| {
					error!("Could not get transaction receipt: {:?}", e);
				})?;
			},
			Intent::CallEthereum(address, input) => {
				let mut tx = TransactionRequest::default()
					.to(Address::from(address))
					.nonce(nonce)
					.input(TransactionInput::from(input));

				tx.set_gas_price(gas_price);
				let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
					error!("Could not send transaction: {:?}", e);
				})?;
				// wait for transaction to be included
				pending_tx.get_receipt().await.map_err(|e| {
					error!("Could not get transaction receipt: {:?}", e);
				})?;
			},
			_ => {
				error!("[EthereumIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}
		Ok(())
	}
}

#[cfg(test)]
pub mod test {
	use alloy::hex;
	use alloy::hex::FromHex;
	use alloy::network::{EthereumWallet, NetworkWallet, TransactionBuilder};
	use alloy::primitives::Address;
	use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
	use alloy::rpc::types::{TransactionInput, TransactionRequest};
	use alloy::signers::local::PrivateKeySigner;
	use log::error;
	use std::str::FromStr;

	// #[tokio::test]
	pub async fn test() {
		// place url here:
		let url = "";

		let signer = PrivateKeySigner::from_str(
			"0x59c6995e998f97a5a0044964f0945389dc9e86dae86c7a8412f4603b6b78690d",
		)
		.unwrap();
		let wallet = EthereumWallet::from(signer);

		let provider = ProviderBuilder::new()
			.with_recommended_fillers()
			.wallet(wallet)
			.on_http(url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e)).unwrap());
		let nonce = provider
			.get_transaction_count(provider.signer_addresses().next().unwrap())
			.await
			.unwrap();
		let gas_price = provider.get_gas_price().await.unwrap();

		let mut tx = TransactionRequest::default()
			.to(Address::from_hex("0x1f754692f0b0578d6af97faed6319542c9ffd468").unwrap())
			.nonce(nonce)
			.input(TransactionInput::from(hex::decode("0x2166e8280000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001e4c6974656e747279204f6d6e694163636f756e7420657865637574696f6e0000").unwrap()));

		tx.set_gas_price(gas_price);

		let pending_tx = provider
			.send_transaction(tx)
			.await
			.map_err(|e| {
				std::println!("Could not send transaction: {:?}", e);
			})
			.map_err(|e| {
				std::println!("Could not get transaction receipt: {:?}", e);
				()
			})
			.unwrap();

		pending_tx
			.get_receipt()
			.await
			.map_err(|e| {
				error!("Could not get transaction receipt: {:?}", e);
			})
			.unwrap();
	}
}
