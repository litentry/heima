use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::{prepare_skeleton_with_nonce, submit_corewriter_userop};
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::AccountId;
use executor_storage::Storage;
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Result of payback loan validation, containing loan details and action plan
struct PaybackLoanValidationResult {
	collateral_ticker: String,
	collateral_size: f64,
	usdc_sold: f64,
	usdc_loaned: f64,
	hedge_open_cloid: u128,
	should_cancel: bool,
	should_close: bool,
	position_size_to_close: f64,
	/// Position data: (unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable)
	position_data: Option<(f64, f64, f64, f64, f64)>,
}

#[derive(Debug, Deserialize)]
pub struct PaybackLoanTestParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: u64,
	pub wallet_index: u32,
	pub omni_account: String,
	pub loan_nonce: u64,
	// Expected minimal account value (USDC) on user's perp account (crossMarginSummary.accountValue)
	// If the actual account value is smaller, it will error out and not close the position.
	//
	// This is just a safety guard to avoid unwanted position close (when e.g user is at big loss)
	pub min_expected_account_value: String,
}

#[derive(Serialize, Clone)]
pub struct PaybackLoanTestResponse {
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub hedge_cancel_tx_hash: Option<String>,
	pub hedge_close_cloid: Option<String>,
	pub hedge_close_tx_hash: Option<String>,
	pub usd_transfer_tx_hash: Option<String>,
	pub spot_buy_cloid: Option<String>,
	pub spot_buy_tx_hash: Option<String>,
}

pub fn register_payback_loan_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_paybackLoanTest", |params, ctx, _ext| async move {
			let params = params.parse::<PaybackLoanTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_paybackLoanTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
				error!("Failed to parse omni account");
				PumpxRpcError::from(DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse omni account",
				))
			})?;

			// Call the handler implementation
			handle_payback_loan_impl(
				Arc::clone(&ctx),
				omni_account,
				params.user_operation,
				params.chain_id,
				params.wallet_index,
				params.loan_nonce,
				params.min_expected_account_value,
			)
			.await
		})
		.expect("Failed to register omni_paybackLoanTest method");
}

