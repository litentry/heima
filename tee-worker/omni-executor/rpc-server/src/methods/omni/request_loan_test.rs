use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_corewriter_userop;
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use executor_storage::{LoanState, Storage};
use hyperliquid::*;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

// minimum perp order notional value ($10 minimum)
const MIN_PERP_NOTIONAL: f64 = 10.0;

#[derive(Debug, Deserialize)]
pub struct RequestLoanTestParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub lending_ratio: u32,
	pub loan_nonce: Option<u64>, // If Some, resume existing loan; if None, create new
}

#[derive(Serialize, Clone)]
pub struct RequestLoanTestResponse {
	pub spot_sell_cloid: String,
	pub hedge_open_cloid: String,
	pub usdc_received: String,
	pub spot_sell_tx_hash: Option<String>,
	pub hedge_open_tx_hash: Option<String>,
}

struct ExecutionContext<'a, CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static> {
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	omni_account: &'a AccountId,
	skeleton_user_op: &'a SerializablePackedUserOperation,
	chain_id: u64,
	wallet_index: u32,
	hypercore_client: &'a HyperCoreClient,
	smart_wallet: &'a str,
	storage_key: &'a executor_storage::loan_record::Key,
}

struct SellSpotContext {
	spot_asset_id: u32,
	spot_sz_decimals: u8,
	spot_mark_price: f64,
	spot_mid_price: f64,
}

struct OpenPositionContext {
	perp_asset_id: u32,
	perp_sz_decimals: u8,
	perp_max_leverage: u32,
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

			let (omni_account, collateral_ticker, collateral_size, lending_ratio_f64) =
				precheck_params(&params)?;

			let ctx = Arc::clone(&ctx);
			let smart_wallet = &params.user_operation.sender;

