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
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use log::error;

pub struct DelegationDetails {
	pub pay_master: PrivateKeySigner,
	pub delegation_contract_address: Address,
}

pub async fn submit(
	rpc_url: &str,
	to: Address,
	value: [u8; 32],
	call_data: Vec<u8>,
	signer_key: PrivateKeySigner,
	delegation_details: Option<DelegationDetails>,
) -> Result<(), ()> {
	let (tx_signer, delegation_contract_address) =
		if let Some(delegation_details) = delegation_details {
			(delegation_details.pay_master, Some(delegation_details.delegation_contract_address))
		} else {
			(signer_key.clone(), None)
		};

	let tx_signer_wallet = EthereumWallet::from(tx_signer);

	let provider = ProviderBuilder::new()
		.with_recommended_fillers()
		.wallet(tx_signer_wallet)
		.on_http(rpc_url.parse().map_err(|e| error!("Could not parse rpc url: {:?}", e))?);

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
				.await
				.map_err(|e| error!("Could not get default signer: {:?}", e))?,
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

	let tx = TransactionRequest::default()
		.with_to(to)
		.with_input(input)
		.with_value(U256::from_be_bytes(value))
		.with_authorization_list(authorization_list);

	let pending_tx = provider.send_transaction(tx).await.map_err(|e| {
		error!("Could not send transaction: {:?}", e);
	})?;
	// wait for transaction to be included
	let _ = pending_tx.get_receipt().await.map_err(|e| {
		error!("Could not get transaction receipt: {:?}", e);
	})?;

	Ok(())
}