// Main handler implementation
async fn handle_payback_loan_impl<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	omni_account: AccountId,
	skeleton_user_op: SerializablePackedUserOperation,
	chain_id: u64,
	wallet_index: u32,
	loan_nonce: u64,
	min_expected_account_value: String,
) -> Result<PaybackLoanTestResponse, PumpxRpcError> {
	let smart_wallet_address_str = &skeleton_user_op.sender;

	// Parse min_expected_account_value
	let min_expected_account_value_f64 =
		min_expected_account_value.parse::<f64>().map_err(|e| {
			error!("Failed to parse min_expected_account_value: {}", e);
			PumpxRpcError::from(
				DetailedError::new(PARSE_ERROR_CODE, "Invalid parameter")
					.with_reason(format!("Invalid min_expected_account_value value: {}", e)),
			)
		})?;

	info!("Minimum expected account value threshold: {} USDC", min_expected_account_value_f64);

	// Initialize HyperCore client
	let hypercore_client = HyperCoreClient::new(chain_id).map_err(|e| {
		error!("Failed to create HyperCore client: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
				.with_reason(format!("Chain ID {} is not supported", chain_id)),
		)
	})?;

	// Print initial account state
	hypercore_client
		.print_account_state(smart_wallet_address_str, "Before Payback")
		.await;

	// Validate payback loan parameters and determine actions
	let validation_result = precheck(
		&ctx,
		&hypercore_client,
		&omni_account,
		smart_wallet_address_str,
		loan_nonce,
		min_expected_account_value_f64,
	)
	.await?;

	let collateral_ticker = &validation_result.collateral_ticker;
	let collateral_size = validation_result.collateral_size;
	let usdc_sold = validation_result.usdc_sold;
	let usdc_loaned = validation_result.usdc_loaned;
	let hedge_open_cloid = validation_result.hedge_open_cloid;
	let should_cancel = validation_result.should_cancel;
	let should_close = validation_result.should_close;
	let position_size_to_close = validation_result.position_size_to_close;
	let position_data = validation_result.position_data;

	// Fetch metadata and market prices from HyperCore in one call each
	// Get spot market prices (markPx and midPx) along with spot metadata
	let (spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(&collateral_ticker).await.map_err(|e| {
			error!("Failed to get spot market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to get spot market prices for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	// Get perp market prices (markPx and midPx) along with perp metadata
	let (meta, perp_mark_price, perp_mid_price) =
		hypercore_client.get_perp_market_prices(&collateral_ticker).await.map_err(|e| {
			error!("Failed to get perp market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to get perp market prices for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	// Get asset IDs
	let spot_asset_id = get_spot_asset_id(&collateral_ticker, &spot_meta).map_err(|e| {
		error!("Failed to get spot asset ID: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	let perp_asset_id = get_perp_asset_id(&collateral_ticker, &meta).map_err(|e| {
		error!("Failed to get perp asset ID: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	// Get token metadata
	let collateral_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(&collateral_ticker))
		.ok_or_else(|| {
			error!("Token {} not found in spot meta", collateral_ticker);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Token {} not found in spot meta", collateral_ticker)),
			)
		})?;

	let perp_asset = meta.universe.get(perp_asset_id as usize).ok_or_else(|| {
		error!("Perp asset {} not found in meta", perp_asset_id);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Perp asset {} not found", perp_asset_id)),
		)
	})?;

	info!(
		"Market prices for {} - spot markPx: {} USDC, spot midPx: {} USDC, perp markPx: {} USDC, perp midPx: {} USDC",
		collateral_ticker, spot_mark_price, spot_mid_price, perp_mark_price, perp_mid_price
	);

	// Track cloids and tx hashes
	let mut current_nonce = skeleton_user_op.nonce;
	let mut hedge_cancel_tx_hash = None;
	let mut hedge_close_cloid_opt = None;
	let mut hedge_close_tx_hash = None;

	// Action 1a: Cancel order if needed
	if should_cancel {
		info!("Action 1a: Canceling unfilled order...");

		let cancel_action = build_cancel_order_by_cloid(perp_asset_id, hedge_open_cloid);
		let cancel_calldata = encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(cancel_action),
		);

		let cancel_tx_hash = submit_corewriter_userop(
			ctx.clone(),
			&omni_account,
			&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
			chain_id,
			wallet_index,
			cancel_calldata,
			"",
		)
		.await?;

		info!("Action 1a: Cancel order submitted with tx_hash: {:?}", cancel_tx_hash);
		hedge_cancel_tx_hash = cancel_tx_hash;
		current_nonce += 1;

		// Wait for the order to be canceled
		let order_canceled = hypercore_client
			.wait_for_order(
				smart_wallet_address_str,
				&hedge_open_cloid.to_string(),
				30,
				OrderWaitCondition::Canceled,
			)
			.await
			.map_err(|e| {
				error!("Order cancel did not complete: {}", e);
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason(format!("Order cancel failed: {}", e)),
				)
			})?;

		if !order_canceled {
			error!("Order cancel was rejected or expired");
			return Err(PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Order cancel was rejected or expired"),
			));
		}

		info!("Action 1a: Order canceled successfully");

		hypercore_client
			.print_account_state(smart_wallet_address_str, "After Action 1a - Order Canceled")
			.await;
	}

	// Action 1b: Close position if needed
	if should_close {
		info!("Action 1b: Closing hedge position...");

		let hedge_close_cloid = generate_cloid();
		hedge_close_cloid_opt = Some(hedge_close_cloid.to_string());
		let close_size_abs = position_size_to_close.abs();
		let clamped_close_size = clamp_size(close_size_abs, perp_asset.sz_decimals);
		// For closing long position (selling), we want to sell at the highest buy price (bid)
		let (perp_bid_price, _perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
		let target_close_price = perp_bid_price * PERP_CLOSE_PRICE_RATIO;
		let clamped_close_price = clamp_price(target_close_price, perp_asset.sz_decimals, false);

		info!(
			"Perp close pricing - markPx: {}, midPx: {}, bid (highest buy): {}, target (with {}x buffer): {}, clamped: {}",
			perp_mark_price, perp_mid_price, perp_bid_price, PERP_CLOSE_PRICE_RATIO, target_close_price, clamped_close_price
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

		let close_action = build_perp_close_order(
			perp_asset_id,
			close_size_units,
			close_price_units,
			hedge_close_cloid,
		);
		let close_calldata = encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(close_action),
		);

		hedge_close_tx_hash = submit_corewriter_userop(
			ctx.clone(),
			&omni_account,
			&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
			chain_id,
			wallet_index,
			close_calldata,
			"",
		)
		.await?;

		info!(
			"Action 1b: Hedge close order submitted, price: {}, size: {}, tx_hash: {:?}",
			clamped_close_price_f64, clamped_close_size_f64, hedge_close_tx_hash
		);
		current_nonce += 1;

		let order_filled = hypercore_client
			.wait_for_order(
				smart_wallet_address_str,
				&hedge_close_cloid.to_string(),
				30,
				OrderWaitCondition::Filled,
			)
			.await
			.map_err(|e| {
				error!("Hedge close order did not complete: {}", e);
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason(format!("Hedge close order failed: {}", e)),
				)
			})?;

		if !order_filled {
			error!("Hedge close order was rejected or canceled");
			return Err(PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Hedge close order was rejected or canceled"),
			));
		}

		info!("Action 1b: Hedge position closed successfully");

		hypercore_client
			.print_account_state(smart_wallet_address_str, "After Action 1b - Hedge Closed")
			.await;
	}

	// Action 2: Transfer USDC from perp to spot
	// Always needed - either closing proceeds or unused margin from unfilled order
	info!("Action 2: Transferring USDC from perp to spot...");

	let perp_state_after_actions = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address_str)
		.await
		.map_err(|e| {
			error!("Failed to get perp state: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp state: {}", e)),
			)
		})?;

	let withdrawable_usdc = perp_state_after_actions.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable USDC: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse withdrawable USDC: {}", e)),
		)
	})?;

	info!("Current withdrawable USDC from perp: {}", withdrawable_usdc);

	// Calculate precise transfer amount
	// For position closed: initial_margin + unrealized_pnl - cum_funding - close_fee - buffer
	// For unfilled order: just initial_margin (what was deposited to perp)
	let initial_margin = usdc_sold - usdc_loaned;

	let transfer_amount = if let Some((
		unrealized_pnl,
		cum_funding_all_time,
		margin_used,
		position_value,
		stored_withdrawable,
	)) = position_data
	{
		// Position was closed - calculate precise amount

		// Calculate close position fee: 0.045% of notional value
		let close_position_fee = position_value * 0.00045;

		// Calculate precise transfer amount with small buffer (0.01 USDC) for rounding errors
		let buffer = 0.01;
		let calculated_amount =
			initial_margin + unrealized_pnl - cum_funding_all_time - close_position_fee - buffer;

		info!(
			"Transfer amount calculation: initial_margin={}, unrealized_pnl={}, cum_funding_all_time={}, close_fee={}, buffer={}, calculated={}",
			initial_margin, unrealized_pnl, cum_funding_all_time, close_position_fee, buffer, calculated_amount
		);

		// Validate: transfer amount must be <= withdrawable + margin_used
		// Note: margin_used from before closing, but after closing it becomes part of withdrawable
		let max_transferable = stored_withdrawable + margin_used;
		if calculated_amount > max_transferable {
			error!(
				"Calculated transfer amount ({}) exceeds max transferable ({} = {} withdrawable + {} margin_used)",
				calculated_amount, max_transferable, stored_withdrawable, margin_used
			);
			return Err(PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Invalid transfer amount").with_reason(
					format!(
						"Calculated transfer amount ({}) exceeds available funds ({})",
						calculated_amount, max_transferable
					),
				),
			));
		}

		// Also ensure we don't try to transfer more than what's actually withdrawable now
		let final_amount = calculated_amount.min(withdrawable_usdc);
		info!(
			"✓ Transfer amount validation passed: {} <= {} (max transferable), using {}",
			calculated_amount, max_transferable, final_amount
		);

		final_amount
	} else {
		// No position was closed (order was unfilled) - transfer just initial margin
		info!("No position closed, transferring initial margin: {}", initial_margin);
		initial_margin.min(withdrawable_usdc)
	};

	info!("Transferring {} USDC from perp to spot", transfer_amount);

	// Get initial spot balance BEFORE submitting the transfer
	let initial_spot_usdc = hypercore_client
		.get_spot_balance(smart_wallet_address_str, "USDC")
		.await
		.map_err(|e| {
			error!("Failed to get initial spot USDC balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query spot USDC balance: {}", e)),
			)
		})?;

	let transfer_amount_units = to_usdc_units(transfer_amount);
	let transfer_action = build_usd_class_transfer_to_spot(transfer_amount_units);
	let transfer_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(transfer_action),
	);

	let usd_transfer_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
		chain_id,
		wallet_index,
		transfer_calldata,
		"",
	)
	.await?;

	info!("Action 2: USD transfer submitted with tx_hash: {:?}", usd_transfer_tx_hash);
	current_nonce += 1;

	hypercore_client
		.wait_for_spot_balance_increase(
			smart_wallet_address_str,
			"USDC",
			initial_spot_usdc,
			transfer_amount,
			30,
		)
		.await
		.map_err(|e| {
			error!("USD transfer to spot failed: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Transfer timeout").with_reason(e),
			)
		})?;

	info!("Action 2: USDC transfer completed");

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 2 - USD Transfer to Spot")
		.await;

	// Action 3: Spot buy collateral
	info!("Action 3: Buying back collateral in spot market...");

	// Get current USDC balance to determine how much collateral we can afford
	let current_spot_usdc = hypercore_client
		.get_spot_balance(smart_wallet_address_str, "USDC")
		.await
		.map_err(|e| {
			error!("Failed to get current spot USDC balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query spot USDC balance: {}", e)),
			)
		})?;

	info!("Current spot USDC balance: {} USDC", current_spot_usdc);

	// Refresh spot prices before buying, as time could have elapsed since validation
	info!("Refreshing spot market prices before Action 3...");
	let (_spot_meta_refreshed, spot_mark_price_refreshed, spot_mid_price_refreshed) =
		hypercore_client.get_spot_market_prices(&collateral_ticker).await.map_err(|e| {
			error!("Failed to refresh spot market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to refresh spot market prices for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	let spot_buy_cloid = generate_cloid();
	let spot_buy_cloid_str = spot_buy_cloid.to_string();
	// For buying, we want to buy at the lowest sell price (ask)
	let (_spot_bid_price, spot_ask_price) =
		get_bid_ask_prices(spot_mark_price_refreshed, spot_mid_price_refreshed);
	let target_buy_price = spot_ask_price * SPOT_BUY_PRICE_RATIO;
	let clamped_buy_price = clamp_price(target_buy_price, collateral_token.sz_decimals, true);

	// Calculate affordable collateral: how much can we buy with available USDC?
	let clamped_buy_price_f64 = clamped_buy_price.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy price: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped buy price: {}", e)),
		)
	})?;

	let affordable_collateral = current_spot_usdc / clamped_buy_price_f64;

	// Use the minimum of what we want vs what we can afford
	let actual_buy_size = collateral_size.min(affordable_collateral);
	let clamped_buy_size = clamp_size(actual_buy_size, collateral_token.sz_decimals);

	info!(
		"Spot buy calculation - desired: {}, affordable: {}, using: {} (clamped: {})",
		collateral_size, affordable_collateral, actual_buy_size, clamped_buy_size
	);

	info!(
		"Spot buy pricing - markPx: {}, midPx: {}, ask (lowest sell): {}, target (with {}x buffer): {}, clamped: {}",
		spot_mark_price_refreshed, spot_mid_price_refreshed, spot_ask_price, SPOT_BUY_PRICE_RATIO, target_buy_price, clamped_buy_price
	);

	let clamped_buy_size_f64 = clamped_buy_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped buy size: {}", e)),
		)
	})?;
	let clamped_buy_price_f64 = clamped_buy_price.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy price: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped buy price: {}", e)),
		)
	})?;

	let buy_size_units = to_price_units(clamped_buy_size_f64);
	let buy_price_units = to_price_units(clamped_buy_price_f64);

	let spot_buy_action =
		build_spot_buy_order(spot_asset_id, buy_size_units, buy_price_units, spot_buy_cloid);
	let spot_buy_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(spot_buy_action),
	);

	let spot_buy_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
		chain_id,
		wallet_index,
		spot_buy_calldata,
		"",
	)
	.await?;

	info!(
		"Action 3: Spot buy order submitted, price: {}, size: {}, tx_hash: {:?}",
		clamped_buy_price_f64, clamped_buy_size_f64, spot_buy_tx_hash
	);

	let buy_order_filled = hypercore_client
		.wait_for_order(
			smart_wallet_address_str,
			&spot_buy_cloid.to_string(),
			30,
			OrderWaitCondition::Filled,
		)
		.await
		.map_err(|e| {
			error!("Spot buy order did not complete: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Spot buy order failed: {}", e)),
			)
		})?;

	if !buy_order_filled {
		error!("Spot buy order was rejected or canceled");
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason("Spot buy order was rejected or canceled"),
		));
	}

	info!("Action 3: Spot buy order filled successfully");

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 3 - Spot Buy Complete")
		.await;

	Ok(PaybackLoanTestResponse {
		collateral_ticker: collateral_ticker.to_string(),
		collateral_size: collateral_size.to_string(),
		hedge_cancel_tx_hash,
		hedge_close_cloid: hedge_close_cloid_opt,
		hedge_close_tx_hash,
		usd_transfer_tx_hash,
		spot_buy_cloid: Some(spot_buy_cloid_str),
		spot_buy_tx_hash,
	})
}