			// Calculate storage_key based on loan_nonce parameter
			let storage_key = executor_storage::loan_record::Key {
				account_id: omni_account.clone(),
				nonce: params.loan_nonce.unwrap_or(params.user_operation.nonce as u64),
			};

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|e| {
				PumpxRpcError::from(
					DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported").with_reason(e),
				)
			})?;

			let exec_ctx = ExecutionContext {
				ctx: ctx.clone(),
				omni_account: &omni_account,
				skeleton_user_op: &params.user_operation,
				chain_id: params.chain_id,
				wallet_index: params.wallet_index,
				hypercore_client: &hypercore_client,
				smart_wallet,
				storage_key: &storage_key,
			};
			let mut current_nonce = params.user_operation.nonce;

			match ctx.loan_record_storage.get(&storage_key).ok().flatten() {
				None => {
					info!("Starting new loan request");
					hypercore_client.print_account_state(smart_wallet, "Before Request").await;

					let sell_ctx = precheck_sell_spot(
						&hypercore_client,
						smart_wallet,
						&collateral_ticker,
						collateral_size,
					)
					.await?;

					// use spot_mid_price as estimation
					let usdc_for_perp =
						sell_ctx.spot_mid_price * collateral_size * (1.0 - lending_ratio_f64);

					let open_ctx = precheck_open_position(
						&hypercore_client,
						&collateral_ticker,
						usdc_for_perp,
						lending_ratio_f64,
					)
					.await?;

					let (usdc_sold, spot_cloid, hedge_cloid) = do_sell_spot(
						&exec_ctx,
						&collateral_ticker,
						collateral_size,
						lending_ratio_f64,
						&sell_ctx,
						&mut current_nonce,
					)
					.await?;

					let usdc_loaned = usdc_sold * lending_ratio_f64;
					let usdc_for_perp = usdc_sold * (1.0 - lending_ratio_f64);

					do_move_to_perp(&exec_ctx, usdc_for_perp, &mut current_nonce).await?;

					do_open_position(
						&exec_ctx,
						&collateral_ticker,
						usdc_for_perp,
						lending_ratio_f64,
						hedge_cloid,
						&open_ctx,
						current_nonce,
					)
					.await?;

					Ok(RequestLoanTestResponse {
						spot_sell_cloid: spot_cloid.to_string(),
						hedge_open_cloid: hedge_cloid.to_string(),
						usdc_received: format!("{:.2}", usdc_loaned),
						spot_sell_tx_hash: None,
						hedge_open_tx_hash: None,
					})
				},
				Some(existing) => {
					if existing.collateral_ticker != collateral_ticker
						|| existing.collateral_size != format!("{}", collateral_size)
					{
						return Err(PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Loan parameters mismatch")
								.with_reason(format!(
									"Existing: {}@{}; Requested: {}@{}",
									existing.collateral_ticker,
									existing.collateral_size,
									collateral_ticker,
									collateral_size
								)),
						));
					}

					match existing.state {
						LoanState::PositionOpened => {
							info!("Loan already completed");
							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing.spot_sell_cloid,
								hedge_open_cloid: existing.hedge_open_cloid,
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								hedge_open_tx_hash: None,
							})
						},
						LoanState::SpotSold => {
							info!("Resuming from SpotSold");

							let usdc_for_perp =
								existing.usdc_for_perp.parse::<f64>().map_err(|e| {
									PumpxRpcError::from(
										DetailedError::new(
											INTERNAL_ERROR_CODE,
											"Invalid stored data",
										)
										.with_reason(format!("Invalid usdc_for_perp: {}", e)),
									)
								})?;

							let open_ctx = precheck_open_position(
								&hypercore_client,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
							)
							.await?;

							let hedge_cloid =
								existing.hedge_open_cloid.parse::<u128>().map_err(|e| {
									PumpxRpcError::from(
										DetailedError::new(
											INTERNAL_ERROR_CODE,
											"Invalid stored data",
										)
										.with_reason(format!("Invalid hedge_open_cloid: {}", e)),
									)
								})?;

							do_move_to_perp(&exec_ctx, usdc_for_perp, &mut current_nonce).await?;

							do_open_position(
								&exec_ctx,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
								hedge_cloid,
								&open_ctx,
								current_nonce,
							)
							.await?;

							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing.spot_sell_cloid,
								hedge_open_cloid: existing.hedge_open_cloid,
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								hedge_open_tx_hash: None,
							})
						},
						LoanState::ToPerpMoved => {
							info!("Resuming from ToPerpMoved");

							let usdc_for_perp =
								existing.usdc_for_perp.parse::<f64>().map_err(|e| {
									PumpxRpcError::from(
										DetailedError::new(
											INTERNAL_ERROR_CODE,
											"Invalid stored data",
										)
										.with_reason(format!("Invalid usdc_for_perp: {}", e)),
									)
								})?;

							let open_ctx = precheck_open_position(
								&hypercore_client,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
							)
							.await?;

							let hedge_cloid =
								existing.hedge_open_cloid.parse::<u128>().map_err(|e| {
									PumpxRpcError::from(
										DetailedError::new(
											INTERNAL_ERROR_CODE,
											"Invalid stored data",
										)
										.with_reason(format!("Invalid hedge_open_cloid: {}", e)),
									)
								})?;

							do_open_position(
								&exec_ctx,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
								hedge_cloid,
								&open_ctx,
								current_nonce,
							)
							.await?;

							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing.spot_sell_cloid,
								hedge_open_cloid: existing.hedge_open_cloid,
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								hedge_open_tx_hash: None,
							})
						},
						_ => Err(PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Invalid loan state")
								.with_reason(format!(
									"Cannot resume from state: {:?}",
									existing.state
								)),
						)),
					}
				},
			}
		})
		.expect("Failed to register omni_requestLoanTest method");
}

