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

use crate::Chain;
use crate::CrossChainIntentExecutor;
use crate::RpcEndpointRegistry;
use crate::U256;
use accounting_contract_client::AccountingContractApi;
use alloy::primitives::Address;
use binance_api::spot_trading_api::types::SymbolPrice;
use binance_api::wallet_api::types::{CoinInfo, DepositAddress, NetworkInfo};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use executor_primitives::Identity;
use executor_primitives::Intent;
use executor_primitives::PumpxOrderType;
use executor_primitives::SingleChainSwapProvider;
use executor_primitives::SolanaToken;
use executor_primitives::{EthereumToken, PumpxConfig, SwapOrder};
use executor_storage::PumpxJwtStorage;
use executor_storage::Storage;
use executor_storage::StorageDB;
use heima_authentication::auth_token::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::BoundedVec;
use heima_primitives::IdentityString;
use intent_asset_lock::precise::PreciseAssetsLock;
use intent_asset_lock::AccountAssetLocks;
use intent_asset_lock::AmountType;
use pumpx::methods::common::GasType;
use pumpx::methods::common::OrderInfoResponse;
use pumpx::methods::common::OrderInfoResponseData;
use pumpx::methods::common::SwapType;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxResponse;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxResponseData;
use pumpx::methods::get_gas_info::GasInfo;
use pumpx::methods::get_gas_info::GetGasInfoResponse;
use pumpx::methods::get_gas_info::GetGasInfoResponseData;
use pumpx::methods::send_order_tx::SendOrderTxResponse;
use pumpx::methods::send_order_tx::SendOrderTxResponseData;
use pumpx::signer_client::SignerClient;
use pumpx::PumpxApi;
use reqwest::Method;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn simple_cross_chain_swap() {
	let account_id: AccountId =
		Identity::Pumpx(IdentityString::new("1".as_bytes().to_vec())).to_omni_account();

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = pumpx::signer_client_mocks::MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] =
		hex::decode("7f2202c7e1f34f3ad0647e97c63eb00b0eab7434800ef56d441adeb750a57e1f")
			.unwrap()
			.try_into()
			.unwrap();
	let to_chain_id = 56;
	let order_id = 0;
	let expected_payout_address = "0x8Fc876ca8b23Ef6b735e388Ea94327e189DD5Ea7";
	let binance_deposit_address = "binance_deposit_address";
	let solana_coin_ticker = "SOL";
	let solana_coin_name = "Solana";

	// this is called twice, one for solana address and later for ethereum address
	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Solana),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok(hex::decode("96dd2f4ecf7c9330e4f0e58a8e6272672fefee208857cd772e8aa1327b39dbfa")
				.unwrap()
				.to_vec())
		});

	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(2)
		.returning(|_, _, _| {
			Ok(hex::decode("0365db18229197e1ff835e0faaec9a9a9900b0aeb5f18e3faa5a4ca60c80213d7c")
				.unwrap()
				.to_vec())
		});

	let create_order_encoded_tx: Vec<u8> = hex::decode(
		"e9808504e3b29200831e848094d8da6bf26964af9d7eed9e03e53415d37aa96045843b9aca0080018080",
	)
	.unwrap();
	let create_order_tx_signature: Vec<u8> = hex::decode("f6c76effbee5ac9ff341a0efe85b5f9401ab673c5d1c2746041e446e3d19aea337470aac13f1cb4da20f086a475cceb36a5bb4fd9cf52b48300ad840ad1480ad1b").unwrap();

	pumpx_signer_client_mock
		.expect_request_signatures()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| Ok(vec![create_order_tx_signature.to_vec()]));

	let mut pumpx_api_mock = pumpx::mocks::MockPumpxApiClient::new();
	pumpx_api_mock.expect_create_cross_order().times(1).returning(move |_, _| {
		Ok(OrderInfoResponse {
			code: 0,
			data: OrderInfoResponseData { order_id: Some(order_id) },
			message: "".to_string(),
		})
	});
	pumpx_api_mock
		.expect_get_gas_info()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(to_chain_id))
		.times(1)
		.returning(move |_, _| {
			Ok(GetGasInfoResponse {
				code: 0,
				data: GetGasInfoResponseData {
					gas_info: Some(vec![GasInfo {
						chain_id: to_chain_id.to_string(),
						normal: "1".to_string(),
						fast: "1".to_string(),
						super_fast: "1".to_string(),
						normal_usd: "1".to_string(),
						fast_usd: "1".to_string(),
						super_fast_usd: "1".to_string(),
						normal_price: "1".to_string(),
						fast_price: "1".to_string(),
						super_fast_price: "1".to_string(),
					}]),
				},
				message: "".to_string(),
			})
		});

	pumpx_api_mock.expect_create_market_order_unsigned_tx()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(
			CreateMarketOrderUnsignedTxBody {
				request_id: 0,
				chain_id: to_chain_id,
				token_ca: "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_string(),
				swap_type: SwapType::Buy,
				amount_in: "998.000000000000000000".to_string(),
				double_out: false,
				is_one_click: false,
				address: expected_payout_address.to_string(),
				is_anti_mev: false,
				is_auto_slippage: false,
				gas_type: GasType::Slow,
				slippage: 0,
				wallet_index: 1,
				recipient_address: expected_payout_address.to_string()
			}
		))
		.times(1)
		.returning(move |_, _| {
			Ok(CreateMarketOrderUnsignedTxResponse {
				code: 0,
				data: CreateMarketOrderUnsignedTxResponseData {
					chain_id: Some(to_chain_id),
					order_id: Some(order_id),
					tx_data: Some(
						vec![format!("0x{}", hex::encode(create_order_encoded_tx.clone()))]
					),
				},
				message: "".to_string(),
			})
		});

	pumpx_api_mock
		.expect_send_order_tx()
		.with(
			mockall::predicate::eq("test_token"),
			mockall::predicate::always(/* SendOrderTxBody {
				chain_id: to_chain_id,
				order_id: order_id,
				tx_data: vec!["0xf869808504e3b29200831e848094d8da6bf26964af9d7eed9e03e53415d37aa96045843b9aca008025a0f6c76effbee5ac9ff341a0efe85b5f9401ab673c5d1c2746041e446e3d19aea3a037470aac13f1cb4da20f086a475cceb36a5bb4fd9cf52b48300ad840ad1480ad".to_string()]
			}*/),
		)
		.times(1)
		.returning(|_, _| {
			Ok(SendOrderTxResponse {
				code: 0,
				data: SendOrderTxResponseData { tx_hash: None },
				message: "".to_string(),
			})
		});

	let mut binance_api_mock = binance_api::mocks::MockBinanceApiClient::new();

	binance_api_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(vec![prepare_sol_coin_info(solana_coin_ticker, solana_coin_name)])
		});

	binance_api_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/deposit/address"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::always(),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(DepositAddress {
				address: binance_deposit_address.to_string(),
				coin: solana_coin_ticker.to_string(),
				tag: "tag".to_string(),
				url: "url".to_string(),
			})
		});

	let mut sol_usdt_price_params: HashMap<String, String> = HashMap::new();
	sol_usdt_price_params.insert("symbol".to_string(), "SOLUSDT".to_string());
	binance_api_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_usdt_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut sol_bnb_price_params: HashMap<String, String> = HashMap::new();
	sol_bnb_price_params.insert("symbol".to_string(), "SOLBNB".to_string());
	binance_api_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_bnb_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut accounting_contract_client_mock =
		accounting_contract_client::mocks::MockAccountingContractClient::new();

	accounting_contract_client_mock
		.expect_get_balance()
		.times(1)
		.returning(|| Ok(U256::from_str_radix("1000000000000000000000", 10).unwrap()));

	accounting_contract_client_mock
		.expect_get_nonce()
		.times(1)
		.returning(|_| Ok(U256::from(1)));

	accounting_contract_client_mock
		.expect_execute_pay_out_request()
		.with(
			mockall::predicate::eq(Address::from_str(expected_payout_address).unwrap()),
			mockall::predicate::eq(U256::from(2)),
			mockall::predicate::eq(U256::from_str("999000000000000000000").unwrap()),
		)
		.times(1)
		.returning(|_, _, _| Ok(()));

	let mut solana_client_mock = solana::mocks::MockSolanaRpcClient::new();

	solana_client_mock
		.expect_transfer_sol()
		.withf(move |to: &str, value, _| to == binance_deposit_address && value == &100000000000)
		.times(1)
		.returning(|_, _, _| Ok("".to_string()));

	let rpc_endpoint_registry = RpcEndpointRegistry::new();
	let pumpx_signer_client: Arc<Box<dyn SignerClient>> =
		Arc::new(Box::new(pumpx_signer_client_mock));
	let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(pumpx_api_mock));
	let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
	let binance_api = Arc::new(binance_api_mock);
	let solana_client = Arc::new(solana_client_mock);
	let accounting_contract_client: Arc<Box<dyn AccountingContractApi>> =
		Arc::new(Box::new(accounting_contract_client_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("101".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(to_chain_id, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider =
		SingleChainSwapProvider::Pumpx(prepare_pumpx_config(to_chain_id, pumpx_wallet_index));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock.clone(),
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		binance_api,
		solana_client,
		accounting_contract_client,
		Decimal::from_str("1").unwrap(),
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = PumpxJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
	assert_eq!(
		account_assets_lock.get_locked_amount(
			&account_id,
			intent_asset_lock::AssetId::Solana(SolanaToken::Native)
		),
		Ok(AmountType::from(0))
	);
}

#[tokio::test]
async fn instant_payout_cross_chain_swap() {
	let account_id: AccountId =
		Identity::Pumpx(IdentityString::new("1".as_bytes().to_vec())).to_omni_account();

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = pumpx::signer_client_mocks::MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] =
		hex::decode("7f2202c7e1f34f3ad0647e97c63eb00b0eab7434800ef56d441adeb750a57e1f")
			.unwrap()
			.try_into()
			.unwrap();
	let to_chain_id = 56;
	let order_id = 0;
	let expected_payout_address = "0x8Fc876ca8b23Ef6b735e388Ea94327e189DD5Ea7";
	let binance_deposit_address = "binance_deposit_address";
	let solana_coin_ticker = "SOL";
	let solana_coin_name = "Solana";
	let accounting_contract_client_address = "0x7CE3464A6dc52001754b0b90878d9Ffd61B47c6C";

	// this is called twice, one for solana address and later for ethereum address
	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Solana),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok(hex::decode("96dd2f4ecf7c9330e4f0e58a8e6272672fefee208857cd772e8aa1327b39dbfa")
				.unwrap()
				.to_vec())
		});

	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq([
				127, 34, 2, 199, 225, 243, 79, 58, 208, 100, 126, 151, 198, 62, 176, 11, 14, 171,
				116, 52, 128, 14, 245, 109, 68, 26, 222, 183, 80, 165, 126, 31,
			]),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok(hex::decode("0365db18229197e1ff835e0faaec9a9a9900b0aeb5f18e3faa5a4ca60c80213d7c")
				.unwrap()
				.to_vec())
		});

	let create_order_encoded_tx: Vec<u8> = hex::decode(
		"e9808504e3b29200831e848094d8da6bf26964af9d7eed9e03e53415d37aa96045843b9aca0080018080",
	)
	.unwrap();
	let create_order_tx_signature: Vec<u8> = hex::decode("f6c76effbee5ac9ff341a0efe85b5f9401ab673c5d1c2746041e446e3d19aea337470aac13f1cb4da20f086a475cceb36a5bb4fd9cf52b48300ad840ad1480ad1b").unwrap();

	pumpx_signer_client_mock
		.expect_request_signatures()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| Ok(vec![create_order_tx_signature.to_vec()]));

	let mut pumpx_api_mock = pumpx::mocks::MockPumpxApiClient::new();
	pumpx_api_mock.expect_create_cross_order().times(1).returning(move |_, _| {
		Ok(OrderInfoResponse {
			code: 0,
			data: OrderInfoResponseData { order_id: Some(order_id) },
			message: "".to_string(),
		})
	});
	pumpx_api_mock
		.expect_get_gas_info()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(to_chain_id))
		.times(1)
		.returning(move |_, _| {
			Ok(GetGasInfoResponse {
				code: 0,
				data: GetGasInfoResponseData {
					gas_info: Some(vec![GasInfo {
						chain_id: to_chain_id.to_string(),
						normal: "1".to_string(),
						fast: "1".to_string(),
						super_fast: "1".to_string(),
						normal_usd: "1".to_string(),
						fast_usd: "1".to_string(),
						super_fast_usd: "1".to_string(),
						normal_price: "1".to_string(),
						fast_price: "1".to_string(),
						super_fast_price: "1".to_string(),
					}]),
				},
				message: "".to_string(),
			})
		});

	pumpx_api_mock.expect_create_market_order_unsigned_tx()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(
			CreateMarketOrderUnsignedTxBody {
				request_id: 0,
				chain_id: to_chain_id,
				token_ca: "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_string(),
				swap_type: SwapType::Buy,
				amount_in: "998.000000000000000000".to_string(),
				double_out: false,
				is_one_click: false,
				address: accounting_contract_client_address.to_string(),
				is_anti_mev: false,
				is_auto_slippage: false,
				gas_type: GasType::Slow,
				slippage: 0,
				wallet_index: 1,
				recipient_address: expected_payout_address.to_string()
			}
		))
		.times(1)
		.returning(move |_, _| {
			Ok(CreateMarketOrderUnsignedTxResponse {
				code: 0,
				data: CreateMarketOrderUnsignedTxResponseData {
					chain_id: Some(to_chain_id),
					order_id: Some(order_id),
					tx_data: Some(
						vec![format!("0x{}", hex::encode(create_order_encoded_tx.clone()))]
					),
				},
				message: "".to_string(),
			})
		});

	pumpx_api_mock
		.expect_send_order_tx()
		.with(
			mockall::predicate::eq("test_token"),
			mockall::predicate::always(),
			// mockall::predicate::eq(SendOrderTxBody {
			// 	chain_id: to_chain_id,
			// 	order_id: order_id,
			// 	tx_data: vec!["0xf869808504e3b29200831e848094d8da6bf26964af9d7eed9e03e53415d37aa96045843b9aca008025a0f6c76effbee5ac9ff341a0efe85b5f9401ab673c5d1c2746041e446e3d19aea3a037470aac13f1cb4da20f086a475cceb36a5bb4fd9cf52b48300ad840ad1480ad".to_string()]
			// })
		)
		.times(1)
		.returning(|_, _| {
			Ok(SendOrderTxResponse {
				code: 0,
				data: SendOrderTxResponseData { tx_hash: None },
				message: "".to_string(),
			})
		});

	let mut binance_api_mock = binance_api::mocks::MockBinanceApiClient::new();

	binance_api_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(vec![prepare_sol_coin_info(solana_coin_ticker, solana_coin_name)])
		});

	binance_api_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/deposit/address"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::always(),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(DepositAddress {
				address: binance_deposit_address.to_string(),
				coin: solana_coin_ticker.to_string(),
				tag: "tag".to_string(),
				url: "url".to_string(),
			})
		});

	let mut sol_usdt_price_params: HashMap<String, String> = HashMap::new();
	sol_usdt_price_params.insert("symbol".to_string(), "SOLUSDT".to_string());
	binance_api_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_usdt_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut sol_bnb_price_params: HashMap<String, String> = HashMap::new();
	sol_bnb_price_params.insert("symbol".to_string(), "SOLBNB".to_string());
	binance_api_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_bnb_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut accounting_contract_client_mock =
		accounting_contract_client::mocks::MockAccountingContractClient::new();

	accounting_contract_client_mock
		.expect_get_signer_address()
		.times(1)
		.returning(|| Address::from_str(accounting_contract_client_address).unwrap());

	let mut solana_client_mock = solana::mocks::MockSolanaRpcClient::new();

	solana_client_mock
		.expect_get_balance()
		.times(1)
		.returning(|_| Ok(100_000_000_000));

	solana_client_mock
		.expect_transfer_sol()
		.withf(move |to: &str, value, _| to == binance_deposit_address && value == &100000000000)
		.times(1)
		.returning(|_, _, _| Ok("".to_string()));

	let mut rpc_endpoint_registry = RpcEndpointRegistry::new();
	rpc_endpoint_registry.insert(Chain::Solana, "http://solana-rpc.io".to_string());
	let pumpx_signer_client: Arc<Box<dyn SignerClient>> =
		Arc::new(Box::new(pumpx_signer_client_mock));
	let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(pumpx_api_mock));
	let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
	let binance_api = Arc::new(binance_api_mock);
	let solana_client = Arc::new(solana_client_mock);
	let accounting_contract_client: Arc<Box<dyn AccountingContractApi>> =
		Arc::new(Box::new(accounting_contract_client_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("1".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(to_chain_id, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider =
		SingleChainSwapProvider::Pumpx(prepare_pumpx_config(to_chain_id, pumpx_wallet_index));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock.clone(),
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		binance_api,
		solana_client,
		accounting_contract_client,
		Decimal::from_str("100000").unwrap(),
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = PumpxJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
	assert_eq!(
		account_assets_lock.get_locked_amount(
			&account_id,
			intent_asset_lock::AssetId::Solana(SolanaToken::Native)
		),
		Ok(AmountType::from(0))
	);
}

fn prepare_sol_coin_info(ticker: &str, name: &str) -> CoinInfo {
	CoinInfo {
		coin: ticker.to_string(),
		deposit_all_enable: false,
		free: "".to_string(),
		freeze: "".to_string(),
		ipoable: "".to_string(),
		ipoing: "".to_string(),
		is_legal_money: false,
		locked: "".to_string(),
		name: name.to_string(),
		network_list: vec![NetworkInfo {
			address_regex: "".to_string(),
			coin: ticker.to_string(),
			deposit_desc: None,
			deposit_enable: false,
			is_default: false,
			memo_regex: "".to_string(),
			min_confirm: 0,
			name: name.to_string(),
			network: "SOL".to_string(),
			special_tips: None,
			special_withdraw_tips: None,
			un_lock_confirm: None,
			withdraw_desc: None,
			withdraw_enable: None,
			withdraw_fee: None,
			withdraw_integer_multiple: None,
			withdraw_max: None,
			withdraw_min: None,
			withdraw_internal_min: None,
			same_address: None,
			estimated_arrival_time: None,
			busy: None,
			contract_address_url: None,
			contract_address: None,
			reset_address_status: None,
			deposit_dust: None,
			denomination: None,
		}],
		storage: None,
		trading: None,
		withdraw_all_enable: None,
		withdrawing: None,
	}
}

fn prepare_pumpx_config(to_chain_id: u32, wallet_index: u32) -> PumpxConfig {
	PumpxConfig {
		order_type: PumpxOrderType::Market,
		swap_type: 1,
		from_chain_id: 100000,
		from_token_ca: BoundedVec::truncate_from([0u8; 128].to_vec()),
		to_chain_id,
		to_token_ca: BoundedVec::truncate_from([0u8; 128].to_vec()),
		from_amount: BoundedVec::truncate_from("100".as_bytes().to_vec()),
		double_out: false,
		is_one_click: false,
		is_anti_mev: false,
		is_auto_slippage: false,
		gas_type: 1,
		slippage: 0,
		wallet_index,
		token_cap: None,
		price_usd: None,
		usd_worth: BoundedVec::truncate_from([0u8; 128].to_vec()),
		trailing_percent: None,
	}
}