/// Helper to verify hedge position exists and is not liquidated
/// Returns: (position_size, account_value, unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable)
/// where account_value = crossMarginSummary.accountValue
async fn verify_hedge_position(
	hypercore_client: &HyperCoreClient,
	smart_wallet_address: &str,
	collateral_ticker: &str,
) -> Result<(f64, f64, f64, f64, f64, f64, f64), PumpxRpcError> {
	let perp_state = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address)
		.await
		.map_err(|e| {
			error!("Failed to get perp state: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp state: {}", e)),
			)
		})?;

	let hedge_position = perp_state
		.asset_positions
		.iter()
		.find(|pos| pos.position.coin.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_else(|| {
			error!("Hedge position for {} not found", collateral_ticker);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Position not found").with_reason(format!(
					"Position for {} not found or liquidated",
					collateral_ticker
				)),
			)
		})?;

	let position_size = hedge_position.position.szi.parse::<f64>().map_err(|e| {
		error!("Failed to parse position size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid position size: {}", e)),
		)
	})?;

	if position_size <= 0.0 {
		error!("Position liquidated (size: {})", position_size);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Position liquidated")
				.with_reason(format!("Position for {} has been liquidated", collateral_ticker)),
		));
	}

	// Get account value from crossMarginSummary
	let account_value =
		perp_state.cross_margin_summary.account_value.parse::<f64>().map_err(|e| {
			error!("Failed to parse account value: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Invalid account value: {}", e)),
			)
		})?;

	// Parse additional position data
	let unrealized_pnl = hedge_position.position.unrealized_pnl.parse::<f64>().map_err(|e| {
		error!("Failed to parse unrealized PnL: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid unrealized PnL: {}", e)),
		)
	})?;

	let cum_funding_all_time =
		hedge_position.position.cum_funding.all_time.parse::<f64>().map_err(|e| {
			error!("Failed to parse cumulative funding: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Invalid cumulative funding: {}", e)),
			)
		})?;

	let margin_used = hedge_position.position.margin_used.parse::<f64>().map_err(|e| {
		error!("Failed to parse margin used: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid margin used: {}", e)),
		)
	})?;

	let position_value = hedge_position.position.position_value.parse::<f64>().map_err(|e| {
		error!("Failed to parse position value: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid position value: {}", e)),
		)
	})?;

	let withdrawable = perp_state.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid withdrawable: {}", e)),
		)
	})?;

	info!(
		"Position data for {}: size={}, account_value={}, unrealized_pnl={}, cum_funding_all_time={}, margin_used={}, position_value={}, withdrawable={}",
		collateral_ticker, position_size, account_value, unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable
	);

	Ok((
		position_size,
		account_value,
		unrealized_pnl,
		cum_funding_all_time,
		margin_used,
		position_value,
		withdrawable,
	))
}

