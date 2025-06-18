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

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolCall;
use ethereum_rpc::RpcProvider;
use solana::SolanaClient;
use solana_account_decoder_client_types::UiAccountData;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use tracing::log::error;

use executor_primitives::{EthereumToken, SolanaToken};

sol!("artifacts/IERC20.sol");

pub type SolanaPubkey = solana_sdk::pubkey::Pubkey;
pub type EthereumAddress = alloy::primitives::Address;

pub async fn query_solana<Client: SolanaClient>(
	client: &Client,
	key: &Pubkey,
	token: &SolanaToken,
) -> Result<u64, ()> {
	match token {
		SolanaToken::Native => client
			.get_balance(key)
			.await
			.map_err(|e| error!("Could not get solana native balance: {:?}", e)),
		SolanaToken::SPL(mint) => {
			let mint: Pubkey = (*mint.as_ref()).into();
			let accounts = client
				.get_token_accounts_by_owner(key, TokenAccountsFilter::Mint(mint))
				.await
				.map_err(|e| error!("Could not get owner token accounts: {:?}", e))?;

			let mut amount = 0;
			for account in accounts {
				match account.account.data {
					UiAccountData::Json(ref parsed) => {
						let acc_amount = parsed.parsed["info"]["tokenAmount"]["amount"]
							.as_str()
							.ok_or(error!("Could not find amount in token account"))?;
						let amm = acc_amount
							.parse::<u64>()
							.map_err(|e| error!("Could not parse token amount: {:?}", e))?;
						amount += amm;
					},
					UiAccountData::LegacyBinary(_) | UiAccountData::Binary(_, _) => {
						let decoded = account
							.account
							.data
							.decode()
							.ok_or(error!("Could not decode binary account"))?;
						let account = spl_token::state::Account::unpack(&decoded)
							.map_err(|e| error!("Could not unpack binary account: {:?}", e))?;

						amount += account.amount
					},
				}
			}

			Ok(amount)
		},
	}
}

pub async fn query_ethereum(
	rpc_url: &str,
	account: Address,
	token: &EthereumToken,
) -> Result<U256, ()> {
	let provider = ethereum_rpc::AlloyRpcProvider::new(rpc_url);

	match token {
		EthereumToken::Native => provider.get_balance(account).await,
		EthereumToken::ERC20(address) => {
			let address = address.as_ref().into();
			let call = IERC20::balanceOfCall { account };
			let tx = TransactionRequest::default().with_to(address).with_input(call.abi_encode());
			provider
				.call(tx)
				.await
				.map_err(|_| ())
				.map(|balance| U256::from_be_slice(&balance))
		},
	}
}

#[cfg(test)]
pub mod tests {
	use hex_literal::hex;
	use solana::SolanaRpcClient;
	use solana_sdk::pubkey::Pubkey;
	use std::str::FromStr;

	use crate::{query_ethereum, query_solana};

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn check_query_ethereum() {
		let rpc_url = "http://127.0.0.1:8545";
		let account =
			alloy::primitives::Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8")
				.unwrap();
		let balance = query_ethereum(
			rpc_url,
			account,
			&crate::EthereumToken::ERC20(
				hex!("5FC8d32690cc91D4c39d9d3abcBD16989F875707").try_into().unwrap(),
			),
		)
		.await
		.unwrap();
		println!("Balance is: {:?}", balance);
	}

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn check_query_solana() {
		// address with sol: GKEXWhTFSt48dSN9tZzY5H3qcLL9HZsiQTk9yX8gjDZW

		// mint: HHSfQJEhWkXQZbxuNzmtCwZGNYwAX2VNezdXuVYJbrgE

		let rpc_url = "<solana rpc endpoint>";
		let client = SolanaRpcClient::new(rpc_url);

		// key with solana on devnet
		// let key = Pubkey::from_str("GKEXWhTFSt48dSN9tZzY5H3qcLL9HZsiQTk9yX8gjDZW").unwrap();

		// key with SPL on devnet
		let key = Pubkey::from_str("J9DZj7dzbBYEVfaaBwQ7kBzGkKVTZssBPiatKyfxE3ZL").unwrap();
		let mint_key = Pubkey::from_str("HHSfQJEhWkXQZbxuNzmtCwZGNYwAX2VNezdXuVYJbrgE").unwrap();
		let balance =
			query_solana(&client, &key, &crate::SolanaToken::SPL(mint_key.to_bytes().into()))
				.await
				.unwrap();

		println!("Balance is {:?}", balance);
	}
}
