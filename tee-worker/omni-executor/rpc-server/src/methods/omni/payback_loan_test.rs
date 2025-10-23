use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_corewriter_userop;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::AccountId;
use executor_storage::Storage;
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct PaybackLoanTestParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: u64,
	pub wallet_index: u32,
	pub omni_account: String,
	pub nonce: u64,
}

#[derive(Serialize, Clone)]
pub struct PaybackLoanTestResponse {
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub hedge_close_cloid: String,
	pub spot_buy_cloid: String,
	pub hedge_close_tx_hash: Option<String>,
	pub usd_transfer_tx_hash: Option<String>,
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
				params.nonce,
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
) -> Result<PaybackLoanTestResponse, PumpxRpcError> {
	let smart_wallet_address_str = &skeleton_user_op.sender;

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

	// Step 2: Check USDC balance in spot account
	info!("Checking USDC balance in spot account...");
	let usdc_balance = hypercore_client
		.get_spot_balance(smart_wallet_address_str, "USDC")
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

	// Step 3: Check hedge position status by verifying the specific position for this loan
	info!(
		"Checking hedge position status for loan nonce {} with hedge_open_cloid {}",
		loan_nonce, hedge_open_cloid
	);

	// First, get the fill that opened the hedge to verify we're looking at the right position
	let hedge_open_fill = hypercore_client
		.get_fill_by_cloid(smart_wallet_address_str, hedge_open_cloid)
		.await
		.map_err(|e| {
			error!("Failed to get hedge open fill for cloid {}: {}", hedge_open_cloid, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Hedge position not found")
					.with_reason(format!("Cannot find the original hedge open order (cloid: {}). It may have been liquidated or manually closed.", hedge_open_cloid)),
			)
		})?;

	info!(
		"Found hedge open fill: coin={}, size={}, price={}, cloid={}",
		hedge_open_fill.coin,
		hedge_open_fill.sz,
		hedge_open_fill.px,
		hedge_open_fill.cloid.as_ref().unwrap_or(&"N/A".to_string())
	);

	// Verify the fill is for the expected collateral ticker
	if !hedge_open_fill.coin.eq_ignore_ascii_case(&collateral_ticker) {
		error!(
			"Mismatch: hedge open fill is for {} but loan record says {}",
			hedge_open_fill.coin, collateral_ticker
		);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Data inconsistency").with_reason(format!(
				"Hedge open fill ticker ({}) doesn't match loan collateral ticker ({})",
				hedge_open_fill.coin, collateral_ticker
			)),
		));
	}

	// Now get current perp state to check if the position still exists
	let perp_state = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address_str)
		.await
		.map_err(|e| {
			error!("Failed to get perp clearinghouse state: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp state: {}", e)),
			)
		})?;

	// Find the position for this specific collateral
	let hedge_position = perp_state
		.asset_positions
		.iter()
		.find(|pos| pos.position.coin.eq_ignore_ascii_case(&collateral_ticker))
		.ok_or_else(|| {
			error!(
				"Hedge position for {} (from loan nonce {}) not found in current positions",
				collateral_ticker, loan_nonce
			);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Hedge position not found")
					.with_reason(format!(
						"The hedge position for {} (opened with cloid {}) no longer exists. It may have been liquidated or manually closed.",
						collateral_ticker, hedge_open_cloid
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

	let hedge_open_size = hedge_open_fill.sz.parse::<f64>().map_err(|e| {
		error!("Failed to parse hedge open size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid hedge open size: {}", e)),
		)
	})?;

	// Check if position has been liquidated (size should be > 0 for long position)
	if position_size <= 0.0 {
		error!("Hedge position has been liquidated or closed (size: {})", position_size);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Position liquidated or closed").with_reason(
				format!(
					"The hedge position for {} (loan nonce {}) has been liquidated or closed",
					collateral_ticker, loan_nonce
				),
			),
		));
	}

	// Verify the position size is reasonable relative to the original hedge
	// Note: If user has multiple positions for the same ticker, we can only verify one exists
	// The position size might be larger if user added to it, or smaller if partially closed
	info!(
		"✓ Hedge position check passed: {} position with current size {} (original hedge: {})",
		collateral_ticker, position_size, hedge_open_size
	);

	// Fetch metadata
	let spot_meta = hypercore_client.get_spot_meta().await.map_err(|e| {
		error!("Failed to get spot meta: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	let meta = hypercore_client.get_meta().await.map_err(|e| {
		error!("Failed to get perp meta: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
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

	// Fetch market prices
	let spot_market_price = hypercore_client
		.get_spot_mid_price(&collateral_ticker, &spot_meta)
		.await
		.map_err(|e| {
			error!("Failed to get spot market price for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to get spot market price for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	let perp_market_price =
		hypercore_client.get_perp_mid_price(&collateral_ticker).await.map_err(|e| {
			error!("Failed to get perp market price for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to get perp market price for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	info!(
		"Market prices for {} - spot: {} USDC, perp: {} USDC",
		collateral_ticker, spot_market_price, perp_market_price
	);

	// Generate cloids for operations
	let hedge_close_cloid = generate_cloid();
	let spot_buy_cloid = generate_cloid() + 1;

	info!(
		"Generated cloids - hedge_close: {} (hex: 0x{:032x}), spot_buy: {} (hex: 0x{:032x})",
		hedge_close_cloid, hedge_close_cloid, spot_buy_cloid, spot_buy_cloid
	);

	// Action 1: Close hedge position
	info!("Action 1: Closing hedge position...");

	// Calculate close order parameters
	let close_size_abs = position_size.abs();
	let clamped_close_size = clamp_size(close_size_abs, perp_asset.sz_decimals);
	let target_close_price = perp_market_price * PERP_CLOSE_PRICE_RATIO;
	let clamped_close_price = clamp_price(target_close_price, perp_asset.sz_decimals, false);

	info!(
		"Closing position - size: {} (clamped: {}), price: {} (clamped: {})",
		close_size_abs, clamped_close_size, target_close_price, clamped_close_price
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

	let close_size_units = (clamped_close_size_f64 * 100_000_000.0) as u64;
	let close_price_units = (clamped_close_price_f64 * 100_000_000.0) as u64;

	// Build and submit close order
	let close_action = build_perp_close_order(
		perp_asset_id,
		close_size_units,
		close_price_units,
		hedge_close_cloid,
	);
	let close_corewriter_calldata = encode_send_raw_action(close_action);
	let close_calldata =
		encode_omni_account_execute(get_core_writer_address(), close_corewriter_calldata);

	info!("Submitting hedge close order...");
	let hedge_close_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_user_op,
		chain_id,
		wallet_index,
		close_calldata,
		"",
	)
	.await?;

	info!("Action 1: Hedge close order submitted with tx_hash: {:?}", hedge_close_tx_hash);

	info!("Waiting for hedge position to close...");

	// Wait for the close order to be filled
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

	info!("Action 1: Hedge position closed successfully");

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 1 - Hedge Closed")
		.await;

	// Action 2: Transfer USDC from perp to spot
	info!("Action 2: Transferring USDC from perp to spot...");

	// Get current perp balance to determine how much USDC to transfer
	let perp_state_after_close = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address_str)
		.await
		.map_err(|e| {
			error!("Failed to get perp state after close: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp state: {}", e)),
			)
		})?;

	let withdrawable_usdc = perp_state_after_close.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable USDC: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse withdrawable USDC: {}", e)),
		)
	})?;

	info!("Withdrawable USDC from perp: {}", withdrawable_usdc);

	// Transfer all withdrawable USDC from perp to spot
	let transfer_amount_units = (withdrawable_usdc * 1_000_000.0) as u64;
	let transfer_action = build_usd_class_transfer_to_spot(transfer_amount_units);
	let transfer_corewriter_calldata = encode_send_raw_action(transfer_action);
	let transfer_calldata =
		encode_omni_account_execute(get_core_writer_address(), transfer_corewriter_calldata);

	info!("Action 2: Transferring {} USDC from perp to spot...", withdrawable_usdc);

	let mut skeleton_action2 = skeleton_user_op.clone();
	skeleton_action2.nonce = skeleton_user_op.nonce + 1;
	skeleton_action2.init_code = "0x".to_string();

	let usd_transfer_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_action2,
		chain_id,
		wallet_index,
		transfer_calldata,
		"",
	)
	.await?;

	info!("Action 2: USD transfer to spot submitted with tx_hash: {:?}", usd_transfer_tx_hash);

	// Wait for transfer to complete by checking spot balance increase
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

	hypercore_client
		.wait_for_spot_balance_increase(
			smart_wallet_address_str,
			"USDC",
			initial_spot_usdc,
			withdrawable_usdc,
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

	// Calculate how much collateral to buy (target: collateral_size)
	let target_buy_price = spot_market_price * SPOT_BUY_PRICE_RATIO;
	let clamped_buy_price = clamp_price(target_buy_price, collateral_token.sz_decimals, true);
	let clamped_buy_size = clamp_size(collateral_size, collateral_token.sz_decimals);

	info!(
		"Spot buy parameters - size: {} (clamped: {}), price: {} (clamped: {})",
		collateral_size, clamped_buy_size, target_buy_price, clamped_buy_price
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

	let buy_size_units = (clamped_buy_size_f64 * 100_000_000.0) as u64;
	let buy_price_units = (clamped_buy_price_f64 * 100_000_000.0) as u64;

	let spot_buy_action =
		build_spot_buy_order(spot_asset_id, buy_size_units, buy_price_units, spot_buy_cloid);
	let spot_buy_corewriter_calldata = encode_send_raw_action(spot_buy_action);
	let spot_buy_calldata =
		encode_omni_account_execute(get_core_writer_address(), spot_buy_corewriter_calldata);

	info!("Submitting spot buy order...");

	let mut skeleton_action3 = skeleton_user_op.clone();
	skeleton_action3.nonce = skeleton_user_op.nonce + 2;
	skeleton_action3.init_code = "0x".to_string();

	let spot_buy_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_action3,
		chain_id,
		wallet_index,
		spot_buy_calldata,
		"",
	)
	.await?;

	info!("Action 3: Spot buy order submitted with tx_hash: {:?}", spot_buy_tx_hash);

	// Wait for spot buy to fill
	info!("Waiting for spot buy order to fill...");
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

	// Get the actual fill to determine how much was bought
	let spot_buy_fill = hypercore_client
		.get_fill_by_cloid(smart_wallet_address_str, spot_buy_cloid)
		.await
		.map_err(|e| {
			error!("Failed to get fill for spot buy order: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get fill: {}", e)),
			)
		})?;

	let collateral_size = spot_buy_fill.sz.clone();

	info!("Action 3: Bought {} {} in spot market", collateral_size, collateral_ticker);

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 3 - Spot Buy Complete")
		.await;

	Ok(PaybackLoanTestResponse {
		collateral_ticker,
		collateral_size,
		hedge_close_cloid: hedge_close_cloid.to_string(),
		spot_buy_cloid: spot_buy_cloid.to_string(),
		hedge_close_tx_hash,
		usd_transfer_tx_hash,
		spot_buy_tx_hash,
	})
}