fn precheck_params(
	params: &RequestLoanTestParams,
) -> Result<(AccountId, String, f64, f64), PumpxRpcError> {
	let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
		PumpxRpcError::from(DetailedError::new(PARSE_ERROR_CODE, "Invalid omni account"))
	})?;

	params.user_operation.sender.parse::<Address>().map_err(|e| {
		PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Invalid sender address")
				.with_field("sender")
				.with_reason(format!("{}", e)),
		)
	})?;

	if params.collateral_ticker.is_empty() {
		return Err(PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Empty collateral ticker")
				.with_field("collateral_ticker"),
		));
	}

	if params.collateral_size.is_empty() {
		return Err(PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Empty collateral size")
				.with_field("collateral_size"),
		));
	}

	if params.lending_ratio > 100 {
		return Err(PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Invalid lending ratio")
				.with_field("lending_ratio")
				.with_received(params.lending_ratio.to_string())
				.with_expected("0-100"),
		));
	}

	let collateral_ticker = params.collateral_ticker.to_uppercase();
	let collateral_size = params.collateral_size.parse::<f64>().map_err(|e| {
		PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Invalid collateral size")
				.with_field("collateral_size")
				.with_reason(format!("{}", e)),
		)
	})?;
	let lending_ratio_f64 = params.lending_ratio as f64 / 100.0;

	Ok((omni_account, collateral_ticker, collateral_size, lending_ratio_f64))
}

async fn precheck_sell_spot(
	hypercore_client: &HyperCoreClient,
	smart_wallet: &str,
	collateral_ticker: &str,
	collateral_size: f64,
) -> Result<SellSpotContext, PumpxRpcError> {
	let (spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get spot market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get spot market prices: {}", e)),
			)
		})?;

	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta).map_err(|e| {
		error!("Failed to get spot asset ID: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e),
		)
	})?;

	let spot_token = spot_meta
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

	let spot_sz_decimals = spot_token.sz_decimals;

	// Validate trade size
	validate_trade_size(collateral_size, spot_sz_decimals, None).map_err(|e| {
		error!("Invalid collateral size for spot trading: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid collateral size: {}", e)),
		)
	})?;

	// Validate balance
	let user_balance = hypercore_client
		.get_spot_balance(smart_wallet, collateral_ticker)
		.await
		.map_err(|e| {
			error!("Failed to get user balance: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query balance: {}", e)),
			)
		})?;

	if user_balance < collateral_size {
		error!("Insufficient balance: has {} but needs {}", user_balance, collateral_size);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Insufficient balance").with_reason(format!(
				"User has {} but needs {} {}",
				user_balance, collateral_size, collateral_ticker
			)),
		));
	}

	info!("✓ Spot sell precheck passed");

	Ok(SellSpotContext { spot_asset_id, spot_sz_decimals, spot_mark_price, spot_mid_price })
}

async fn precheck_open_position(
	hypercore_client: &HyperCoreClient,
	collateral_ticker: &str,
	usdc_for_perp: f64,
	lending_ratio_f64: f64,
) -> Result<OpenPositionContext, PumpxRpcError> {
	let (perp_meta, perp_mark_price, perp_mid_price) =
		hypercore_client.get_perp_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get perp market prices for {}: {}", collateral_ticker, e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get perp market prices: {}", e)),
			)
		})?;

	let perp_asset_id = get_perp_asset_id(collateral_ticker, &perp_meta).map_err(|e| {
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
	let perp_max_leverage = perp_asset.max_leverage;

	let desired_leverage = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(perp_max_leverage as f64);

	let estimated_notional = usdc_for_perp * effective_leverage;

	if estimated_notional < MIN_PERP_NOTIONAL {
		error!(
			"Perp notional too small, estimated: {}, required: {}",
			estimated_notional, MIN_PERP_NOTIONAL
		);
		return Err(PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
				"Perp notional too small, estimated: {}, required: {}",
				estimated_notional, MIN_PERP_NOTIONAL
			)),
		));
	}

	// Worst case for opening long position: lowest sell price (ask) with buffer
	let (_perp_bid_price, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let worst_case_perp_open_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
	let estimated_hedge_size = estimated_notional / worst_case_perp_open_price;

	validate_trade_size(estimated_hedge_size, perp_sz_decimals, None).map_err(|e| {
		error!("Invalid estimated hedge size for perp trading: {}", e);
		PumpxRpcError::from(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(
			format!(
				"Invalid estimated hedge size (margin={:.2}, leverage={:.2}x, perp_price={:.2}, size={}): {}",
				usdc_for_perp, effective_leverage, worst_case_perp_open_price, estimated_hedge_size, e
			),
		))
	})?;

	info!("✓ Open position precheck passed");

	Ok(OpenPositionContext { perp_asset_id, perp_sz_decimals, perp_max_leverage })
}

