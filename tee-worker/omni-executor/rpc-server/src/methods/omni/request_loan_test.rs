use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::{prepare_skeleton_with_nonce, submit_corewriter_userop};
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use executor_storage::{LoanRecord, Storage};
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Result of loan request validation, containing metadata needed for order construction
struct LoanRequestValidationResult {
	spot_asset_id: u32,
	perp_asset_id: u32,
	spot_sz_decimals: u8,
	perp_sz_decimals: u8,
	max_leverage: u32,
	lending_ratio_f64: f64,
}

#[derive(Debug, Deserialize)]
pub struct RequestLoanTestParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub client_id: String,
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub lending_ratio: u32,
}

#[derive(Serialize, Clone)]
pub struct RequestLoanTestResponse {
	pub spot_sell_cloid: String,
	pub hedge_open_cloid: String,
	pub usdc_received: String,
	pub spot_sell_tx_hash: Option<String>,
	pub hedge_open_tx_hash: Option<String>,
}

pub fn register_request_loan_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_requestLoanTest", |params, ctx, _ext| async move {
			let params = params.parse::<RequestLoanTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_requestLoanTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
				error!("Failed to parse omni account");
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Failed to parse omni account"),
				)
			})?;

			// Validate sender address
			params.user_operation.sender.parse::<Address>().map_err(|e| {
				error!("Invalid sender address '{}': {}", params.user_operation.sender, e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_field("sender")
						.with_reason(format!(
							"Invalid sender address '{}': {}",
							params.user_operation.sender, e
						)),
				)
			})?;

			// Validate collateral ticker is not empty and normalize to uppercase
			if params.collateral_ticker.is_empty() {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Collateral ticker cannot be empty")
						.with_field("collateral_ticker")
						.with_suggestion("Provide a valid ticker like ETH, PURR, etc."),
				));
			}
			let collateral_ticker = params.collateral_ticker.to_uppercase();

			// Validate collateral_size is not empty
			if params.collateral_size.is_empty() {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Collateral size cannot be empty")
						.with_field("collateral_size")
						.with_suggestion(
							"Provide a valid collateral size in human-readable format",
						),
				));
			}

			// Validate lending_ratio is between 0 and 100 (0-100%)
			if params.lending_ratio > 100 {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Lending ratio must be between 0 and 100")
						.with_field("lending_ratio")
						.with_received(params.lending_ratio.to_string())
						.with_expected("0-100 (percentage)"),
				));
			}

			// Call the inlined handler logic
			handle_request_loan_impl(
				Arc::clone(&ctx),
				omni_account,
				params.user_operation.clone(),
				params.chain_id,
				params.wallet_index,
				&collateral_ticker,
				&params.collateral_size,
				params.lending_ratio,
				&params.client_id,
			)
			.await
		})
		.expect("Failed to register omni_requestLoanTest method");
}

