use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_corewriter_userop;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::ChainId;
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct OpenPositionTestParams {
	pub sender: String,
	pub nonce: u128,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub client_id: String,
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
			let params = params.parse::<OpenPositionTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_openPositionTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
				error!("Failed to parse omni account");
				PumpxRpcError::from(DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse omni account",
				))
			})?;

			let smart_wallet = &params.sender;
			let ticker = &params.ticker;

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|e| {
				PumpxRpcError::from(
					DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported").with_reason(e),
				)
			})?;

			let (perp_meta, perp_mark_price, perp_mid_price) =
				hypercore_client.get_perp_market_prices(ticker).await.map_err(|e| {
					error!("Failed to get perp market prices for {}: {}", ticker, e);
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to get perp market prices: {}", e)),
					)
				})?;

			let perp_asset_id = get_perp_asset_id(ticker, &perp_meta).map_err(|e| {
				error!("Failed to get perp asset ID: {}", e);
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
				)
			})?;

			let perp_asset = perp_meta.universe.get(perp_asset_id as usize).ok_or_else(|| {
				error!("Perp asset {} not found in meta", perp_asset_id);
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason(format!("Perp asset {} not found", perp_asset_id)),
				)
			})?;

			let perp_sz_decimals = perp_asset.sz_decimals;

			let (_, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
			let target_hedge_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
			let hedge_size = params.position_size.parse::<f64>().unwrap();

			let clamped_hedge_size = clamp_size(hedge_size, perp_sz_decimals);
			let clamped_hedge_price = clamp_price(target_hedge_price, perp_sz_decimals, false);

			let clamped_hedge_size_f64 = clamped_hedge_size.parse::<f64>().map_err(|e| {
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason(format!("Failed to parse clamped hedge size: {}", e)),
				)
			})?;
			let clamped_hedge_price_f64 = clamped_hedge_price.parse::<f64>().map_err(|e| {
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason(format!("Failed to parse clamped hedge price: {}", e)),
				)
			})?;

			let cloid = generate_cloid();

			let hedge_action = build_perp_long_order(
				perp_asset_id,
				to_price_units(clamped_hedge_size_f64),
				to_price_units(clamped_hedge_price_f64),
				cloid,
			);

			let hedge_open_tx_hash = submit_corewriter_userop(
				ctx.clone(),
				&omni_account,
				&generate_userop(smart_wallet, params.nonce),
				params.chain_id,
				params.wallet_index,
				encode_omni_account_execute(
					get_core_writer_address(),
					encode_send_raw_action(hedge_action),
				),
				&params.client_id,
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
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Hedge order failed to open: {}", e)),
					)
				})?;

			if !order_opened {
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Hedge order was rejected or canceled"),
				));
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
