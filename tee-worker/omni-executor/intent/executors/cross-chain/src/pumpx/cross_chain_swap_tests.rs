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

use crate::types::Chain;
use crate::CrossChainIntentExecutor;
use crate::RpcEndpointRegistry;
use crate::U256;
use alloy::primitives::Address;
use heima_primitives::BoundedVec;
use oe_client_accounting::mocks::MockAccountingContractClient;
use oe_client_accounting::solana::mocks::MockAccountingContractClient as SolanaMockAccountingContractClient;
use oe_client_accounting::AccountingContractApi;
use oe_client_binance::spot_trading_api::types::SymbolPrice;
use oe_client_binance::wallet_api::types::{CoinInfo, DepositAddress, NetworkInfo};
use oe_client_pumpx::methods::common::GasType;
use oe_client_pumpx::methods::common::OrderInfoResponse;
use oe_client_pumpx::methods::common::OrderInfoResponseData;
use oe_client_pumpx::methods::common::SwapType;
use oe_client_pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use oe_client_pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxResponse;
use oe_client_pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxResponseData;
use oe_client_pumpx::methods::get_gas_info::GasInfo;
use oe_client_pumpx::methods::get_gas_info::GetGasInfoResponse;
use oe_client_pumpx::methods::get_gas_info::GetGasInfoResponseData;
use oe_client_pumpx::methods::send_order_tx::SendOrderTxBody;
use oe_client_pumpx::methods::send_order_tx::SendOrderTxResponse;
use oe_client_pumpx::methods::send_order_tx::SendOrderTxResponseData;
use oe_client_pumpx::PumpxApi;
use oe_client_signer::mocks::MockSignerClient;
use oe_client_signer::SignerClient;
use oe_core::auth::constants::AUTH_TOKEN_ACCESS_TYPE;
use oe_core::auth::constants::CLIENT_ID_PUMPX;
use oe_core::intent::asset_lock::precise::PreciseAssetsLock;
use oe_core::intent::asset_lock::AccountAssetLocks;
use oe_core::intent::asset_lock::AmountType;
use oe_core::intent::executor::IntentExecutor;
use oe_primitives::ChainAsset;
use oe_primitives::Identity;
use oe_primitives::Intent;
use oe_primitives::PumpxAccountProfile;
use oe_primitives::PumpxOrderType;
use oe_primitives::SingleChainSwapProvider;
use oe_primitives::SolanaToken;
use oe_primitives::{EthereumToken, PumpxConfig, SwapOrder};
use oe_storage::Storage;
use oe_storage::StorageDB;
use oe_storage::{HeimaJwtStorage, PumpxProfileStorage};
use reqwest::Method;
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempdir;
use test_log::test;

