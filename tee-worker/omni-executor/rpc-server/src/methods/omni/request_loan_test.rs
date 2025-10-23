use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::user_op::submit_corewriter_userop;
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use executor_storage::{LoanRecord, Storage};
use hyperliquid::*;
use jsonrpsee::RpcModule;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

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

			let address_bytes =
				hex::decode(params.omni_account.strip_prefix("0x").unwrap_or(&params.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason("Failed to decode omni account hex string"),
						)
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
						"Invalid omni account length: expected 32 bytes, got {}",
						address_bytes.len()
					)),
				));
			}

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

			let omni_account = AccountId::decode(&mut &address_bytes[..]).map_err(|_| {
				error!("Failed to decode AccountId from bytes");
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to decode AccountId from bytes"),
				)
			})?;

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

// Helper function to print account state
async fn print_account_state(hypercore_client: &HyperCoreClient, user_address: &str, label: &str) {
	info!("========== Account State: {} ==========", label);

	// Print spot balances
	match hypercore_client.get_spot_clearinghouse_state(user_address).await {
		Ok(spot_state) => {
			info!("Spot Balances:");
			for balance in &spot_state.balances {
				let total: f64 = balance.total.parse().unwrap_or(0.0);
				let hold: f64 = balance.hold.parse().unwrap_or(0.0);
				if total > 0.0 || hold > 0.0 {
					info!("  {} - Total: {}, Hold: {}", balance.coin, balance.total, balance.hold);
				}
			}
		},
		Err(e) => {
			info!("Failed to fetch spot balances: {}", e);
		},
	}

	// Print perp clearinghouse state
	match hypercore_client.get_perp_clearinghouse_state(user_address).await {
		Ok(perp_state) => {
			info!("Perp Margin Summary:");
			info!(
				"  Account Value: {}, Total Margin Used: {}, Withdrawable: {}",
				perp_state.margin_summary.account_value,
				perp_state.margin_summary.total_margin_used,
				perp_state.withdrawable
			);

			if !perp_state.asset_positions.is_empty() {
				info!("Open Positions:");
				for asset_pos in &perp_state.asset_positions {
					let pos = &asset_pos.position;
					info!(
						"  {} - Size: {}, Entry Px: {}, Position Value: {}, Unrealized PnL: {}, Leverage: {}x",
						pos.coin,
						pos.szi,
						pos.entry_px.as_ref().unwrap_or(&"N/A".to_string()),
						pos.position_value,
						pos.unrealized_pnl,
						pos.leverage.value
					);
				}
			} else {
				info!("Open Positions: None");
			}
		},
		Err(e) => {
			info!("Failed to fetch perp clearinghouse state: {}", e);
		},
	}

	info!("==========================================");
}

