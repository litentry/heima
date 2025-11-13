use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::types::{RpcOptionExt, RpcResultExt};
use crate::utils::user_op::submit_corewriter_user_ops;
use crate::utils::validation::{parse_as, parse_rpc_params, validate_evm_address};
use crate::RpcResult;
use jsonrpsee::RpcModule;
use oe_client_hyperliquid::*;
use oe_core::intent_executor::IntentExecutor;
use oe_core::types::SerializablePackedUserOperation;
use oe_primitives::{AccountId, ChainId};
use oe_storage::{LoanState, Storage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Truncates a float to at most 3 decimal places without rounding.
/// Removes trailing zeros for cleaner storage.
///
/// Examples: 9.1866 -> "9.186" (truncated, not rounded to 9.187)
fn truncate_usdc(value: f64) -> String {
	let truncated = (value * 1000.0).floor() / 1000.0;
	format!("{:.3}", truncated)
		.trim_end_matches('0')
		.trim_end_matches('.')
		.to_string()
}

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
	pub to_perp_move_tx_hash: Option<String>,
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
	storage_key: &'a oe_storage::loan_record::Key,
}

struct SellSpotContext {
	spot_asset_id: u32,
	spot_sz_decimals: u8,
	spot_mark_price: f64,
	spot_mid_price: f64,
}

struct OpenHedgeContext {
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
			let params = parse_rpc_params::<RequestLoanTestParams>(params)?;

			debug!("Received omni_requestLoanTest, params: {:?}", params);

			let (omni_account, collateral_ticker, collateral_size, lending_ratio_f64) =
				precheck_params(&params)?;

			let ctx = Arc::clone(&ctx);
			let smart_wallet = &params.user_operation.sender;

			// Calculate storage_key based on loan_nonce parameter
			let storage_key = oe_storage::loan_record::Key {
				account_id: omni_account.clone(),
				nonce: params.loan_nonce.unwrap_or(params.user_operation.nonce as u64),
			};

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|_| {
				DetailedError::internal_error("HyperCore client error").to_rpc_error()
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

					let open_ctx = precheck_open_hedge(
						&hypercore_client,
						&collateral_ticker,
						usdc_for_perp,
						lending_ratio_f64,
					)
					.await?;

					let (usdc_sold, spot_sell_cloid, spot_sell_tx_hash) = do_sell_spot(
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

					let to_perp_move_tx_hash =
						do_move_to_perp(&exec_ctx, usdc_for_perp, &mut current_nonce).await?;

					let (hedge_open_cloid, hedge_open_tx_hash) = do_open_hedge(
						&exec_ctx,
						&collateral_ticker,
						usdc_for_perp,
						lending_ratio_f64,
						&open_ctx,
						current_nonce,
					)
					.await?;

					Ok(RequestLoanTestResponse {
						spot_sell_cloid: spot_sell_cloid.to_string(),
						hedge_open_cloid: hedge_open_cloid.to_string(),
						usdc_received: truncate_usdc(usdc_loaned),
						spot_sell_tx_hash,
						to_perp_move_tx_hash,
						hedge_open_tx_hash,
					})
				},
				Some(existing) => {
					if existing.collateral_ticker != collateral_ticker
						|| existing.collateral_size != format!("{}", collateral_size)
					{
						return Err(DetailedError::invalid_params(
							"collateral",
							"mismatch collateral ticker or size",
						)
						.to_rpc_error());
					}

					match existing.state {
						LoanState::HedgeOpened => {
							info!("Loan already completed");
							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing
									.cloids
									.iter()
									.find(|(name, _)| name == "spot_sell")
									.map(|(_, cloid)| cloid.clone())
									.unwrap_or_else(|| "0".to_string()),
								hedge_open_cloid: existing
									.cloids
									.iter()
									.find(|(name, _)| name == "hedge_open")
									.map(|(_, cloid)| cloid.clone())
									.unwrap_or_else(|| "0".to_string()),
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								to_perp_move_tx_hash: None,
								hedge_open_tx_hash: None,
							})
						},
						LoanState::SpotSold => {
							info!("Resuming from SpotSold");

							let usdc_for_perp: f64 =
								parse_as(&existing.usdc_for_perp, "usdc_for_perp")?;

							let open_ctx = precheck_open_hedge(
								&hypercore_client,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
							)
							.await?;

							let to_perp_move_tx_hash =
								do_move_to_perp(&exec_ctx, usdc_for_perp, &mut current_nonce)
									.await?;

							let (hedge_open_cloid, hedge_open_tx_hash) = do_open_hedge(
								&exec_ctx,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
								&open_ctx,
								current_nonce,
							)
							.await?;

							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing
									.cloids
									.iter()
									.find(|(name, _)| name == "spot_sell")
									.map(|(_, cloid)| cloid.clone())
									.unwrap_or_else(|| "0".to_string()),
								hedge_open_cloid: hedge_open_cloid.to_string(),
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								to_perp_move_tx_hash,
								hedge_open_tx_hash,
							})
						},
						LoanState::ToPerpMoved => {
							info!("Resuming from ToPerpMoved");

							let usdc_for_perp: f64 =
								parse_as(&existing.usdc_for_perp, "usdc_for_perp")?;

							let open_ctx = precheck_open_hedge(
								&hypercore_client,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
							)
							.await?;

							let (hedge_open_cloid, hedge_open_tx_hash) = do_open_hedge(
								&exec_ctx,
								&collateral_ticker,
								usdc_for_perp,
								lending_ratio_f64,
								&open_ctx,
								current_nonce,
							)
							.await?;

							Ok(RequestLoanTestResponse {
								spot_sell_cloid: existing
									.cloids
									.iter()
									.find(|(name, _)| name == "spot_sell")
									.map(|(_, cloid)| cloid.clone())
									.unwrap_or_else(|| "0".to_string()),
								hedge_open_cloid: hedge_open_cloid.to_string(),
								usdc_received: existing.usdc_loaned,
								spot_sell_tx_hash: None,
								to_perp_move_tx_hash: None,
								hedge_open_tx_hash,
							})
						},
						_ => Err(DetailedError::internal_error(&format!(
							"Unexpected state: {:?}",
							existing.state
						))
						.to_rpc_error()),
					}
				},
			}
		})
		.expect("Failed to register omni_requestLoanTest method");
}