#[test(tokio::test)]
async fn simple_cross_chain_swap_sol_to_bsc() {
	let account_id = Identity::Pumpx("1".into()).to_omni_account(CLIENT_ID_PUMPX);

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = oe_client_signer::mocks::MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] = account_id.clone().into();
	let from_chain_id = 100000;
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
			mockall::predicate::eq(oe_client_signer::ChainType::Solana),
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
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
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
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| Ok(vec![create_order_tx_signature.to_vec()]));

	let mut pumpx_api_mock = oe_client_pumpx::mocks::MockPumpxApiClient::new();
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

	let mut oe_client_binance_mock = oe_client_binance::mocks::MockBinanceApiClient::new();

	oe_client_binance_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(vec![prepare_coin_info(solana_coin_ticker, solana_coin_name, "SOL")])
		});

	oe_client_binance_mock
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
	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_usdt_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut sol_bnb_price_params: HashMap<String, String> = HashMap::new();
	sol_bnb_price_params.insert("symbol".to_string(), "SOLBNB".to_string());
	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_bnb_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut evm_oe_client_accounting_mock = MockAccountingContractClient::new();
	let solana_oe_client_accounting_mock = SolanaMockAccountingContractClient::new();

	evm_oe_client_accounting_mock
		.expect_get_balance()
		.times(1)
		.returning(|| Ok(U256::from_str_radix("1000000000000000000000", 10).unwrap()));

	evm_oe_client_accounting_mock
		.expect_get_nonce()
		.times(1)
		.returning(|_| Ok(U256::from(1)));

	evm_oe_client_accounting_mock
		.expect_execute_pay_out_request()
		.with(
			mockall::predicate::eq(Address::from_str(expected_payout_address).unwrap()),
			mockall::predicate::eq(U256::from(2)),
			mockall::predicate::eq(U256::from_str("999000000000000000000").unwrap()),
		)
		.times(1)
		.returning(|_, _, _| Ok(()));

	let bsc_client_mock = oe_client_ethereum::client::mocks::MockEthereumRpcClient::new();
	let mut solana_client_mock = oe_client_solana::mocks::MockSolanaRpcClient::new();

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
	let oe_client_binance = Arc::new(oe_client_binance_mock);
	let bsc_client = Arc::new(bsc_client_mock);
	let solana_client = Arc::new(solana_client_mock);
	let evm_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Address, U256>>> =
		Arc::new(Box::new(evm_oe_client_accounting_mock));
	let solana_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Pubkey, u64>>> =
		Arc::new(Box::new(solana_oe_client_accounting_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("101".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(to_chain_id, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(prepare_pumpx_config(
		from_chain_id,
		to_chain_id,
		pumpx_wallet_index,
	));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider.try_into().unwrap());

	// Placeholder AA contract addresses for testing
	let factory_address = Address::from_slice(&[0u8; 20]);
	let implementation_address = Address::from_slice(&[1u8; 20]);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock,
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		oe_client_binance,
		bsc_client,
		solana_client,
		evm_oe_client_accounting,
		solana_oe_client_accounting,
		Decimal::from(1),
		factory_address,
		implementation_address,
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = HeimaJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
}

#[test(tokio::test)]
async fn simple_cross_chain_swap_bsc_to_sol() {
	let account_id = Identity::Pumpx("1".into()).to_omni_account(CLIENT_ID_PUMPX);

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = oe_client_signer::mocks::MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] = account_id.clone().into();
	let from_chain_id = 56;
	let to_chain_id = 100000;
	let order_id = 0;
	let expected_payout_address = "B9umkjBoYNxyajiVwty2f6W3WWG2tf5SmkCz6KqWrv5f";
	let binance_deposit_address = "binance_deposit_address_bsc";
	let bsc_coin_ticker = "BNB";
	let bsc_coin_name = "Binance Coin";

	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(2)
		.returning(|_, _, _| {
			Ok([
				3, 101, 219, 24, 34, 145, 151, 225, 255, 131, 94, 15, 170, 236, 154, 154, 153, 0,
				176, 174, 181, 241, 142, 63, 170, 90, 76, 166, 12, 128, 33, 61, 124,
			]
			.to_vec())
		});

	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Solana),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok([
				150, 221, 47, 78, 207, 124, 147, 48, 228, 240, 229, 138, 142, 98, 114, 103, 47,
				239, 238, 32, 136, 87, 205, 119, 46, 138, 161, 50, 123, 57, 219, 250,
			]
			.to_vec())
		});

	pumpx_signer_client_mock
		.expect_request_signatures()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Solana),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| {
			Ok(vec![vec![
				175, 134, 93, 15, 148, 42, 139, 142, 212, 48, 148, 7, 200, 57, 255, 237, 249, 2,
				160, 205, 83, 43, 169, 70, 36, 16, 14, 49, 208, 238, 201, 60, 79, 255, 96, 73, 159,
				30, 71, 132, 22, 27, 4, 100, 241, 4, 202, 175, 230, 96, 179, 52, 106, 65, 1, 128,
				53, 130, 202, 9, 13, 94, 105, 14,
			]])
		});

	let mut pumpx_api_mock = oe_client_pumpx::mocks::MockPumpxApiClient::new();
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
				amount_in: "8.990000000".to_string(),
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
						vec!["0x0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001031faf636aea762c1a1fdc1e067cdae804c8990d13ab425fb7d6f49751ef0eacd50000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102020001080000000000000000".to_string()]
					),
				},
				message: "".to_string(),
			})
		});

	pumpx_api_mock
		.expect_send_order_tx()
		.with(
			mockall::predicate::eq("test_token"),
			mockall::predicate::eq(SendOrderTxBody {
				chain_id: to_chain_id,
				order_id,
				tx_data: vec!["0x01af865d0f942a8b8ed4309407c839ffedf902a0cd532ba94624100e31d0eec93c4fff60499f1e4784161b0464f104caafe660b3346a4101803582ca090d5e690e010001031faf636aea762c1a1fdc1e067cdae804c8990d13ab425fb7d6f49751ef0eacd50000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102020001080000000000000000".to_string()]
			}),
		)
		.times(1)
		.returning(|_, _| {
			Ok(SendOrderTxResponse {
				code: 0,
				data: SendOrderTxResponseData { tx_hash: None },
				message: "".to_string(),
			})
		});

	let mut oe_client_binance_mock = oe_client_binance::mocks::MockBinanceApiClient::new();

	oe_client_binance_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| Ok(vec![prepare_coin_info(bsc_coin_ticker, bsc_coin_name, "BSC")]));

	oe_client_binance_mock
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
				coin: bsc_coin_ticker.to_string(),
				tag: "tag".to_string(),
				url: "url".to_string(),
			})
		});

	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(mockall::predicate::eq("/api/v3/ticker/price"), mockall::predicate::always())
		//todo why 2 times?
		.times(2)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let evm_oe_client_accounting_mock = MockAccountingContractClient::new();
	let mut solana_oe_client_accounting_mock = SolanaMockAccountingContractClient::new();

	solana_oe_client_accounting_mock
		.expect_get_balance()
		.times(1)
		.returning(|| Ok(U256::from_str_radix("5000000000000", 10).unwrap()));

	solana_oe_client_accounting_mock
		.expect_get_nonce()
		.times(1)
		.returning(|_| Ok(1u64));

	solana_oe_client_accounting_mock
		.expect_execute_pay_out_request()
		.with(
			mockall::predicate::eq(Pubkey::from_str(expected_payout_address).unwrap()),
			mockall::predicate::eq(2u64),
			mockall::predicate::eq(U256::from(9990000000u64)),
		)
		.times(1)
		.returning(|_, _, _| Ok(()));

	let mut bsc_client_mock = oe_client_ethereum::client::mocks::MockEthereumRpcClient::new();
	let solana_client_mock = oe_client_solana::mocks::MockSolanaRpcClient::new();

	bsc_client_mock
		.expect_transfer()
		.withf(move |to: &str, value, _| {
			to == binance_deposit_address
				&& *value == U256::from_str_radix("100000000000000000000", 10).unwrap()
		})
		.times(1)
		.returning(|_, _, _| Ok("".to_string()));

	let rpc_endpoint_registry = RpcEndpointRegistry::new();
	let pumpx_signer_client: Arc<Box<dyn SignerClient>> =
		Arc::new(Box::new(pumpx_signer_client_mock));
	let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(pumpx_api_mock));
	let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
	let oe_client_binance = Arc::new(oe_client_binance_mock);
	let bsc_client = Arc::new(bsc_client_mock);
	let solana_client = Arc::new(solana_client_mock);
	let evm_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Address, U256>>> =
		Arc::new(Box::new(evm_oe_client_accounting_mock));
	let solana_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Pubkey, u64>>> =
		Arc::new(Box::new(solana_oe_client_accounting_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("101".as_bytes().to_vec()),
		from_asset: ChainAsset::Ethereum(from_chain_id, EthereumToken::Native),
		to_asset: ChainAsset::Solana(SolanaToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(prepare_pumpx_config(
		from_chain_id,
		to_chain_id,
		pumpx_wallet_index,
	));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider.try_into().unwrap());

	// Placeholder AA contract addresses for testing
	let factory_address = Address::from_slice(&[0u8; 20]);
	let implementation_address = Address::from_slice(&[1u8; 20]);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock.clone(),
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		oe_client_binance,
		bsc_client,
		solana_client,
		evm_oe_client_accounting,
		solana_oe_client_accounting,
		Decimal::from_str("1").unwrap(),
		factory_address,
		implementation_address,
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = HeimaJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
	assert_eq!(
		account_assets_lock.get_locked_amount(
			&account_id,
			oe_core::intent::asset_lock::AssetId::Solana(SolanaToken::Native)
		),
		Ok(AmountType::from(0))
	);
}

#[test(tokio::test)]
async fn instant_payout_cross_chain_swap() {
	let account_id = Identity::Pumpx("1".into()).to_omni_account(CLIENT_ID_PUMPX);

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] = account_id.clone().into();
	let from_chain_id = 100000;
	let to_chain_id = 56;
	let order_id = 0;
	let expected_payout_address = "0x8Fc876ca8b23Ef6b735e388Ea94327e189DD5Ea7";
	let binance_deposit_address = "binance_deposit_address";
	let solana_coin_ticker = "SOL";
	let solana_coin_name = "Solana";
	let evm_oe_client_accounting_address = "0x7CE3464A6dc52001754b0b90878d9Ffd61B47c6C";

	// this is called twice, one for solana address and later for ethereum address
	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Solana),
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
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
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
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| Ok(vec![create_order_tx_signature.to_vec()]));

	let mut pumpx_api_mock = oe_client_pumpx::mocks::MockPumpxApiClient::new();
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
				address: evm_oe_client_accounting_address.to_string(),
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

	let mut oe_client_binance_mock = oe_client_binance::mocks::MockBinanceApiClient::new();

	oe_client_binance_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		// second call is done in separate thread when doing deposit asynchronously in case of instant swap
		.times(2)
		.returning(|_, _, _, _| {
			Ok(vec![prepare_coin_info(solana_coin_ticker, solana_coin_name, "SOL")])
		});

	oe_client_binance_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/deposit/address"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::always(),
			mockall::predicate::eq(None),
		)
		// second call is done in separate thread when doing deposit asynchronously in case of instant swap
		.times(2)
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
	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_usdt_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut sol_bnb_price_params: HashMap<String, String> = HashMap::new();
	sol_bnb_price_params.insert("symbol".to_string(), "SOLBNB".to_string());
	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_bnb_price_params)),
		)
		// second call is done in separate thread when doing deposit asynchronously in case of instant swap
		.times(2)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut evm_oe_client_accounting_mock = MockAccountingContractClient::new();
	let solana_oe_client_accounting_mock = SolanaMockAccountingContractClient::new();

	evm_oe_client_accounting_mock
		.expect_get_signer_address()
		.times(1)
		.returning(|| Address::from_str(evm_oe_client_accounting_address).unwrap());

	let bsc_client_mock = oe_client_ethereum::client::mocks::MockEthereumRpcClient::new();
	let mut solana_client_mock = oe_client_solana::mocks::MockSolanaRpcClient::new();

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
	let oe_client_binance = Arc::new(oe_client_binance_mock);
	let bsc_client = Arc::new(bsc_client_mock);
	let solana_client = Arc::new(solana_client_mock);
	let evm_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Address, U256>>> =
		Arc::new(Box::new(evm_oe_client_accounting_mock));
	let solana_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Pubkey, u64>>> =
		Arc::new(Box::new(solana_oe_client_accounting_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("1".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(to_chain_id, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(prepare_pumpx_config(
		from_chain_id,
		to_chain_id,
		pumpx_wallet_index,
	));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider.try_into().unwrap());

	// Placeholder AA contract addresses for testing
	let factory_address = Address::from_slice(&[0u8; 20]);
	let implementation_address = Address::from_slice(&[1u8; 20]);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock.clone(),
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		oe_client_binance,
		bsc_client,
		solana_client,
		evm_oe_client_accounting,
		solana_oe_client_accounting,
		Decimal::from_str("100000").unwrap(),
		factory_address,
		implementation_address,
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = HeimaJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
	// give a background task a chance to remove locks
	tokio::task::yield_now().await;
	assert_eq!(
		account_assets_lock.get_locked_amount(
			&account_id,
			oe_core::intent::asset_lock::AssetId::Solana(SolanaToken::Native)
		),
		Ok(AmountType::from(0))
	);
}

#[test(tokio::test)]
async fn no_instant_payout_if_exported_wallet() {
	let account_id = Identity::Pumpx("1".into()).to_omni_account(CLIENT_ID_PUMPX);

	// ************************ MOCKS SETUP ************************
	let tmp_dir = tempdir().unwrap();

	let mut pumpx_signer_client_mock = MockSignerClient::new();

	let pumpx_wallet_index = 1;
	let pumpx_wallet_omni_account: [u8; 32] = account_id.clone().into();
	let from_chain_id = 100000;
	let to_chain_id = 56;
	let order_id = 0;
	let expected_payout_address = "0x8Fc876ca8b23Ef6b735e388Ea94327e189DD5Ea7";
	let binance_deposit_address = "binance_deposit_address";
	let solana_coin_ticker = "SOL";
	let solana_coin_name = "Solana";

	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Solana),
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
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok(hex::decode("0365db18229197e1ff835e0faaec9a9a9900b0aeb5f18e3faa5a4ca60c80213d7c")
				.unwrap()
				.to_vec())
		});

	let mut pumpx_api_mock = oe_client_pumpx::mocks::MockPumpxApiClient::new();
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

	let create_order_encoded_tx: Vec<u8> = hex::decode(
		"e9808504e3b29200831e848094d8da6bf26964af9d7eed9e03e53415d37aa96045843b9aca0080018080",
	)
	.unwrap();
	let create_order_tx_signature: Vec<u8> = hex::decode("f6c76effbee5ac9ff341a0efe85b5f9401ab673c5d1c2746041e446e3d19aea337470aac13f1cb4da20f086a475cceb36a5bb4fd9cf52b48300ad840ad1480ad1b").unwrap();

	pumpx_signer_client_mock
		.expect_request_signatures()
		.with(
			mockall::predicate::eq(oe_client_signer::ChainType::Evm),
			mockall::predicate::eq(pumpx_wallet_index),
			mockall::predicate::eq(pumpx_wallet_omni_account),
			mockall::predicate::always(),
		)
		.times(1)
		.returning(move |_, _, _, _| Ok(vec![create_order_tx_signature.to_vec()]));

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

	let mut oe_client_binance_mock = oe_client_binance::mocks::MockBinanceApiClient::new();

	oe_client_binance_mock
		.expect_make_signed_request()
		.with(
			mockall::predicate::eq("/sapi/v1/capital/config/getall"),
			mockall::predicate::eq(Method::GET),
			mockall::predicate::eq(None),
			mockall::predicate::eq(None),
		)
		.times(1)
		.returning(|_, _, _, _| {
			Ok(vec![prepare_coin_info(solana_coin_ticker, solana_coin_name, "SOL")])
		});

	oe_client_binance_mock
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

	// let mut sol_usdt_price_params: HashMap<String, String> = HashMap::new();
	// sol_usdt_price_params.insert("symbol".to_string(), "SOLUSDT".to_string());
	// oe_client_binance_mock
	// 	.expect_make_public_get_request()
	// 	.with(
	// 		mockall::predicate::eq("/api/v3/ticker/price"),
	// 		mockall::predicate::eq(Some(sol_usdt_price_params)),
	// 	)
	// 	.times(1)
	// 	.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut sol_bnb_price_params: HashMap<String, String> = HashMap::new();
	sol_bnb_price_params.insert("symbol".to_string(), "SOLBNB".to_string());
	oe_client_binance_mock
		.expect_make_public_get_request()
		.with(
			mockall::predicate::eq("/api/v3/ticker/price"),
			mockall::predicate::eq(Some(sol_bnb_price_params)),
		)
		.times(1)
		.returning(|_, _| Ok(SymbolPrice { price: "10".to_string() }));

	let mut evm_oe_client_accounting_mock = MockAccountingContractClient::new();
	let solana_oe_client_accounting_mock = SolanaMockAccountingContractClient::new();

	evm_oe_client_accounting_mock
		.expect_get_balance()
		.times(1)
		.returning(|| Ok(U256::from_str_radix("1000000000000000000000", 10).unwrap()));

	evm_oe_client_accounting_mock
		.expect_get_nonce()
		.times(1)
		.returning(|_| Ok(U256::from(1)));

	evm_oe_client_accounting_mock
		.expect_execute_pay_out_request()
		.with(
			mockall::predicate::eq(Address::from_str(expected_payout_address).unwrap()),
			mockall::predicate::eq(U256::from(2)),
			mockall::predicate::eq(U256::from_str("999000000000000000000").unwrap()),
		)
		.times(1)
		.returning(|_, _, _| Ok(()));

	let bsc_client_mock = oe_client_ethereum::client::mocks::MockEthereumRpcClient::new();

	let mut solana_client_mock = oe_client_solana::mocks::MockSolanaRpcClient::new();

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
	let oe_client_binance = Arc::new(oe_client_binance_mock);
	let bsc_client = Arc::new(bsc_client_mock);
	let solana_client = Arc::new(solana_client_mock);
	let evm_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Address, U256>>> =
		Arc::new(Box::new(evm_oe_client_accounting_mock));
	let solana_oe_client_accounting: Arc<Box<dyn AccountingContractApi<Pubkey, u64>>> =
		Arc::new(Box::new(solana_oe_client_accounting_mock));

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("1".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(to_chain_id, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(prepare_pumpx_config(
		from_chain_id,
		to_chain_id,
		pumpx_wallet_index,
	));

	let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
		Arc::new(AccountAssetLocks::new(storage_db.clone()));

	let intent = Intent::Swap(order, None, single_chain_swap_provider.try_into().unwrap());

	// Placeholder AA contract addresses for testing
	let factory_address = Address::from_slice(&[0u8; 20]);
	let implementation_address = Address::from_slice(&[1u8; 20]);

	let executor = CrossChainIntentExecutor::new(
		account_assets_lock.clone(),
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		oe_client_binance,
		bsc_client,
		solana_client,
		evm_oe_client_accounting,
		solana_oe_client_accounting,
		Decimal::from_str("100000").unwrap(),
		factory_address,
		implementation_address,
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = HeimaJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	// store wallet exported information so it should skip instant payout
	let pumpx_account_profile_storage = PumpxProfileStorage::new(storage_db.clone());
	pumpx_account_profile_storage
		.insert(&account_id.clone(), PumpxAccountProfile { wallet_exported: true })
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
	// give a background task a chance to remove locks
	tokio::task::yield_now().await;
	assert_eq!(
		account_assets_lock.get_locked_amount(
			&account_id,
			oe_core::intent::asset_lock::AssetId::Solana(SolanaToken::Native)
		),
		Ok(AmountType::from(0))
	);
}

fn prepare_coin_info(ticker: &str, name: &str, network: &str) -> CoinInfo {
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
			network: network.to_string(),
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

fn prepare_pumpx_config(from_chain_id: u32, to_chain_id: u32, wallet_index: u32) -> PumpxConfig {
	PumpxConfig {
		order_type: PumpxOrderType::Market,
		swap_type: 1,
		from_chain_id,
		from_token_ca: String::from_utf8([0u8; 128].to_vec()).unwrap(),
		to_chain_id,
		to_token_ca: String::from_utf8([0u8; 128].to_vec()).unwrap(),
		from_amount: "100".to_string(),
		double_out: false,
		is_one_click: false,
		is_anti_mev: false,
		is_auto_slippage: false,
		gas_type: 1,
		slippage: 0,
		wallet_index,
		token_cap: None,
		price_usd: None,
		usd_worth: String::from_utf8([0u8; 128].to_vec()).unwrap(),
		trailing_percent: None,
	}
}
