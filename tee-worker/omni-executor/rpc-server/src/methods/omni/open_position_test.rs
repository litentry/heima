use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::types::{RpcOptionExt, RpcResultExt};
use crate::utils::user_op::submit_corewriter_user_ops;
use crate::utils::validation::{parse_as, parse_rpc_params};
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::ChainId;
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Deserialize)]
pub struct OpenPositionTestParams {
	pub sender: String,
	pub nonce: u128,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub ticker: String,
	pub position_size: String,
}

#[derive(Serialize, Clone)]
pub struct OpenPositionTestResponse {
	pub cloid: String,
	pub tx_hash: Option<String>,
}

pub fn register_open_position_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_openPositionTest", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<OpenPositionTestParams>(params)?;

			debug!("Received omni_openPositionTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account)?;

			let smart_wallet = &params.sender;
			let ticker = &params.ticker;

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|_| {
				DetailedError::internal_error("HyperCore client error").to_rpc_error()
			})?;

			let (perp_meta, perp_mark_price, perp_mid_price) = hypercore_client
				.get_perp_market_prices(ticker)
				.await
				.map_err_internal("Failed to get perp market prices")?;

			let perp_asset_id = get_perp_asset_id(ticker, &perp_meta)
				.map_err_internal("Failed to get perp asset id")?;

			let perp_asset = perp_meta
				.universe
				.get(perp_asset_id as usize)
				.ok_or_internal("Perp asset not found in meta")?;

			let perp_sz_decimals = perp_asset.sz_decimals;

			let (_, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
			let target_hedge_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
			let hedge_size: f64 = parse_as(&params.position_size, "position_size")?;

			let clamped_hedge_size = clamp_size(hedge_size, perp_sz_decimals);
			let clamped_hedge_price = clamp_price(target_hedge_price, perp_sz_decimals, false);

			let clamped_hedge_size_f64: f64 = parse_as(&clamped_hedge_size, "clamped_hedge_size")?;
			let clamped_hedge_price_f64: f64 =
				parse_as(&clamped_hedge_price, "clamped_hedge_price")?;

			let cloid = generate_cloid();

			let hedge_action = build_perp_long_order(
				perp_asset_id,
				to_price_units(clamped_hedge_size_f64),
				to_price_units(clamped_hedge_price_f64),
				cloid,
			);

			let hedge_open_tx_hash = submit_corewriter_user_ops(
				ctx.clone(),
				&omni_account,
				&generate_userop(smart_wallet, params.nonce),
				params.chain_id,
				params.wallet_index,
				encode_omni_account_execute(
					get_core_writer_address(),
					encode_send_raw_action(hedge_action),
				),
			)
			.await?;

			info!(
				"Hedge position submitted: price={}, size={}, tx={:?}",
				clamped_hedge_price_f64, clamped_hedge_size_f64, hedge_open_tx_hash
			);

			let order_opened = hypercore_client
				.wait_for_order(smart_wallet, &cloid.to_string(), 20, OrderWaitCondition::Opened)
				.await
				.map_err(|e| {
					DetailedError::internal_error(&format!("Hedge order failed to open: {}", e))
						.to_rpc_error()
				})?;

			if !order_opened {
				return Err(DetailedError::internal_error("Hedge order was rejected or canceled")
					.to_rpc_error());
			}

			hypercore_client.print_account_state(smart_wallet, "After Open Position").await;

			Ok(OpenPositionTestResponse { cloid: cloid.to_string(), tx_hash: hedge_open_tx_hash })
		})
		.expect("Failed to register omni_requestLoanTest method");
}

fn generate_userop(sender: &str, nonce: u128) -> SerializablePackedUserOperation {
	SerializablePackedUserOperation {
			sender: sender.to_string(),
			nonce,
	 init_code: "0x".to_string(),
	 call_data: "0x".to_string(),
	 account_gas_limits: "0x000000000000000000000000000f4240000000000000000000000000000186a0".to_string(),
	 pre_verification_gas: 100000,
	 gas_fees: "0x0000000000000000000000000000000000000000000000000000000007270e00".to_string(),
	 paymaster_and_data: "0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912000000000000000000000000000f4240000000000000000000000000000186a0".to_string(),
	 signature: None,
	}
}
