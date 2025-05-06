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
use heima_primitives::IdentityString;
use pumpx::signer_client::SignerClient;
use pumpx::types::CreateMarketOrderTxBody;
use pumpx::types::GasType;
use pumpx::types::SwapType;
use pumpx::types::{
	CreateMarketOrderTxResponse, CreateMarketOrderTxResponseData, GasInfo, GetGasInfoResponse,
	GetGasInfoResponseData, OrderInfoResponse, OrderInfoResponseData,
};
use pumpx::PumpxApi;
use reqwest::Method;
use sp_core::bounded::BoundedVec;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn simple_cross_chain_swap() {
	let tmp_dir = tempdir().unwrap();

	let account_id: AccountId =
		Identity::Pumpx(IdentityString::new("1".as_bytes().to_vec())).to_omni_account();

	let intent_id = 0;

	let order = SwapOrder {
		from_amount: BoundedVec::truncate_from("101".as_bytes().to_vec()),
		from_asset: ChainAsset::Solana(SolanaToken::Native),
		to_asset: ChainAsset::Ethereum(56, EthereumToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(prepare_pumpx_config());

	let intent = Intent::Swap(order, None, single_chain_swap_provider);

	let mut pumpx_signer_client_mock = pumpx::signer_client_mocks::MockSignerClient::new();

	// this is called twice, one for solana address and later for ethereum address
	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Solana),
			mockall::predicate::eq(1),
			mockall::predicate::eq([
				127, 34, 2, 199, 225, 243, 79, 58, 208, 100, 126, 151, 198, 62, 176, 11, 14, 171,
				116, 52, 128, 14, 245, 109, 68, 26, 222, 183, 80, 165, 126, 31,
			]),
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
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(pumpx::signer_client::ChainType::Evm),
			mockall::predicate::eq(1),
			mockall::predicate::eq([
				127, 34, 2, 199, 225, 243, 79, 58, 208, 100, 126, 151, 198, 62, 176, 11, 14, 171,
				116, 52, 128, 14, 245, 109, 68, 26, 222, 183, 80, 165, 126, 31,
			]),
		)
		.times(1)
		.returning(|_, _, _| {
			Ok([
				3, 101, 219, 24, 34, 145, 151, 225, 255, 131, 94, 15, 170, 236, 154, 154, 153, 0,
				176, 174, 181, 241, 142, 63, 170, 90, 76, 166, 12, 128, 33, 61, 124,
			]
			.to_vec())
		});

	let mut pumpx_api_mock = pumpx::mocks::MockPumpxApiClient::new();
	pumpx_api_mock.expect_create_cross_order().times(1).returning(|_, _| {
		Ok(OrderInfoResponse {
			code: 0,
			data: OrderInfoResponseData { order_id: Some(1) },
			message: "".to_string(),
		})
	});
	pumpx_api_mock
		.expect_get_gas_info()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(56))
		.times(1)
		.returning(|_, _| {
			Ok(GetGasInfoResponse {
				code: 0,
				data: GetGasInfoResponseData {
					gas_info: Some(vec![GasInfo {
						chain_id: "56".to_string(),
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

	pumpx_api_mock.expect_create_market_order_tx()
		.with(mockall::predicate::eq("test_token"), mockall::predicate::eq(CreateMarketOrderTxBody {
			request_id: 0,
			chain_id: 56,
			token_ca: "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_string(),
			swap_type: SwapType::Buy,
			amount_in: "998.000000000000000000".to_string(),
			double_out: false,
			is_one_click: false,
			address: "0x8Fc876ca8b23Ef6b735e388Ea94327e189DD5Ea7".to_string(),
			is_anti_mev: false,
			is_auto_slippage: false,
			gas_type: GasType::Slow,
			slippage: 0,
			wallet_index: 1
		}))
		.times(1)
		.returning(|_, _| {
			Ok(CreateMarketOrderTxResponse {
				code: 0,
				data: CreateMarketOrderTxResponseData {
					chain_id: None,
					order_id: None,
					tx_hash: None,
				},
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
		.returning(|_, _, _, _| Ok(vec![prepare_sol_coin_info()]));

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
				address: "deposit_address".to_string(),
				coin: "SOL".to_string(),
				tag: "tag".to_string(),
				url: "url".to_string(),
			})
		});

	binance_api_mock
		.expect_make_public_get_request()
		.with(mockall::predicate::eq("/api/v3/ticker/price"), mockall::predicate::always())
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
			mockall::predicate::eq(
				Address::from_str("0x8fc876ca8b23ef6b735e388ea94327e189dd5ea7").unwrap(),
			),
			mockall::predicate::eq(U256::from(2)),
			mockall::predicate::eq(U256::from_str("999000000000000000000").unwrap()),
		)
		.times(1)
		.returning(|_, _, _| Ok(()));

	let mut solana_client_mock = solana::mocks::MockSolanaRpcClient::new();

	solana_client_mock
		.expect_transfer_sol()
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

	let executor = CrossChainIntentExecutor::new(
		rpc_endpoint_registry,
		pumpx_signer_client.clone(),
		pumpx_api,
		storage_db.clone(),
		binance_api,
		solana_client,
		accounting_contract_client,
	)
	.unwrap();

	// store jwt token
	let pumpx_jwt_storage = PumpxJwtStorage::new(storage_db.clone());
	pumpx_jwt_storage
		.insert(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE), "test_token".to_string())
		.unwrap();

	executor.execute(&account_id, intent_id, intent).await.unwrap();
}

fn prepare_sol_coin_info() -> CoinInfo {
	CoinInfo {
		coin: "SOL".to_string(),
		deposit_all_enable: false,
		free: "".to_string(),
		freeze: "".to_string(),
		ipoable: "".to_string(),
		ipoing: "".to_string(),
		is_legal_money: false,
		locked: "".to_string(),
		name: "Solana".to_string(),
		network_list: vec![NetworkInfo {
			address_regex: "".to_string(),
			coin: "SOL".to_string(),
			deposit_desc: None,
			deposit_enable: false,
			is_default: false,
			memo_regex: "".to_string(),
			min_confirm: 0,
			name: "SOL".to_string(),
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

fn prepare_pumpx_config() -> PumpxConfig {
	PumpxConfig {
		order_type: PumpxOrderType::Limit,
		swap_type: 1,
		from_chain_id: 100000,
		from_token_ca: BoundedVec::truncate_from([0u8; 128].to_vec()),
		to_chain_id: 56,
		to_token_ca: BoundedVec::truncate_from([0u8; 128].to_vec()),
		from_amount: BoundedVec::truncate_from("100".as_bytes().to_vec()),
		double_out: false,
		is_one_click: false,
		is_anti_mev: false,
		is_auto_slippage: false,
		gas_type: 1,
		slippage: 0,
		wallet_index: 1,
		token_cap: None,
		price_usd: None,
		usd_worth: BoundedVec::truncate_from([0u8; 128].to_vec()),
		trailing_percent: None,
	}
}
