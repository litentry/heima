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

use crate::CrossChainIntentExecutor;
use crate::RpcEndpointRegistry;
use accounting_contract_client::AccountingContractApi;
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
use pumpx::methods::common::GasType;
use pumpx::methods::common::OrderInfoResponse;
use pumpx::methods::common::OrderInfoResponseData;
use pumpx::methods::common::SwapType;
use pumpx::methods::create_limit_order::CreateLimitOrderBody;
use pumpx::signer_client::{ChainType, SignerClient};
use pumpx::PumpxApi;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn simple_single_chain_swap() {
	let tmp_dir = tempdir().unwrap();

	let account_id: AccountId =
		Identity::Pumpx(IdentityString::new("1".as_bytes().to_vec())).to_omni_account();
	let intent_id = 0;

	let order = SwapOrder {
		//value below is ignored ?
		from_amount: BoundedVec::truncate_from("101".as_bytes().to_vec()),
		from_asset: ChainAsset::Ethereum(56, EthereumToken::Native),
		to_asset: ChainAsset::Solana(SolanaToken::Native),
		to_address: None,
	};

	let single_chain_swap_provider = SingleChainSwapProvider::Pumpx(PumpxConfig {
		order_type: PumpxOrderType::Limit,
		swap_type: 1,
		from_chain_id: 100000,
		from_token_ca: BoundedVec::truncate_from([0u8; 128].to_vec()),
		to_chain_id: 100000,
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
	});

	let intent = Intent::Swap(order, None, single_chain_swap_provider);

	let mut pumpx_signer_client_mock = pumpx::signer_client_mocks::MockSignerClient::new();
	pumpx_signer_client_mock
		.expect_request_wallet()
		.with(
			mockall::predicate::eq(ChainType::Solana),
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

	let mut pumpx_api_mock = pumpx::mocks::MockPumpxApiClient::new();
	pumpx_api_mock.expect_create_limit_order()
    .with(mockall::predicate::eq("test_token"),
    mockall::predicate::eq(CreateLimitOrderBody {
        request_id: 0,
        chain_id: 100000,
        token_ca: "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_string(),
        amount: "100".to_string(),
        swap_type: SwapType::Buy,
        double_out: false,
        token_cap: None,
        price_usd: None,
        trailing_percent: None,
        address: "B9umkjBoYNxyajiVwty2f6W3WWG2tf5SmkCz6KqWrv5f".to_string(),
        is_anti_mev: false,
        is_auto_slippage: false,
        gas_type: GasType::Slow,
        slippage: 0,
        wallet_index: 1
    }))
    .times(1).returning(|_, _| {
		Ok(OrderInfoResponse {
			code: 0,
			data: OrderInfoResponseData { order_id: Some(1) },
			message: "".to_string(),
		})
	});

	let rpc_endpoint_registry = RpcEndpointRegistry::new();
	let pumpx_signer_client: Arc<Box<dyn SignerClient>> =
		Arc::new(Box::new(pumpx_signer_client_mock));
	let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(pumpx_api_mock));
	let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
	let binance_api = Arc::new(binance_api::mocks::MockBinanceApiClient::new());
	let solana_client = Arc::new(solana::mocks::MockSolanaRpcClient::new());
	let accounting_contract_client: Arc<Box<dyn AccountingContractApi>> =
		Arc::new(Box::new(accounting_contract_client::mocks::MockAccountingContractClient::new()));

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
