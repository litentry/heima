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
pub struct ClosePositionTestParams {
	pub sender: String,
	pub nonce: u128,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub ticker: Option<String>, // If None, close all positions
}

#[derive(Serialize, Clone)]
pub struct ClosePositionTestResponse {
	pub positions_closed: Vec<PositionClosed>,
}

#[derive(Serialize, Clone)]
pub struct PositionClosed {
	pub ticker: String,
	pub cloid: String,
	pub size: String,
	pub tx_hash: Option<String>,
}

pub fn register_close_position_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_closePositionTest", |params, ctx, _ext| async move {
			let params = params.parse::<ClosePositionTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_closePositionTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
				error!("Failed to parse omni account");
				PumpxRpcError::from(DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse omni account",
				))
			})?;

			let smart_wallet = &params.sender;

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|e| {
				PumpxRpcError::from(
					DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported").with_reason(e),
				)
			})?;

			hypercore_client.print_account_state(smart_wallet, "Before Close Position").await;

			// Get current positions
			let perp_state =
				hypercore_client.get_perp_clearinghouse_state(smart_wallet).await.map_err(|e| {
					error!("Failed to get perp state: {}", e);
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to query perp state: {}", e)),
					)
				})?;

			// Filter positions based on ticker parameter
			let positions_to_close: Vec<_> = perp_state
				.asset_positions
				.iter()
				.filter(|pos| {
					let position_size = pos.position.szi.parse::<f64>().unwrap_or(0.0);
					if position_size == 0.0 {
						return false;
					}
					match &params.ticker {
						Some(ticker) => pos.position.coin.eq_ignore_ascii_case(ticker),
						None => true, // Close all positions
					}
				})
				.collect();

			if positions_to_close.is_empty() {
				info!("No positions to close");
				return Ok(ClosePositionTestResponse { positions_closed: vec![] });
			}

			info!("Found {} position(s) to close", positions_to_close.len());

			let mut positions_closed = Vec::new();
			let mut current_nonce = params.nonce;

			for pos in positions_to_close {
				let ticker = &pos.position.coin;
				let position_size = pos.position.szi.parse::<f64>().map_err(|e| {
					error!("Failed to parse position size for {}: {}", ticker, e);
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to parse position size: {}", e)),
					)
				})?;

				info!("Closing position for {}: size={}", ticker, position_size);

				// Get perp market data
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

				// For closing long position (selling), we want to sell at the highest buy price (bid)
				let close_size_abs = position_size.abs();
				let clamped_close_size = clamp_size(close_size_abs, perp_sz_decimals);
				let (perp_bid_price, _perp_ask_price) =
					get_bid_ask_prices(perp_mark_price, perp_mid_price);
				let target_close_price = perp_bid_price * PERP_CLOSE_PRICE_RATIO;
				let clamped_close_price = clamp_price(target_close_price, perp_sz_decimals, false);

				info!(
					"Perp close pricing for {} - markPx: {}, midPx: {}, bid (highest buy): {}, target (with {}x buffer): {}, clamped: {}",
					ticker, perp_mark_price, perp_mid_price, perp_bid_price, PERP_CLOSE_PRICE_RATIO, target_close_price, clamped_close_price
				);

				let clamped_close_size_f64 = clamped_close_size.parse::<f64>().map_err(|e| {
					error!("Failed to parse clamped close size: {}", e);
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to parse clamped close size: {}", e)),
					)
				})?;
				let clamped_close_price_f64 = clamped_close_price.parse::<f64>().map_err(|e| {
					error!("Failed to parse clamped close price: {}", e);
					PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to parse clamped close price: {}", e)),
					)
				})?;

				let close_size_units = to_price_units(clamped_close_size_f64);
				let close_price_units = to_price_units(clamped_close_price_f64);

				let close_cloid = generate_cloid();

				let close_action = build_perp_close_order(
					perp_asset_id,
					close_size_units,
					close_price_units,
					close_cloid,
				);

				let close_calldata = encode_omni_account_execute(
					get_core_writer_address(),
					encode_send_raw_action(close_action),
				);

				let close_tx_hash = submit_corewriter_userop(
					ctx.clone(),
					&omni_account,
					&generate_userop(smart_wallet, current_nonce),
					params.chain_id,
					params.wallet_index,
					close_calldata,
				)
				.await?;

				info!(
					"Close order submitted for {}: price={}, size={}, tx={:?}",
					ticker, clamped_close_price_f64, clamped_close_size_f64, close_tx_hash
				);

				current_nonce += 1;

				let order_filled = hypercore_client
					.wait_for_order(
						smart_wallet,
						&close_cloid.to_string(),
						30,
						OrderWaitCondition::Filled,
					)
					.await
					.map_err(|e| {
						error!("Close order did not complete for {}: {}", ticker, e);
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason(format!("Close order failed: {}", e)),
						)
					})?;

				if !order_filled {
					error!("Close order was rejected or canceled for {}", ticker);
					return Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Close order was rejected or canceled for {}", ticker)),
					));
				}

				info!("Position closed successfully for {}", ticker);

				positions_closed.push(PositionClosed {
					ticker: ticker.to_string(),
					cloid: close_cloid.to_string(),
					size: clamped_close_size.to_string(),
					tx_hash: close_tx_hash,
				});
			}

			hypercore_client.print_account_state(smart_wallet, "After Close Position").await;

			Ok(ClosePositionTestResponse { positions_closed })
		})
		.expect("Failed to register omni_closePositionTest method");
}

fn generate_userop(sender: &str, nonce: u128) -> SerializablePackedUserOperation {
	SerializablePackedUserOperation {
		sender: sender.to_string(),
		nonce,
		init_code: "0x".to_string(),
		call_data: "0x".to_string(),
		account_gas_limits: "0x000000000000000000000000000f4240000000000000000000000000000186a0"
			.to_string(),
		pre_verification_gas: 100000,
		gas_fees: "0x0000000000000000000000000000000000000000000000000000000007270e00".to_string(),
		paymaster_and_data:
			"0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912000000000000000000000000000f4240000000000000000000000000000186a0"
				.to_string(),
		signature: None,
	}
}