// Main handler implementation
#[allow(clippy::too_many_arguments)]
async fn handle_request_loan_impl<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	omni_account: AccountId,
	skeleton_user_op: SerializablePackedUserOperation,
	chain_id: u64,
	wallet_index: u32,
	collateral_ticker: &str,
	collateral_size_str: &str,
	lending_ratio: u32,
	client_id: &str,
) -> Result<RequestLoanTestResponse, PumpxRpcError> {
	let smart_wallet_address_str = &skeleton_user_op.sender;

	let hypercore_client = HyperCoreClient::new(chain_id).map_err(|e| {
		error!("Failed to create HyperCore client: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
				.with_reason(format!("Chain ID {} is not supported", chain_id)),
		)
	})?;

	let collateral_size = collateral_size_str.parse::<f64>().map_err(|e| {
		error!("Failed to parse collateral_size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid collateral_size: {}", e)),
		)
	})?;

	// Validate loan request parameters (including balance, size, etc.)
	let validation_result = precheck(
		&hypercore_client,
		smart_wallet_address_str,
		collateral_ticker,
		collateral_size,
		lending_ratio,
	)
	.await?;

	let lending_ratio_f64 = validation_result.lending_ratio_f64;

	// Print initial account state
	hypercore_client
		.print_account_state(smart_wallet_address_str, "Before Actions")
		.await;

	// Action 1: Sell collateral_size as spot to get X USDC
	// Use prices from validation - precheck is fast enough that we don't need to refresh
	let (_spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get spot market prices: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get spot prices: {}", e)),
			)
		})?;

	let clamped_size = clamp_size(collateral_size, validation_result.spot_sz_decimals);
	// For selling, we want to sell at the highest buy price (bid)
	let (spot_bid_price, _spot_ask_price) = get_bid_ask_prices(spot_mark_price, spot_mid_price);
	let target_price = spot_bid_price * SPOT_SELL_PRICE_RATIO;
	let clamped_price = clamp_price(target_price, validation_result.spot_sz_decimals, true);

	info!(
		"Spot sell pricing - markPx: {}, midPx: {}, bid (highest buy): {}, target (with {}x buffer): {}",
		spot_mark_price, spot_mid_price, spot_bid_price, SPOT_SELL_PRICE_RATIO, target_price
	);

	let clamped_size_f64 = clamped_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped size: {}", e)),
		)
	})?;
	let clamped_price_f64 = clamped_price.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped price: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped price: {}", e)),
		)
	})?;

	let spot_sell_size_units = to_price_units(clamped_size_f64);
	let spot_sell_price_units = to_price_units(clamped_price_f64);

	let spot_sell_cloid = generate_cloid();
	let hedge_open_cloid = generate_cloid() + 1;

	info!(
		"Generated cloids - spot_sell: {} (hex: 0x{:032x}), hedge_open: {} (hex: 0x{:032x})",
		spot_sell_cloid, spot_sell_cloid, hedge_open_cloid, hedge_open_cloid
	);

	// Build and submit spot sell action
	let spot_sell_action = build_spot_sell_order(
		validation_result.spot_asset_id,
		spot_sell_size_units,
		spot_sell_price_units,
		spot_sell_cloid,
	);
	let spot_sell_corewriter_calldata = encode_send_raw_action(spot_sell_action);
	let spot_sell_calldata =
		encode_omni_account_execute(get_core_writer_address(), spot_sell_corewriter_calldata);

	let spot_sell_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_user_op,
		chain_id,
		wallet_index,
		spot_sell_calldata,
		client_id,
	)
	.await?;

	info!(
		"Action 1: Spot sell submitted, price: {}, size: {}, tx_hash: {:?}",
		clamped_price_f64, clamped_size_f64, spot_sell_tx_hash
	);

	// Wait for order to be filled
	info!("Polling HyperCore API for spot sell order completion (cloid: {})...", spot_sell_cloid);
	let order_filled = hypercore_client
		.wait_for_order(
			smart_wallet_address_str,
			&spot_sell_cloid.to_string(),
			20,
			OrderWaitCondition::Filled,
		)
		.await
		.map_err(|e| {
			error!("Spot sell order did not complete: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Spot sell order failed: {}", e)),
			)
		})?;

	if !order_filled {
		error!("Spot sell order was rejected or canceled");
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason("Spot sell order was rejected or canceled"),
		));
	}

	info!("Action 1: Spot sell order filled successfully");

	// Get the actual fill
	let spot_sell_fill = hypercore_client
		.get_fill_by_cloid(smart_wallet_address_str, spot_sell_cloid)
		.await
		.map_err(|e| {
			error!("Failed to get fill for spot sell order: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get fill: {}", e)),
			)
		})?;

	let usdc_received = calculate_usdc_received_from_spot_sell(&spot_sell_fill).map_err(|e| {
		error!("Failed to calculate USDC received: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to calculate USDC: {}", e)),
		)
	})?;

	info!("Action 1: Spot sell completed - received {:.2} USDC", usdc_received);

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 1 - Spot Sell")
		.await;

	// Calculate USDC allocation
	let usdc_for_perp = usdc_received * (1.0 - lending_ratio_f64);
	let usdc_to_lend = usdc_received * lending_ratio_f64;

	info!(
		"USDC allocation: total_received={:.2}, for_perp={:.2}, to_lend={:.2}",
		usdc_received, usdc_for_perp, usdc_to_lend
	);

	// Store loan record in storage (position_size will be updated after Action 3)
	let loan_record = LoanRecord {
		collateral_ticker: collateral_ticker.to_string(),
		collateral_size: collateral_size_str.to_string(),
		usdc_sold: format!("{:.2}", usdc_received),
		usdc_loaned: format!("{:.2}", usdc_to_lend),
		spot_sell_cloid: spot_sell_cloid.to_string(),
		hedge_open_cloid: hedge_open_cloid.to_string(),
		position_size: "0".to_string(), // Will be updated after hedge order completes
	};

	let storage_key = executor_storage::loan_record::Key {
		account_id: omni_account.clone(),
		nonce: skeleton_user_op.nonce as u64,
	};

	if ctx.loan_record_storage.insert(&storage_key, loan_record.clone()).is_err() {
		error!(
			"Failed to store loan record for omni_account {:?}, nonce {}",
			omni_account, skeleton_user_op.nonce
		);
		// Don't fail the entire operation, just log the error
	} else {
		info!(
			"Stored loan record for omni_account {:?}, nonce {}",
			omni_account, skeleton_user_op.nonce
		);
	}

	// Action 2: Move USDC into perps
	let mut current_nonce = skeleton_user_op.nonce + 1;

	// Get initial perp balance BEFORE submitting the transfer
	let initial_perp_balance = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address_str)
		.await
		.map_err(|e| {
			error!("Failed to get initial perp balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp balance: {}", e)),
			)
		})?
		.cross_margin_summary
		.account_value
		.parse::<f64>()
		.map_err(|e| {
			error!("Failed to parse initial perp balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to parse perp balance: {}", e)),
			)
		})?;

	let usdc_for_perp_units = to_usdc_units(usdc_for_perp);
	let usd_transfer_action = build_usd_class_transfer_to_perp(usdc_for_perp_units);
	let usd_transfer_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(usd_transfer_action),
	);

	let usd_transfer_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
		chain_id,
		wallet_index,
		usd_transfer_calldata,
		client_id,
	)
	.await?;

	info!(
		"Action 2: USD class transfer submitted, size: {}, tx_hash: {:?}",
		usdc_for_perp, usd_transfer_tx_hash
	);

	// Wait for USD transfer to complete
	let _actual_perp_balance = hypercore_client
		.wait_for_perp_balance_increase(
			smart_wallet_address_str,
			initial_perp_balance,
			usdc_for_perp,
			20,
		)
		.await
		.map_err(|e| {
			error!("USD transfer to perp did not complete: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("USD transfer failed: {}", e)),
			)
		})?;

	info!("Action 2: USD class transfer completed successfully");

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 2 - USD Transfer to Perp")
		.await;

	// Action 3: Open hedge position
	current_nonce += 1;

	// Refresh perp market prices to get latest prices for order construction
	info!("Refreshing perp market prices before Action 3...");
	let (_perp_meta, perp_mark_price, perp_mid_price) =
		hypercore_client.get_perp_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to refresh perp market prices: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to refresh perp prices: {}", e)),
			)
		})?;

	let desired_leverage: f64 = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(validation_result.max_leverage as f64);
	// For opening long position (buying), we want to buy at the lowest sell price (ask)
	let (_perp_bid_price, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let target_hedge_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
	let hedge_size = (usdc_for_perp * effective_leverage) / target_hedge_price;

	info!(
		"Perp hedge open pricing - markPx: {}, midPx: {}, ask (lowest sell): {}, target (with {}x buffer): {}, hedge_size: {}",
		perp_mark_price, perp_mid_price, perp_ask_price, PERP_ENTRY_PRICE_RATIO, target_hedge_price, hedge_size
	);

	let clamped_hedge_size = clamp_size(hedge_size, validation_result.perp_sz_decimals);
	let clamped_hedge_price =
		clamp_price(target_hedge_price, validation_result.perp_sz_decimals, false);

	let clamped_hedge_size_f64 = clamped_hedge_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped hedge size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped hedge size: {}", e)),
		)
	})?;
	let clamped_hedge_price_f64 = clamped_hedge_price.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped hedge price: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped hedge price: {}", e)),
		)
	})?;

	let hedge_size_units = to_price_units(clamped_hedge_size_f64);
	let hedge_price_units = to_price_units(clamped_hedge_price_f64);

	let hedge_action = build_perp_long_order(
		validation_result.perp_asset_id,
		hedge_size_units,
		hedge_price_units,
		hedge_open_cloid,
	);
	let hedge_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(hedge_action),
	);

	let hedge_open_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&prepare_skeleton_with_nonce(&skeleton_user_op, current_nonce),
		chain_id,
		wallet_index,
		hedge_calldata,
		client_id,
	)
	.await?;

	info!(
		"Action 3: Hedge position submitted, price: {}, size: {}, tx_hash: {:?}",
		clamped_hedge_price_f64, clamped_hedge_size_f64, hedge_open_tx_hash
	);

	// Wait for hedge order to be opened
	info!("Polling HyperCore API to verify hedge order is opened (cloid: {})...", hedge_open_cloid);
	let order_opened = hypercore_client
		.wait_for_order(
			smart_wallet_address_str,
			&hedge_open_cloid.to_string(),
			20,
			OrderWaitCondition::Opened,
		)
		.await
		.map_err(|e| {
			error!("Hedge order was not successfully opened: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Hedge order failed to open: {}", e)),
			)
		})?;

	if !order_opened {
		error!("Hedge order was rejected or canceled");
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason("Hedge order was rejected or canceled"),
		));
	}

	info!("Action 3: Hedge order successfully opened");

	hypercore_client
		.print_account_state(smart_wallet_address_str, "After Action 3 - Hedge Open")
		.await;

	// Get the actual position size and update the loan record
	let perp_state = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address_str)
		.await
		.map_err(|e| {
			error!("Failed to get perp state after hedge open: {}", e);
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
			error!("Hedge position for {} not found after opening", collateral_ticker);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Position not found").with_reason(format!(
					"Position for {} not found after hedge order opened",
					collateral_ticker
				)),
			)
		})?;

	let actual_position_size = hedge_position.position.szi.parse::<f64>().map_err(|e| {
		error!("Failed to parse position size: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid position size: {}", e)),
		)
	})?;

	info!(
		"Actual hedge position size opened: {} (expected: ~{})",
		actual_position_size, clamped_hedge_size_f64
	);

	// Update loan record with actual position size
	let updated_loan_record =
		LoanRecord { position_size: format!("{}", actual_position_size), ..loan_record };

	let storage_key = executor_storage::loan_record::Key {
		account_id: omni_account.clone(),
		nonce: skeleton_user_op.nonce as u64,
	};

	if ctx.loan_record_storage.insert(&storage_key, updated_loan_record).is_err() {
		error!(
			"Failed to update loan record with position size for omni_account {:?}, nonce {}",
			omni_account, skeleton_user_op.nonce
		);
		// Don't fail the entire operation, just log the error
	} else {
		info!(
			"Updated loan record with position_size {} for omni_account {:?}, nonce {}",
			actual_position_size, omni_account, skeleton_user_op.nonce
		);
	}

	let usdc_received_str = format!("{:.2}", usdc_to_lend);

	Ok(RequestLoanTestResponse {
		spot_sell_cloid: spot_sell_cloid.to_string(),
		hedge_open_cloid: hedge_open_cloid.to_string(),
		usdc_received: usdc_received_str,
		spot_sell_tx_hash,
		hedge_open_tx_hash,
	})
}