// Helper function to validate loan request parameters
#[allow(clippy::too_many_arguments)]
async fn validate_loan_request_parameters(
	hypercore_client: &HyperCoreClient,
	smart_wallet_address: &str,
	collateral_ticker: &str,
	collateral_size: f64,
	collateral_token: &SpotToken,
	perp_asset: &PerpAsset,
	lending_ratio: u32,
	spot_market_price: f64,
	perp_market_price: f64,
) -> Result<(f64, f64), PumpxRpcError> {
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

	// 2. Calculate estimated values for perp position
	let lending_ratio_f64 = (lending_ratio as f64) / 100.0;
	let estimated_usdc_from_spot = collateral_size * spot_market_price;
	let estimated_usdc_for_perp = estimated_usdc_from_spot * (1.0 - lending_ratio_f64);
	let estimated_leverage = (1.0 / (1.0 - lending_ratio_f64)).min(perp_asset.max_leverage as f64);
	let estimated_perp_notional = estimated_usdc_for_perp * estimated_leverage;

	// 3. Validate minimum perp order notional value ($10 minimum)
	const MIN_PERP_NOTIONAL: f64 = 10.0;

	if estimated_perp_notional < MIN_PERP_NOTIONAL {
		error!(
			"Perp order notional value too small: estimated ${:.2} (collateral_size={}, spot_price={:.2}, lending_ratio={}%, leverage={:.2}x) - minimum required: ${}",
			estimated_perp_notional, collateral_size, spot_market_price, lending_ratio, estimated_leverage, MIN_PERP_NOTIONAL
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
	let estimated_hedge_size = estimated_perp_notional / perp_market_price;

	validate_trade_size(estimated_hedge_size, perp_asset.sz_decimals, None).map_err(|e| {
		error!("Invalid estimated hedge size for perp trading: {}", e);
		PumpxRpcError::from(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(
			format!(
				"Invalid estimated hedge size (margin={:.2}, leverage={:.2}x, perp_price={:.2}, size={}): {}",
				estimated_usdc_for_perp, estimated_leverage, perp_market_price, estimated_hedge_size, e
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

	Ok((user_balance, lending_ratio_f64))
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

	// Fetch metadata from HyperCore
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

	let _usdc_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case("USDC"))
		.ok_or_else(|| {
			error!("USDC token not found in spot meta");
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("USDC token not found in spot meta"),
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
		.get_spot_mid_price(collateral_ticker, &spot_meta)
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
		hypercore_client.get_perp_mid_price(collateral_ticker).await.map_err(|e| {
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

	// Run all early validations
	let (_user_balance, lending_ratio_f64) = validate_loan_request_parameters(
		&hypercore_client,
		smart_wallet_address_str,
		collateral_ticker,
		collateral_size,
		collateral_token,
		perp_asset,
		lending_ratio,
		spot_market_price,
		perp_market_price,
	)
	.await?;

	// Print initial account state
	print_account_state(&hypercore_client, smart_wallet_address_str, "Before Actions").await;

	// Action 1: Sell collateral_size as spot to get X USDC
	let clamped_size = clamp_size(collateral_size, collateral_token.sz_decimals);
	let target_price = spot_market_price * SPOT_SELL_PRICE_RATIO;
	let clamped_price = clamp_price(target_price, collateral_token.sz_decimals, true);

	info!(
		"Clamped values for spot sell - size: {} -> {}, price: {} -> {}",
		collateral_size, clamped_size, target_price, clamped_price
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

	let spot_sell_size_units = (clamped_size_f64 * 100_000_000.0) as u64;
	let spot_sell_price_units = (clamped_price_f64 * 100_000_000.0) as u64;

	let spot_sell_cloid = generate_cloid();
	let hedge_open_cloid = generate_cloid() + 1;

	info!(
		"Generated cloids - spot_sell: {} (hex: 0x{:032x}), hedge_open: {} (hex: 0x{:032x})",
		spot_sell_cloid, spot_sell_cloid, hedge_open_cloid, hedge_open_cloid
	);

	// Build and submit spot sell action
	let spot_sell_action = build_spot_sell_order(
		spot_asset_id,
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

	info!("Action 1: Spot sell submitted with tx_hash: {:?}", spot_sell_tx_hash);

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

	print_account_state(&hypercore_client, smart_wallet_address_str, "After Action 1 - Spot Sell")
		.await;

	// Calculate USDC allocation
	let usdc_for_perp = usdc_received * (1.0 - lending_ratio_f64);
	let usdc_to_lend = usdc_received * lending_ratio_f64;

	info!(
		"USDC allocation: total_received={:.2}, for_perp={:.2}, to_lend={:.2}",
		usdc_received, usdc_for_perp, usdc_to_lend
	);

	// Store loan record in storage
	let loan_record = LoanRecord {
		collateral_ticker: collateral_ticker.to_string(),
		collateral_size: collateral_size_str.to_string(),
		usdc_sold: format!("{:.2}", usdc_received),
		usdc_loaned: format!("{:.2}", usdc_to_lend),
		spot_sell_cloid: spot_sell_cloid.to_string(),
		hedge_open_cloid: hedge_open_cloid.to_string(),
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
	let usdc_for_perp_units = (usdc_for_perp * 1_000_000.0) as u64;
	let usd_transfer_action = build_usd_class_transfer_to_perp(usdc_for_perp_units);
	let usd_transfer_corewriter_calldata = encode_send_raw_action(usd_transfer_action);
	let usd_transfer_calldata =
		encode_omni_account_execute(get_core_writer_address(), usd_transfer_corewriter_calldata);

	let mut skeleton_action2 = skeleton_user_op.clone();
	skeleton_action2.nonce = current_nonce;
	skeleton_action2.init_code = "0x".to_string();

	let _usd_transfer_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_action2,
		chain_id,
		wallet_index,
		usd_transfer_calldata,
		client_id,
	)
	.await?;

	info!("Action 2: USD class transfer submitted");

	// Get initial perp balance
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

	print_account_state(
		&hypercore_client,
		smart_wallet_address_str,
		"After Action 2 - USD Transfer to Perp",
	)
	.await;

	// Action 3: Open hedge position
	current_nonce += 1;

	let desired_leverage: f64 = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(perp_asset.max_leverage as f64);
	let hedge_size = (usdc_for_perp * effective_leverage) / perp_market_price;

	let clamped_hedge_size = clamp_size(hedge_size, perp_asset.sz_decimals);
	let target_hedge_price = perp_market_price * PERP_ENTRY_PRICE_RATIO;
	let clamped_hedge_price = clamp_price(target_hedge_price, perp_asset.sz_decimals, false);

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

	let hedge_size_units = (clamped_hedge_size_f64 * 100_000_000.0) as u64;
	let hedge_price_units = (clamped_hedge_price_f64 * 100_000_000.0) as u64;

	let hedge_action =
		build_perp_long_order(perp_asset_id, hedge_size_units, hedge_price_units, hedge_open_cloid);
	let hedge_corewriter_calldata = encode_send_raw_action(hedge_action);
	let hedge_calldata =
		encode_omni_account_execute(get_core_writer_address(), hedge_corewriter_calldata);

	let mut skeleton_action3 = skeleton_user_op.clone();
	skeleton_action3.nonce = current_nonce;
	skeleton_action3.init_code = "0x".to_string();

	let _hedge_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		&skeleton_action3,
		chain_id,
		wallet_index,
		hedge_calldata,
		client_id,
	)
	.await?;

	info!("Action 3: Hedge position submitted");

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

	print_account_state(&hypercore_client, smart_wallet_address_str, "After Action 3 - Hedge Open")
		.await;

	let usdc_received_str = format!("{:.2}", usdc_to_lend);

	Ok(RequestLoanTestResponse {
		spot_sell_cloid: spot_sell_cloid.to_string(),
		hedge_open_cloid: hedge_open_cloid.to_string(),
		usdc_received: usdc_received_str,
		spot_sell_tx_hash,
		hedge_open_tx_hash: None,
	})
}