fn precheck_params(params: &RequestLoanTestParams) -> RpcResult<(AccountId, String, f64, f64)> {
	let omni_account = to_omni_account(&params.omni_account)?;

	validate_evm_address(&params.user_operation.sender, "sender")?;

	if params.collateral_ticker.is_empty() {
		return Err(DetailedError::parse_error("Empty collateral_ticker").to_rpc_error());
	}

	if params.collateral_ticker.to_uppercase() == "USDC" {
		return Err(DetailedError::parse_error("Expect non-USDC collateral_ticker").to_rpc_error());
	}

	if params.collateral_size.is_empty() {
		return Err(DetailedError::parse_error("Empty collateral_size").to_rpc_error());
	}

	if params.lending_ratio > 100 {
		return Err(DetailedError::parse_error("Too large lending_ratio").to_rpc_error());
	}

	let collateral_ticker = params.collateral_ticker.to_uppercase();
	let collateral_size: f64 = parse_as(&params.collateral_size, "collateral_size")?;
	let lending_ratio_f64 = params.lending_ratio as f64 / 100.0;

	Ok((omni_account, collateral_ticker, collateral_size, lending_ratio_f64))
}

async fn precheck_sell_spot(
	hypercore_client: &HyperCoreClient,
	smart_wallet: &str,
	collateral_ticker: &str,
	collateral_size: f64,
) -> RpcResult<SellSpotContext> {
	let (spot_meta, spot_mark_price, spot_mid_price) = hypercore_client
		.get_spot_market_prices(collateral_ticker)
		.await
		.map_err_internal("Failed to get spot market prices")?;
	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta)
		.map_err_internal("Failed to get spot asset id")?;

	let spot_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_internal("Token not found in spot meta")?;

	let spot_sz_decimals = spot_token.sz_decimals;

	// Validate trade size
	validate_trade_size(collateral_size, spot_sz_decimals, None)
		.map_err_internal("Invalid collateral size for spot trading")?;

	// Validate notional value with clamped size
	let clamped_size = clamp_size(collateral_size, spot_sz_decimals);
	let clamped_size_f64: f64 = parse_as(&clamped_size, "clamped_size")?;

	let (spot_bid_price, _) = get_bid_ask_prices(spot_mark_price, spot_mid_price);
	validate_notional_value(spot_bid_price, clamped_size_f64, "Spot sell")
		.map_err_internal("Notional value too low")?;

	// Validate balance
	let user_balance = hypercore_client
		.get_spot_balance(smart_wallet, collateral_ticker)
		.await
		.map_err_internal("Failed to get user balance")?;

	if user_balance < collateral_size {
		let msg =
			format!("Insufficient balance: has {} but needs {}", user_balance, collateral_size);
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	}

	info!("✓ Spot sell precheck passed");

	Ok(SellSpotContext { spot_asset_id, spot_sz_decimals, spot_mark_price, spot_mid_price })
}