async fn do_sell_spot<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	collateral_ticker: &str,
	collateral_size: f64,
	lending_ratio_f64: f64,
	sell_ctx: &SellSpotContext,
	current_nonce: &mut u128,
) -> Result<(f64, u128, u128), PumpxRpcError> {
	info!("Action: Selling {} {} in spot market", collateral_size, collateral_ticker);

	let clamped_size = clamp_size(collateral_size, sell_ctx.spot_sz_decimals);
	let (spot_bid_price, _) = get_bid_ask_prices(sell_ctx.spot_mark_price, sell_ctx.spot_mid_price);
	let target_price = spot_bid_price * SPOT_SELL_PRICE_RATIO;
	let clamped_price = clamp_price(target_price, sell_ctx.spot_sz_decimals, true);

	let clamped_size_f64 = clamped_size.parse::<f64>().map_err(|e| {
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped size: {}", e)),
		)
	})?;
	let clamped_price_f64 = clamped_price.parse::<f64>().map_err(|e| {
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped price: {}", e)),
		)
	})?;

	let spot_sell_cloid = generate_cloid();
	let hedge_open_cloid = spot_sell_cloid + 1;

	let spot_sell_action = build_spot_sell_order(
		sell_ctx.spot_asset_id,
		to_price_units(clamped_size_f64),
		to_price_units(clamped_price_f64),
		spot_sell_cloid,
	);

	let mut user_op = exec_ctx.skeleton_user_op.clone();
	user_op.nonce = *current_nonce;
	// init_code is kept as-is from skeleton

	let spot_sell_tx_hash = submit_corewriter_userop(
		exec_ctx.ctx.clone(),
		exec_ctx.omni_account,
		&user_op,
		exec_ctx.chain_id,
		exec_ctx.wallet_index,
		encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(spot_sell_action),
		),
	)
	.await?;
	*current_nonce += 1;

	info!(
		"Spot sell submitted: price={}, size={}, tx={:?}",
		clamped_price_f64, clamped_size_f64, spot_sell_tx_hash
	);

	let order_filled = exec_ctx
		.hypercore_client
		.wait_for_order(
			exec_ctx.smart_wallet,
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

	let spot_sell_fill = exec_ctx
		.hypercore_client
		.get_fill_by_cloid(exec_ctx.smart_wallet, spot_sell_cloid)
		.await
		.map_err(|e| {
			error!("Failed to get fill for spot sell order: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to get fill: {}", e)),
			)
		})?;

	let usdc_sold = usdc_from_spot_fill(&spot_sell_fill).map_err(|e| {
		error!("Failed to calculate USDC received: {}", e);
		PumpxRpcError::from(
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to calculate USDC: {}", e)),
		)
	})?;

	info!("Spot sell completed: received {:.2} USDC", usdc_sold);

	let usdc_loaned = usdc_sold * lending_ratio_f64;
	let usdc_for_perp = usdc_sold * (1.0 - lending_ratio_f64);

	// Store loan record with SpotSold state
	exec_ctx
		.ctx
		.loan_record_storage
		.create(
			exec_ctx.storage_key,
			executor_storage::loan_record::NewLoanRecord {
				collateral_ticker: collateral_ticker.to_string(),
				collateral_size: collateral_size.to_string(),
				usdc_sold: format!("{:.2}", usdc_sold),
				usdc_loaned: format!("{:.2}", usdc_loaned),
				usdc_for_perp: format!("{:.2}", usdc_for_perp),
				spot_sell_cloid: spot_sell_cloid.to_string(),
				hedge_open_cloid: hedge_open_cloid.to_string(),
			},
		)
		.map_err(|e| {
			error!("Failed to create loan record: {}", e);
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to create loan record: {}", e)),
			)
		})?;

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Spot Sell")
		.await;

	Ok((usdc_sold, spot_sell_cloid, hedge_open_cloid))
}