/// Performs all validation checks for payback loan, including retrieving loan record and determining actions
async fn precheck<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	ctx: &RpcContext<CrossChainIntentExecutor>,
	hypercore_client: &HyperCoreClient,
	omni_account: &AccountId,
	smart_wallet_address: &str,
	loan_nonce: u64,
	min_expected_account_value_f64: f64,
) -> Result<PaybackLoanValidationResult, PumpxRpcError> {
	// Step 1: Retrieve loan record from storage
	info!("Retrieving loan record for omni_account {:?}, nonce {}", omni_account, loan_nonce);

	let storage_key =
		executor_storage::loan_record::Key { account_id: omni_account.clone(), nonce: loan_nonce };

	let loan_record = ctx
		.loan_record_storage
		.get(&storage_key)
		.map_err(|_| {
			error!("Failed to retrieve loan record from storage");
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Failed to retrieve loan record from storage"),
			)
		})?
		.ok_or_else(|| {
			error!("Loan record not found for nonce {}", loan_nonce);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Loan record not found")
					.with_reason(format!("No loan record found for nonce {}", loan_nonce)),
			)
		})?;

	info!("Retrieved loan record: {:?}", loan_record);

	let collateral_ticker = loan_record.collateral_ticker.to_uppercase();
	let collateral_size = loan_record.collateral_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse collateral_size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid collateral_size in loan record: {}", e)),
		)
	})?;

	let usdc_sold = loan_record.usdc_sold.parse::<f64>().map_err(|e| {
		error!("Failed to parse usdc_sold: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid usdc_sold in loan record: {}", e)),
		)
	})?;

	let usdc_loaned = loan_record.usdc_loaned.parse::<f64>().map_err(|e| {
		error!("Failed to parse usdc_loaned: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid usdc_loaned in loan record: {}", e)),
		)
	})?;

	let hedge_open_cloid = loan_record.hedge_open_cloid.parse::<u128>().map_err(|e| {
		error!("Failed to parse hedge_open_cloid: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid hedge_open_cloid in loan record: {}", e)),
		)
	})?;

	// Step 2: Check USDC balance in spot account
	info!("Checking USDC balance in spot account...");
	let usdc_balance = hypercore_client
		.get_spot_balance(smart_wallet_address, "USDC")
		.await
		.map_err(|e| {
			error!("Failed to get USDC balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query USDC balance: {}", e)),
			)
		})?;

	info!("User USDC balance: {}, loan amount: {}", usdc_balance, usdc_loaned);

	if usdc_balance < usdc_loaned {
		error!(
			"Insufficient USDC balance: user has {} but needs {} to pay back loan",
			usdc_balance, usdc_loaned
		);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Insufficient USDC balance").with_reason(
				format!(
					"User has {} USDC but needs {} USDC to pay back the loan",
					usdc_balance, usdc_loaned
				),
			),
		));
	}

	info!("✓ USDC balance check passed: user has sufficient USDC");

	// Step 3: Check hedge order status
	info!(
		"Checking hedge order status for loan nonce {} with hedge_open_cloid {}",
		loan_nonce, hedge_open_cloid
	);

	let order_status = hypercore_client
		.get_order_status(smart_wallet_address, &hedge_open_cloid.to_string())
		.await
		.map_err(|e| {
			error!("Failed to get order status for cloid {}: {}", hedge_open_cloid, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Failed to get order status").with_reason(
					format!("Cannot retrieve order status for cloid {}: {}", hedge_open_cloid, e),
				),
			)
		})?;

	// Store position data for transfer amount calculation
	let mut position_data: Option<(f64, f64, f64, f64, f64)> = None; // (unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable)

	let (should_cancel, should_close, position_size_to_close) = if let Some(order_info) =
		order_status.order
	{
		let status = order_info.status.as_str();
		info!("Hedge order status: {}", status);

		match status {
			"filled" => {
				// Case 1: Fully filled - close position
				info!("Case 1: Order fully filled, will close position");
				let (
					position_size,
					account_value,
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				) = verify_hedge_position(hypercore_client, smart_wallet_address, &collateral_ticker)
					.await?;

				// Validate account value against minimum threshold
				if account_value < min_expected_account_value_f64 {
					error!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					return Err(PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Account value too low").with_reason(
								format!(
									"Account value ({} USDC) is below minimum expected ({} USDC). Refusing to close position with unexpected loss.",
									account_value, min_expected_account_value_f64
								),
							),
						));
				}
				info!(
					"✓ Account value check passed: {} USDC >= {} USDC",
					account_value, min_expected_account_value_f64
				);

				// Store position data for later transfer calculation
				position_data = Some((
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				));

				(false, true, position_size)
			},
			"open" => {
				// Case 2: Not filled at all - cancel order
				info!("Case 2: Order not filled, will cancel");
				(true, false, 0.0)
			},
			"partial_fill" => {
				// Case 3: Partially filled - cancel + close filled part
				info!(
					"Case 3: Order partially filled, will cancel order and close filled position"
				);
				let (
					position_size,
					account_value,
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				) = verify_hedge_position(hypercore_client, smart_wallet_address, &collateral_ticker)
					.await?;

				// Validate account value against minimum threshold
				if account_value < min_expected_account_value_f64 {
					error!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					return Err(PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Account value too low").with_reason(
								format!(
									"Account value ({} USDC) is below minimum expected ({} USDC). Refusing to close position with unexpected loss.",
									account_value, min_expected_account_value_f64
								),
							),
						));
				}
				info!(
					"✓ Account value check passed: {} USDC >= {} USDC",
					account_value, min_expected_account_value_f64
				);

				// Store position data for later transfer calculation
				position_data = Some((
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				));

				(true, true, position_size)
			},
			"canceled" | "rejected" | "expired" => {
				error!("Order already in terminal state: {}", status);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Order not active")
						.with_reason(format!("Order is already {}", status)),
				));
			},
			_ => {
				error!("Unexpected order status: {}", status);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Unexpected order status")
						.with_reason(format!("Unknown status: {}", status)),
				));
			},
		}
	} else {
		error!("Order not found for cloid {}", hedge_open_cloid);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Order not found")
				.with_reason(format!("No order found with cloid {}", hedge_open_cloid)),
		));
	};

	Ok(PaybackLoanValidationResult {
		collateral_ticker,
		collateral_size,
		usdc_sold,
		usdc_loaned,
		hedge_open_cloid,
		should_cancel,
		should_close,
		position_size_to_close,
		position_data,
	})
}