async fn precheck_open_hedge(
	hypercore_client: &HyperCoreClient,
	collateral_ticker: &str,
	usdc_for_perp: f64,
	lending_ratio_f64: f64,
) -> RpcResult<OpenHedgeContext> {
	let (perp_meta, perp_mark_price, perp_mid_price) = hypercore_client
		.get_perp_market_prices(collateral_ticker)
		.await
		.map_err_internal("Failed to get perp market prices")?;
	let perp_asset_id = get_perp_asset_id(collateral_ticker, &perp_meta)
		.map_err_internal("Failed to get perp asset id")?;

	let perp_asset = perp_meta
		.universe
		.get(perp_asset_id as usize)
		.ok_or_internal("Perp asset not found in meta")?;

	let perp_sz_decimals = perp_asset.sz_decimals;
	let perp_max_leverage = perp_asset.max_leverage;

	let desired_leverage = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(perp_max_leverage as f64);

	// Calculate estimated size for opening long position using lowest sell price (ask)
	let (_perp_bid_price, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let estimated_notional = usdc_for_perp * effective_leverage;
	let estimated_hedge_size = estimated_notional / perp_ask_price;

	validate_trade_size(estimated_hedge_size, perp_sz_decimals, None)
		.map_err_internal("Invalid estimated hedge size for perp trading")?;

	// Validate notional value with clamped size
	let clamped_size = clamp_size(estimated_hedge_size, perp_sz_decimals);
	let clamped_size_f64: f64 = parse_as(&clamped_size, "clamped_hedge_size")?;

	validate_notional_value(perp_ask_price, clamped_size_f64, "Perp open")
		.map_err_internal("Notional value too low")?;

	info!("✓ Open hedge precheck passed");

	Ok(OpenHedgeContext { perp_asset_id, perp_sz_decimals, perp_max_leverage })
}

async fn do_sell_spot<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	collateral_ticker: &str,
	collateral_size: f64,
	lending_ratio_f64: f64,
	sell_ctx: &SellSpotContext,
	current_nonce: &mut u128,
) -> RpcResult<(f64, u128, Option<String>)> {
	info!("Action: Selling {} {} in spot market", collateral_size, collateral_ticker);

	let clamped_size = clamp_size(collateral_size, sell_ctx.spot_sz_decimals);
	let (spot_bid_price, _) = get_bid_ask_prices(sell_ctx.spot_mark_price, sell_ctx.spot_mid_price);
	let target_price = spot_bid_price * SPOT_SELL_PRICE_RATIO;
	let clamped_price = clamp_price(target_price, sell_ctx.spot_sz_decimals, true);

	let clamped_size_f64: f64 = parse_as(&clamped_size, "clamped_size")?;
	let clamped_price_f64: f64 = parse_as(&clamped_price, "clamped_price")?;

	let spot_sell_cloid = generate_cloid();

	let spot_sell_action = build_spot_sell_order(
		sell_ctx.spot_asset_id,
		to_price_units(clamped_size_f64),
		to_price_units(clamped_price_f64),
		spot_sell_cloid,
	);

	let mut user_op = exec_ctx.skeleton_user_op.clone();
	user_op.nonce = *current_nonce;
	// init_code is kept as-is from skeleton

	let spot_sell_tx_hash = submit_corewriter_user_ops(
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
		"spot_sell submitted: price={}, size={}, tx={:?}",
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
		.map_err_internal("Spot sell order did not complete")?;

	if !order_filled {
		let msg = "Spot sell order was rejected or canceled".to_string();
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	}

	let spot_sell_fill = exec_ctx
		.hypercore_client
		.get_fill_by_cloid(exec_ctx.smart_wallet, spot_sell_cloid)
		.await
		.map_err_internal("Failed to get fill for spot sell order")?;

	let usdc_sold_f64 = usdc_from_spot_fill(&spot_sell_fill)
		.map_err_internal("Failed to calculate USDC received")?;

	let usdc_loaned_f64 = usdc_sold_f64 * lending_ratio_f64;
	let usdc_for_perp_f64 = usdc_sold_f64 * (1.0 - lending_ratio_f64);

	let usdc_sold = truncate_usdc(usdc_sold_f64);
	let usdc_loaned = truncate_usdc(usdc_loaned_f64);
	let usdc_for_perp = truncate_usdc(usdc_for_perp_f64);

	info!(
		"Spot sell completed: usdc_sold={}, usdc_loaned={}, usdc_for_perp={}",
		usdc_sold, usdc_loaned, usdc_for_perp
	);

	// Store loan record with SpotSold state
	exec_ctx
		.ctx
		.loan_record_storage
		.create(
			exec_ctx.storage_key,
			oe_storage::loan_record::NewLoanRecord {
				collateral_ticker: collateral_ticker.to_string(),
				collateral_size: collateral_size.to_string(),
				usdc_sold,
				usdc_loaned,
				usdc_for_perp,
			},
		)
		.map_err_internal("Failed to create loan record")?;

	// Populate spot_sell tx and cloid
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.cloids.push(("spot_sell".to_string(), spot_sell_cloid.to_string()));
		if let Some(ref tx_hash) = spot_sell_tx_hash {
			r.txs.push(("spot_sell".to_string(), tx_hash.clone()));
		}
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Spot Sell")
		.await;

	Ok((usdc_sold_f64, spot_sell_cloid, spot_sell_tx_hash))
}

async fn do_move_to_perp<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	usdc_for_perp: f64,
	current_nonce: &mut u128,
) -> RpcResult<Option<String>> {
	info!("Action: Moving {:.2} USDC to perp", usdc_for_perp);

	let perp_state = exec_ctx
		.hypercore_client
		.get_perp_clearinghouse_state(exec_ctx.smart_wallet)
		.await
		.map_err_internal("Failed to query perp balance")?;

	let initial_perp_balance: f64 =
		parse_as(&perp_state.cross_margin_summary.account_value, "initial_perp_balance")?;

	// Clear init_code (account already created by sell_spot)
	let mut user_op = exec_ctx.skeleton_user_op.clone();
	user_op.nonce = *current_nonce;
	user_op.init_code = "0x".to_string();

	let to_perp_move_tx_hash = submit_corewriter_user_ops(
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

	info!("to_perp_move submitted: size={:.2}, tx={:?}", usdc_for_perp, to_perp_move_tx_hash);

	exec_ctx
		.hypercore_client
		.wait_for_perp_balance_increase(
			exec_ctx.smart_wallet,
			initial_perp_balance,
			usdc_for_perp,
			20,
		)
		.await
		.map_err_internal("to_perp_move failed")?;

	info!("to_perp_move completed");

	// Update state: ToPerpMoved and populate to_perp_move tx
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.state = LoanState::ToPerpMoved;
		if let Some(ref tx_hash) = to_perp_move_tx_hash {
			r.txs.push(("to_perp_move".to_string(), tx_hash.clone()));
		}
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Move To Perp")
		.await;

	Ok(to_perp_move_tx_hash)
}

async fn do_open_hedge<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	collateral_ticker: &str,
	usdc_for_perp: f64,
	lending_ratio_f64: f64,
	open_ctx: &OpenHedgeContext,
	current_nonce: u128,
) -> RpcResult<(u128, Option<String>)> {
	let desired_leverage = 1.0 / (1.0 - lending_ratio_f64);
	let effective_leverage = desired_leverage.min(open_ctx.perp_max_leverage as f64);

	info!("Action: Opening hedge position for {}", collateral_ticker);
	let hedge_open_cloid = generate_cloid();

	// Refresh prices
	let (_, perp_mark_price, perp_mid_price) = exec_ctx
		.hypercore_client
		.get_perp_market_prices(collateral_ticker)
		.await
		.map_err_internal("Failed to refresh perp prices")?;

	let (_, perp_ask_price) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
	let target_hedge_price = perp_ask_price * PERP_ENTRY_PRICE_RATIO;
	let hedge_size = (usdc_for_perp * effective_leverage) / target_hedge_price;

	let clamped_hedge_size = clamp_size(hedge_size, open_ctx.perp_sz_decimals);
	let clamped_hedge_price = clamp_price(target_hedge_price, open_ctx.perp_sz_decimals, false);

	let clamped_hedge_size_f64: f64 = parse_as(&clamped_hedge_size, "clamped_hedge_size")?;
	let clamped_hedge_price_f64: f64 = parse_as(&clamped_hedge_price, "clamped_hedge_price")?;

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

	let hedge_open_tx_hash = submit_corewriter_user_ops(
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
		"hedge_open submitted: price={}, size={}, tx={:?}",
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
		.map_err_internal("Hedge order failed to open")?;

	if !order_opened {
		let msg = "Hedge order was rejected or canceled";
		error!(msg);
		return Err(DetailedError::internal_error(msg).to_rpc_error());
	}

	// Update loan record with position size, state: HedgeOpened, and populate hedge_open tx and cloid
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.position_size = format!("{}", clamped_hedge_size_f64);
		r.state = LoanState::HedgeOpened;
		r.cloids.push(("hedge_open".to_string(), hedge_open_cloid.to_string()));
		if let Some(ref tx_hash) = hedge_open_tx_hash {
			r.txs.push(("hedge_open".to_string(), tx_hash.clone()));
		}
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Open Hedge")
		.await;

	Ok((hedge_open_cloid, hedge_open_tx_hash))
}