async fn do_move_to_perp<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	usdc_for_perp: f64,
	current_nonce: &mut u128,
) -> Result<(), PumpxRpcError> {
	info!("Action: Moving {:.2} USDC to perp", usdc_for_perp);

	let initial_perp_balance = exec_ctx
		.hypercore_client
		.get_perp_clearinghouse_state(exec_ctx.smart_wallet)
		.await
		.map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to query perp balance: {}", e)),
			)
		})?
		.cross_margin_summary
		.account_value
		.parse::<f64>()
		.map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to parse perp balance: {}", e)),
			)
		})?;

	// Clear init_code (account already created by sell_spot)
	let mut user_op = exec_ctx.skeleton_user_op.clone();
	user_op.nonce = *current_nonce;
	user_op.init_code = "0x".to_string();

	let usd_transfer_tx_hash = submit_corewriter_userop(
		exec_ctx.ctx.clone(),
		exec_ctx.omni_account,
		&user_op,
		exec_ctx.chain_id,
		exec_ctx.wallet_index,
		encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(build_usd_class_transfer_to_perp(to_usdc_units(usdc_for_perp))),
		),
	)
	.await?;
	*current_nonce += 1;

	info!("USD transfer submitted: size={:.2}, tx={:?}", usdc_for_perp, usd_transfer_tx_hash);

	exec_ctx
		.hypercore_client
		.wait_for_perp_balance_increase(
			exec_ctx.smart_wallet,
			initial_perp_balance,
			usdc_for_perp,
			20,
		)
		.await
		.map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("USD transfer failed: {}", e)),
			)
		})?;

	info!("USD transfer completed");

	// Update state: ToPerpMoved
	let _ = exec_ctx
		.ctx
		.loan_record_storage
		.update(exec_ctx.storage_key, |r| r.state = LoanState::ToPerpMoved);

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Move To Perp")
		.await;

	Ok(())
}

async fn do_open_position<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	collateral_ticker: &str,
	usdc_for_perp: f64,
	lending_ratio_f64: f64,
	hedge_open_cloid: u128,
	open_ctx: &OpenPositionContext,
	current_nonce: u128,
) -> Result<(), PumpxRpcError> {
	let desired_leverage = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(open_ctx.perp_max_leverage as f64);

	info!("Action: Opening hedge position for {}", collateral_ticker);

	// Refresh prices
	let (_, perp_mark_price, perp_mid_price) = exec_ctx
		.hypercore_client
		.get_perp_market_prices(collateral_ticker)
		.await
		.map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Failed to refresh perp prices: {}", e)),
			)
		})?;

	let (_, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let target_hedge_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
	let hedge_size = (usdc_for_perp * effective_leverage) / target_hedge_price;

	let clamped_hedge_size = clamp_size(hedge_size, open_ctx.perp_sz_decimals);
	let clamped_hedge_price = clamp_price(target_hedge_price, open_ctx.perp_sz_decimals, false);

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

	let hedge_action = build_perp_long_order(
		open_ctx.perp_asset_id,
		to_price_units(clamped_hedge_size_f64),
		to_price_units(clamped_hedge_price_f64),
		hedge_open_cloid,
	);

	// Clear init_code (account already created)
	let mut user_op = exec_ctx.skeleton_user_op.clone();
	user_op.nonce = current_nonce;
	user_op.init_code = "0x".to_string();

	let hedge_open_tx_hash = submit_corewriter_userop(
		exec_ctx.ctx.clone(),
		exec_ctx.omni_account,
		&user_op,
		exec_ctx.chain_id,
		exec_ctx.wallet_index,
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

	let order_opened = exec_ctx
		.hypercore_client
		.wait_for_order(
			exec_ctx.smart_wallet,
			&hedge_open_cloid.to_string(),
			20,
			OrderWaitCondition::Opened,
		)
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

	// Update loan record with position size and state: PositionOpened
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.position_size = format!("{}", clamped_hedge_size_f64);
		r.state = LoanState::PositionOpened;
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Open Position")
		.await;

	Ok(())
}