/// Precheck: Performs all validation checks for loan request
async fn precheck(
	hypercore_client: &HyperCoreClient,
	smart_wallet_address: &str,
	collateral_ticker: &str,
	collateral_size: f64,
	lending_ratio: u32,
) -> Result<LoanRequestValidationResult, PumpxRpcError> {
	// Fetch metadata and market prices from HyperCore in one call each
	// Get spot market prices (markPx and midPx) along with spot metadata
	let (spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(collateral_ticker).await.map_err(|e| {
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
		hypercore_client.get_perp_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get perp market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
					"Failed to get perp market prices for {}: {}",
					collateral_ticker, e
				)),
			)
		})?;

	// Get asset IDs
	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta).map_err(|e| {
		error!("Failed to get spot asset ID: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	let perp_asset_id = get_perp_asset_id(collateral_ticker, &meta).map_err(|e| {
		error!("Failed to get perp asset ID: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	info!(
		"Resolved asset IDs - spot: {}, perp: {} for ticker: {}",
		spot_asset_id, perp_asset_id, collateral_ticker
	);

	// Get token metadata
	let collateral_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(collateral_ticker))
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

	// 1. Validate collateral size for spot trading
	validate_trade_size(collateral_size, collateral_token.sz_decimals, None).map_err(|e| {
		error!("Invalid collateral size for spot trading: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid collateral size: {}", e)),
		)
	})?;

	info!(
		"✓ Collateral size {} validated for spot trading (sz_decimals={})",
		collateral_size, collateral_token.sz_decimals
	);

	// 2. Calculate estimated values for perp position using worst-case prices
	let lending_ratio_f64 = (lending_ratio as f64) / 100.0;

	// Worst case for spot sell: highest buy price (bid) with buffer
	let (spot_bid_price, _spot_ask_price) = get_bid_ask_prices(spot_mark_price, spot_mid_price);
	let worst_case_spot_sell_price = spot_bid_price * SPOT_SELL_PRICE_RATIO;
	let estimated_usdc_from_spot = collateral_size * worst_case_spot_sell_price;
	let estimated_usdc_for_perp = estimated_usdc_from_spot * (1.0 - lending_ratio_f64);
	let estimated_leverage = (1.0 / (1.0 - lending_ratio_f64)).min(perp_asset.max_leverage as f64);
	let estimated_perp_notional = estimated_usdc_for_perp * estimated_leverage;

	// 3. Validate minimum perp order notional value ($10 minimum)
	const MIN_PERP_NOTIONAL: f64 = 10.0;

	if estimated_perp_notional < MIN_PERP_NOTIONAL {
		error!(
			"Perp order notional value too small: estimated ${:.2} (collateral_size={}, spot_price={:.2}, lending_ratio={}%, leverage={:.2}x) - minimum required: ${}",
			estimated_perp_notional, collateral_size, worst_case_spot_sell_price, lending_ratio, estimated_leverage, MIN_PERP_NOTIONAL
		);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
				"Perp order notional value too small: ${:.2} < ${} minimum. Increase collateral_size or decrease lending_ratio.",
				estimated_perp_notional, MIN_PERP_NOTIONAL
			)),
		));
	}

	info!(
		"✓ Perp notional value: ${:.2} (margin={:.2}, leverage={:.2}x) >= ${} minimum",
		estimated_perp_notional, estimated_usdc_for_perp, estimated_leverage, MIN_PERP_NOTIONAL
	);

	// 4. Validate estimated hedge size can be properly rounded to perp sz_decimals
	// Worst case for opening long position: lowest sell price (ask) with buffer
	let (_perp_bid_price, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let worst_case_perp_open_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
	let estimated_hedge_size = estimated_perp_notional / worst_case_perp_open_price;

	validate_trade_size(estimated_hedge_size, perp_asset.sz_decimals, None).map_err(|e| {
		error!("Invalid estimated hedge size for perp trading: {}", e);
		PumpxRpcError::from(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(
			format!(
				"Invalid estimated hedge size (margin={:.2}, leverage={:.2}x, perp_price={:.2}, size={}): {}",
				estimated_usdc_for_perp, estimated_leverage, worst_case_perp_open_price, estimated_hedge_size, e
			),
		))
	})?;

	info!(
		"✓ Estimated hedge size {} validated for perp trading (sz_decimals={})",
		estimated_hedge_size, perp_asset.sz_decimals
	);

	// 5. Validate user balance
	let user_balance = hypercore_client
		.get_spot_balance(smart_wallet_address, collateral_ticker)
		.await
		.map_err(|e| {
			error!("Failed to get user balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query balance: {}", e)),
			)
		})?;

	if user_balance < collateral_size {
		error!(
			"Insufficient balance: user has {} but needs {} {}",
			user_balance, collateral_size, collateral_ticker
		);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
				"Insufficient balance: user has {} but needs {} {}",
				user_balance, collateral_size, collateral_ticker
			)),
		));
	}

	info!(
		"✓ Balance check passed: user has {} {} (required: {})",
		user_balance, collateral_ticker, collateral_size
	);

	Ok(LoanRequestValidationResult {
		spot_asset_id,
		perp_asset_id,
		spot_sz_decimals: collateral_token.sz_decimals,
		perp_sz_decimals: perp_asset.sz_decimals,
		max_leverage: perp_asset.max_leverage,
		lending_ratio_f64,
	})
}
