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

use crate::delegate_call::prepare_delegate_call_data;
use alloy::eips::eip7702::Authorization;
use alloy::network::{EthereumWallet, NetworkWallet, TransactionBuilder, TransactionBuilder7702};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use ethereum_rpc::RpcProvider;
use ethereum_rpc::RpcProviderFactory;
use tracing::log::error;

#[allow(dead_code)]
pub enum Paymode {
	Standard,
	Delegated(Address, PrivateKeySigner),
	Prefunded(PrivateKeySigner),
}

#[allow(dead_code)]
pub struct SubmissionDetails {
	prefund_amount: Option<u128>,
}

pub async fn submit<
	RP: RpcProvider<Addr = Address, Transaction = TransactionRequest>,
	RPF: RpcProviderFactory<Context = EthereumWallet, Provider = RP>,
>(
	rpc_provider_factory: &RPF,
	to: Address,
	value: [u8; 32],
	call_data: Vec<u8>,
	signer_key: PrivateKeySigner,
	paymode: Paymode,
) -> Result<SubmissionDetails, ()> {
	match paymode {
		Paymode::Standard => {
			let tx_signer_wallet = EthereumWallet::from(signer_key);
			let provider = rpc_provider_factory.create(tx_signer_wallet);

			let tx: TransactionRequest = TransactionRequest::default()
				.with_to(to)
				.with_input(call_data)
				.with_value(U256::from_be_bytes(value));
			provider.send_transaction(tx).await?;

			Ok(SubmissionDetails { prefund_amount: None })
		},
		Paymode::Delegated(delegation_contract_address, sponsor_signer) => {
			let tx_sponsor_wallet = EthereumWallet::from(sponsor_signer);
			let provider = rpc_provider_factory.create(tx_sponsor_wallet);

			let authorization = Authorization {
				// chain id needs to be parametrized
				chain_id: U256::from(0),
				// Reference to the contract that will be set as code for the authority.
				address: delegation_contract_address,
				nonce: provider
					.get_transaction_count(<EthereumWallet as NetworkWallet<
						alloy::network::Ethereum,
					>>::default_signer_address(&EthereumWallet::from(
						signer_key.clone(),
					)))
					.await?,
			};
			let signature = signer_key
				.sign_hash(&authorization.signature_hash())
				.await
				.map_err(|e| error!("Could not get default signer: {:?}", e))?;

			let authorization_list = vec![authorization.into_signed(signature)];

			let tx: TransactionRequest = TransactionRequest::default()
				.with_to(signer_key.address())
				.with_input(prepare_delegate_call_data(to, call_data))
				.with_value(U256::from_be_bytes(value))
				.with_authorization_list(authorization_list);
			provider.send_transaction(tx).await?;

			Ok(SubmissionDetails { prefund_amount: None })
		},
		Paymode::Prefunded(sponsor_signer) => {
			let signer_address = signer_key.address();
			let tx_signer_wallet = EthereumWallet::from(signer_key);
			let provider = rpc_provider_factory.create(tx_signer_wallet);

			let tx: TransactionRequest = TransactionRequest::default()
				.with_to(to)
				.with_input(call_data)
				.with_value(U256::from_be_bytes(value));

			let gas_required = provider.estimate_gas(tx.clone()).await? as u128;
			let gas_price = provider.get_gas_price().await?;

			let prefund_amount = gas_required * gas_price;

			let balance = provider.get_balance(signer_address).await?;

			let prefund_amount = if balance < U256::from(prefund_amount) {
				let sponsor_wallet = EthereumWallet::from(sponsor_signer);
				let provider = rpc_provider_factory.create(sponsor_wallet);

				let prefund_tx = TransactionRequest::default()
					.with_to(signer_address)
					.with_value(U256::from(prefund_amount));

				provider.send_transaction(prefund_tx).await?;
				Some(prefund_amount)
			} else {
				None
			};

			provider.send_transaction(tx).await?;

			Ok(SubmissionDetails { prefund_amount })
		},
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::HashMap;
	use std::str::FromStr;

	use alloy::hex::FromHex;
	use alloy::primitives::Address;
	use alloy::primitives::U256;
	use alloy::rpc::types::TransactionRequest;
	use alloy::signers::local::PrivateKeySigner;
	use mockall::predicate;

	use crate::tx::Paymode;
	use ethereum_rpc::mocks::{MockRpcProvider, MockedRpcProviderFactory};
	use ethereum_rpc::AlloyRpcProviderFactory;

	use super::submit;

	fn prepare_omni_account_signer() -> PrivateKeySigner {
		PrivateKeySigner::from_str(
			"0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
		)
		.unwrap()
	}

	fn prepare_sponsor_signer() -> PrivateKeySigner {
		PrivateKeySigner::from_str(
			"0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
		)
		.unwrap()
	}

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn check_submit() {
		let url = "http://localhost:8545";
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let paymode = Paymode::Prefunded(prepare_sponsor_signer());
		let rpc_factory = AlloyRpcProviderFactory { url: url.to_string() };
		submit(&rpc_factory, to, value, call_data, signer, paymode).await.unwrap();
	}

	#[tokio::test]
	pub async fn expect_account_not_prefunded_if_enough_balance() {
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let paymode = Paymode::Prefunded(prepare_sponsor_signer());
		let mut signer_rpc_provider = MockRpcProvider::new();

		signer_rpc_provider.expect_estimate_gas().times(1).returning(|_| Ok(10));
		signer_rpc_provider.expect_get_gas_price().times(1).returning(|| Ok(10));

		signer_rpc_provider
			.expect_get_balance()
			.times(1)
			.returning(|_| Ok(U256::from(1000)));

		signer_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let mut providers = HashMap::new();
		providers.insert(signer.address(), signer_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, paymode).await.unwrap();
	}

	#[tokio::test]
	pub async fn expect_account_prefunded_if_not_enough_balance() {
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let sponsor = prepare_sponsor_signer();
		let paymode = Paymode::Prefunded(prepare_sponsor_signer());
		let mut signer_rpc_provider = MockRpcProvider::new();

		signer_rpc_provider.expect_estimate_gas().times(1).returning(|_| Ok(10));
		signer_rpc_provider.expect_get_gas_price().times(1).returning(|| Ok(10));

		signer_rpc_provider
			.expect_get_balance()
			.times(1)
			.returning(|_| Ok(U256::from(0)));

		signer_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let mut sponsor_rpc_provider = MockRpcProvider::new();

		sponsor_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let mut providers = HashMap::new();
		providers.insert(signer.address(), signer_rpc_provider);
		providers.insert(sponsor.address(), sponsor_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, paymode).await.unwrap();
	}

	#[tokio::test]
	pub async fn expect_call_delegated() {
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let sponsor = prepare_sponsor_signer();
		let delegation_contract_address =
			Address::from_hex("0xc07cb79754cf3b252038e2713a138363d55df9e0").unwrap();
		let paymode = Paymode::Delegated(delegation_contract_address, prepare_sponsor_signer());

		let mut sponsor_rpc_provider = MockRpcProvider::new();

		sponsor_rpc_provider
			.expect_get_transaction_count()
			.times(1)
			.returning(|_| Ok(10));

		sponsor_rpc_provider
			.expect_send_transaction()
			.times(1)
			.with(predicate::function(move |t: &TransactionRequest| {
				matches!(t.authorization_list, Some(ref authorization_list) if authorization_list.len() == 1
					&& authorization_list.first().unwrap().nonce == 10
					&& authorization_list.first().unwrap().address == delegation_contract_address)
			}))
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let mut providers = HashMap::new();
		providers.insert(sponsor.address(), sponsor_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, paymode).await.unwrap();
	}
}
