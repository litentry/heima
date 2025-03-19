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
use crate::rpc::RpcProvider;
use crate::rpc::RpcProviderFactory;
use alloy::eips::eip7702::Authorization;
use alloy::network::{EthereumWallet, NetworkWallet, TransactionBuilder, TransactionBuilder7702};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use log::error;

pub struct DelegationDetailsOrPrefund {
	pub sponsor: PrivateKeySigner,
	pub delegation_contract_address: Option<Address>,
	pub prefund: bool,
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
	delegation_or_prefund_details: Option<DelegationDetailsOrPrefund>,
) -> Result<SubmissionDetails, ()> {
	// if we delegate call we need to use another signer
	let (tx_signer, delegation_contract_address) =
		if let Some(ref delegation_details) = delegation_or_prefund_details {
			if delegation_details.prefund {
				(signer_key.clone(), None)
			} else {
				(delegation_details.sponsor.clone(), delegation_details.delegation_contract_address)
			}
		} else {
			(signer_key.clone(), None)
		};

	let tx_signer_wallet = EthereumWallet::from(tx_signer.clone());

	let provider = rpc_provider_factory.create(tx_signer_wallet);

	// Create an authorization in case we are going to use delegation contract and pay fees
	let authorization_list = if let Some(delegation_contract_address) = delegation_contract_address
	{
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

		vec![authorization.into_signed(signature)]
	} else {
		vec![]
	};

	let (to, input, value) = if delegation_contract_address.is_some() {
		(signer_key.address(), prepare_delegate_call_data(to, call_data), value)
	} else {
		(to, call_data, value)
	};

	let mut tx: TransactionRequest = TransactionRequest::default()
		.with_to(to)
		.with_input(input)
		.with_value(U256::from_be_bytes(value));

	if !authorization_list.is_empty() {
		tx.set_authorization_list(authorization_list);
	}

	// prefund account if needed
	let prefund_amount = if let Some(ref details) = delegation_or_prefund_details {
		if details.prefund {
			let gas_required = provider.estimate_gas(tx.clone()).await? as u128;
			let gas_price = provider.get_gas_price().await?;

			let prefund_amount = gas_required * gas_price;

			let tx_signer_wallet = EthereumWallet::from(details.sponsor.clone());

			let balance = provider.get_balance(signer_key.address()).await?;

			if balance < U256::from(prefund_amount) {
				let provider = rpc_provider_factory.create(tx_signer_wallet);

				let prefund_tx = TransactionRequest::default()
					.with_to(tx_signer.address())
					.with_value(U256::from(prefund_amount));

				provider.send_transaction(prefund_tx).await?;
				Some(prefund_amount)
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	provider.send_transaction(tx).await?;

	Ok(SubmissionDetails { prefund_amount })
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

	use crate::{
		rpc::tests::MockedRpcProviderFactory,
		rpc::{AlloyRpcProviderFactory, MockRpcProvider},
	};

	use super::{submit, DelegationDetailsOrPrefund};

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
		let delegation_or_prefund_details = DelegationDetailsOrPrefund {
			delegation_contract_address: None,
			sponsor: prepare_sponsor_signer(),
			prefund: true,
		};
		let rpc_factory = AlloyRpcProviderFactory { url: url.to_string() };
		submit(&rpc_factory, to, value, call_data, signer, Some(delegation_or_prefund_details))
			.await
			.unwrap();
	}

	#[tokio::test]
	pub async fn expect_account_not_prefunded_if_enough_balance() {
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let delegation_or_prefund_details = DelegationDetailsOrPrefund {
			delegation_contract_address: None,
			sponsor: prepare_sponsor_signer(),
			prefund: true,
		};
		let mut signer_rpc_provider = MockRpcProvider::new();

		signer_rpc_provider
			.expect_estimate_gas()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(10))));
		signer_rpc_provider
			.expect_get_gas_price()
			.times(1)
			.returning(|| Box::pin(futures::future::ready(Ok(10))));

		signer_rpc_provider
			.expect_get_balance()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(U256::from(1000)))));

		signer_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(()))));

		let mut providers = HashMap::new();
		providers.insert(signer.address(), signer_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, Some(delegation_or_prefund_details))
			.await
			.unwrap();
	}

	#[tokio::test]
	pub async fn expect_account_prefunded_if_not_enough_balance() {
		let to = Address::default();
		let value = [0; 32];
		let call_data = vec![];
		let signer = prepare_omni_account_signer();
		let sponsor = prepare_sponsor_signer();
		let delegation_or_prefund_details = DelegationDetailsOrPrefund {
			delegation_contract_address: None,
			sponsor: prepare_sponsor_signer(),
			prefund: true,
		};
		let mut signer_rpc_provider = MockRpcProvider::new();

		signer_rpc_provider
			.expect_estimate_gas()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(10))));
		signer_rpc_provider
			.expect_get_gas_price()
			.times(1)
			.returning(|| Box::pin(futures::future::ready(Ok(10))));

		signer_rpc_provider
			.expect_get_balance()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(U256::from(0)))));

		signer_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(()))));

		let mut sponsor_rpc_provider = MockRpcProvider::new();

		sponsor_rpc_provider
			.expect_send_transaction()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(()))));

		let mut providers = HashMap::new();
		providers.insert(signer.address(), signer_rpc_provider);
		providers.insert(sponsor.address(), sponsor_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, Some(delegation_or_prefund_details))
			.await
			.unwrap();
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
		let delegation_or_prefund_details = DelegationDetailsOrPrefund {
			delegation_contract_address: Some(delegation_contract_address.clone()),
			sponsor: prepare_sponsor_signer(),
			prefund: false,
		};

		let mut sponsor_rpc_provider = MockRpcProvider::new();

		sponsor_rpc_provider
			.expect_get_transaction_count()
			.times(1)
			.returning(|_| Box::pin(futures::future::ready(Ok(10))));

		sponsor_rpc_provider
			.expect_send_transaction()
			.times(1)
			.with(predicate::function(move |t: &TransactionRequest| {
				matches!(t.authorization_list, Some(ref authorization_list) if authorization_list.len() == 1 
				&& authorization_list.get(0).unwrap().nonce == 10 && authorization_list.get(0).unwrap().address == delegation_contract_address)
			} ))
			.returning(|_| Box::pin(futures::future::ready(Ok(()))));

		let mut providers = HashMap::new();
		providers.insert(sponsor.address(), sponsor_rpc_provider);

		let rpc_factory = MockedRpcProviderFactory::new(providers);

		submit(&rpc_factory, to, value, call_data, signer, Some(delegation_or_prefund_details))
			.await
			.unwrap();
	}
}
