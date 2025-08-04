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

pub async fn query_ethereum<
	Provider: RpcProvider<Addr = Address, Transaction = TransactionRequest>,
>(
	provider: &Provider,
	account: Address,
	token: &EthereumToken,
) -> Result<U256, ()> {
	match token {
		EthereumToken::Native => provider.get_balance(account).await.map_err(|_| ()),
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
		let provider = ethereum_rpc::AlloyRpcProvider::new(rpc_url);
		let account =
			alloy::primitives::Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8")
				.unwrap();
		let balance = query_ethereum(
			&provider,
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

	#[cfg(test)]
	mod tests {
		use super::*;
		use alloy::primitives::{Address, U256};
		use executor_primitives::{EthereumToken, SolanaToken};
		use solana_account_decoder_client_types::{ParsedAccount, UiAccount, UiAccountData};
		use solana_client::rpc_request::TokenAccountsFilter;
		use solana_client::rpc_response::RpcKeyedAccount;
		use solana_sdk::pubkey::Pubkey;

		#[tokio::test]
		async fn test_query_solana_native_success() {
			let mut mock_client = solana::mocks::MockSolanaRpcClient::new();
			let pubkey = Pubkey::new_unique();
			let expected_balance = 1000000000u64; // 1 SOL in lamports

			mock_client
				.expect_get_balance()
				.with(mockall::predicate::eq(pubkey))
				.times(1)
				.returning(move |_| Ok(expected_balance));

			let result = query_solana(&mock_client, &pubkey, &SolanaToken::Native).await;

			assert_eq!(result, Ok(expected_balance));
		}

		#[tokio::test]
		async fn test_query_solana_native_error() {
			let mut mock_client = solana::mocks::MockSolanaRpcClient::new();
			let pubkey = Pubkey::new_unique();

			mock_client
				.expect_get_balance()
				.with(mockall::predicate::eq(pubkey))
				.times(1)
				.returning(|_| Err(()));

			let result = query_solana(&mock_client, &pubkey, &SolanaToken::Native).await;

			assert_eq!(result, Err(()));
		}

		#[tokio::test]
		async fn test_query_solana_spl_success_json_format() {
			let mut mock_client = solana::mocks::MockSolanaRpcClient::new();
			let pubkey = Pubkey::new_unique();
			let mint_pubkey = Pubkey::new_unique();
			let token = SolanaToken::SPL(mint_pubkey.to_bytes().into());

			// Create JSON response structure
			let parsed_json = serde_json::json!({
				"info": {
					"tokenAmount": {
						"amount": "500"
					}
				}
			});

			let parsed_account =
				ParsedAccount { parsed: parsed_json, program: "spl-token".to_string(), space: 165 };

			let account_data = UiAccountData::Json(parsed_account);

			let rpc_account = RpcKeyedAccount {
				pubkey: Pubkey::new_unique().to_string(),
				account: UiAccount {
					lamports: 2039280,
					data: account_data,
					owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
					executable: false,
					rent_epoch: 361,
					space: Some(165),
				},
			};

			mock_client
				.expect_get_token_accounts_by_owner()
				.with(
					mockall::predicate::eq(pubkey),
					mockall::predicate::function(move |filter: &TokenAccountsFilter| {
						matches!(filter, TokenAccountsFilter::Mint(mint) if *mint == mint_pubkey)
					}),
				)
				.times(1)
				.returning(move |_, _| Ok(vec![rpc_account.clone()]));

			let result = query_solana(&mock_client, &pubkey, &token).await;

			assert_eq!(result, Ok(500));
		}

		#[tokio::test]
		async fn test_query_solana_spl_multiple_accounts() {
			let mut mock_client = solana::mocks::MockSolanaRpcClient::new();
			let pubkey = Pubkey::new_unique();
			let mint_pubkey = Pubkey::new_unique();
			let token = SolanaToken::SPL(mint_pubkey.to_bytes().into());

			// First account with 300 tokens
			let parsed_json1 = serde_json::json!({
				"info": {
					"tokenAmount": {
						"amount": "300"
					}
				}
			});

			let parsed_account1 = ParsedAccount {
				parsed: parsed_json1,
				program: "spl-token".to_string(),
				space: 165,
			};

			let account_data1 = UiAccountData::Json(parsed_account1);

			// Second account with 700 tokens
			let parsed_json2 = serde_json::json!({
				"info": {
					"tokenAmount": {
						"amount": "700"
					}
				}
			});

			let parsed_account2 = ParsedAccount {
				parsed: parsed_json2,
				program: "spl-token".to_string(),
				space: 165,
			};

			let account_data2 = UiAccountData::Json(parsed_account2);

			let accounts = vec![
				RpcKeyedAccount {
					pubkey: Pubkey::new_unique().to_string(),
					account: UiAccount {
						lamports: 2039280,
						data: account_data1,
						owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
						executable: false,
						rent_epoch: 361,
						space: Some(165),
					},
				},
				RpcKeyedAccount {
					pubkey: Pubkey::new_unique().to_string(),
					account: UiAccount {
						lamports: 2039280,
						data: account_data2,
						owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
						executable: false,
						rent_epoch: 361,
						space: Some(165),
					},
				},
			];

			mock_client
				.expect_get_token_accounts_by_owner()
				.with(
					mockall::predicate::eq(pubkey),
					mockall::predicate::function(move |filter: &TokenAccountsFilter| {
						matches!(filter, TokenAccountsFilter::Mint(mint) if *mint == mint_pubkey)
					}),
				)
				.times(1)
				.returning(move |_, _| Ok(accounts.clone()));

			let result = query_solana(&mock_client, &pubkey, &token).await;

			assert_eq!(result, Ok(1000)); // 300 + 700
		}

		#[tokio::test]
		async fn test_query_solana_spl_error() {
			let mut mock_client = solana::mocks::MockSolanaRpcClient::new();
			let pubkey = Pubkey::new_unique();
			let mint_pubkey = Pubkey::new_unique();
			let token = SolanaToken::SPL(mint_pubkey.to_bytes().into());

			mock_client
				.expect_get_token_accounts_by_owner()
				.with(
					mockall::predicate::eq(pubkey),
					mockall::predicate::function(move |filter: &TokenAccountsFilter| {
						matches!(filter, TokenAccountsFilter::Mint(mint) if *mint == mint_pubkey)
					}),
				)
				.times(1)
				.returning(|_, _| Err(()));

			let result = query_solana(&mock_client, &pubkey, &token).await;

			assert_eq!(result, Err(()));
		}

		#[tokio::test]
		async fn test_query_ethereum_native_success() {
			let mut mock_provider = ethereum_rpc::mocks::MockRpcProvider::new();
			let address = Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap();
			let expected_balance = U256::from(1000000000000000000u64); // 1 ETH in wei

			mock_provider
				.expect_get_balance()
				.with(mockall::predicate::eq(address))
				.times(1)
				.returning(move |_| Ok(expected_balance));

			let result = query_ethereum(&mock_provider, address, &EthereumToken::Native).await;

			assert_eq!(result, Ok(expected_balance));
		}

		#[tokio::test]
		async fn test_query_ethereum_native_error() {
			let mut mock_provider = ethereum_rpc::mocks::MockRpcProvider::new();
			let address = Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap();

			mock_provider
				.expect_get_balance()
				.with(mockall::predicate::eq(address))
				.times(1)
				.returning(|_| Err(()));

			let result = query_ethereum(&mock_provider, address, &EthereumToken::Native).await;

			assert_eq!(result, Err(()));
		}

		#[tokio::test]
		async fn test_query_ethereum_erc20_success() {
			let mut mock_provider = ethereum_rpc::mocks::MockRpcProvider::new();
			let account = Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap();
			let token_address = hex!("5FC8d32690cc91D4c39d9d3abcBD16989F875707");
			let token = EthereumToken::ERC20(token_address.try_into().unwrap());
			let expected_balance = U256::from(500000000u64); // 500 tokens (with 6 decimals)

			// Mock the call method to return the balance
			let balance_bytes = expected_balance.to_be_bytes_vec();
			mock_provider
				.expect_call()
				.times(1)
				.returning(move |_| Ok(balance_bytes.clone()));

			let result = query_ethereum(&mock_provider, account, &token).await;

			assert_eq!(result, Ok(expected_balance));
		}

		#[tokio::test]
		async fn test_query_ethereum_erc20_error() {
			let mut mock_provider = ethereum_rpc::mocks::MockRpcProvider::new();
			let account = Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap();
			let token_address = hex!("5FC8d32690cc91D4c39d9d3abcBD16989F875707");
			let token = EthereumToken::ERC20(token_address.try_into().unwrap());

			mock_provider.expect_call().times(1).returning(|_| Err(None));

			let result = query_ethereum(&mock_provider, account, &token).await;

			assert_eq!(result, Err(()));
		}
	}
}
