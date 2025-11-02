use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, INVALID_CHAIN_ID_CODE, PARSE_ERROR_CODE};
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_corewriter_userop;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::AccountId;
use executor_storage::{LoanState, Storage};
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
	pub to_spot_move_tx_hash: Option<String>,
	pub spot_buy_cloid: Option<String>,
	pub spot_buy_tx_hash: Option<String>,
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

struct CloseHedgeContext {
	should_cancel: bool,
	should_close: bool,
	position_size_to_close: f64,
	/// Position data: (unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable)
	position_data: Option<(f64, f64, f64, f64, f64)>,
	perp_asset_id: u32,
	perp_sz_decimals: u8,
	perp_mark_price: f64,
	perp_mid_price: f64,
}

struct MoveToSpotContext {
	transfer_amount: f64,
}

struct BuySpotContext {
	spot_asset_id: u32,
	spot_sz_decimals: u8,
	spot_mark_price: f64,
	spot_mid_price: f64,
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
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.into()
			})?;

			debug!("Received omni_paybackLoanTest, params: {:?}", params);

			let (omni_account, loan_nonce, min_expected_account_value_f64) =
				precheck_params(&params)?;

			let ctx = Arc::clone(&ctx);
			let smart_wallet = &params.user_operation.sender;
			let storage_key = executor_storage::loan_record::Key {
				account_id: omni_account.clone(),
				nonce: loan_nonce,
			};

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|e| {
				error!("Failed to create HyperCore client: {}", e);
				DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
					.with_reason(format!("Chain ID {} is not supported", params.chain_id))
					.into()
			})?;

			hypercore_client.print_account_state(smart_wallet, "Before Payback").await;

			// Retrieve and parse loan record
			let loan_record = ctx
				.loan_record_storage
				.get(&storage_key)
				.map_err(|_| {
					error!("Failed to retrieve loan record from storage");
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to retrieve loan record from storage")
						.into()
				})?
				.ok_or_else(|| {
					error!("Loan record not found for nonce {}", loan_nonce);
					DetailedError::new(INTERNAL_ERROR_CODE, "Loan record not found")
						.with_reason(format!("No loan record found for nonce {}", loan_nonce))
						.into()
				})?;

			info!("Retrieved loan record: {:?}", loan_record);

			let collateral_ticker = loan_record.collateral_ticker.to_uppercase();
			let collateral_size = loan_record.collateral_size.parse::<f64>().map_err(|e| {
				DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
					.with_reason(format!("Invalid collateral_size: {}", e))
					.into()
			})?;
			let usdc_sold = loan_record.usdc_sold.parse::<f64>().map_err(|e| {
				DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
					.with_reason(format!("Invalid usdc_sold: {}", e))
					.into()
			})?;
			let usdc_loaned = loan_record.usdc_loaned.parse::<f64>().map_err(|e| {
				DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
					.with_reason(format!("Invalid usdc_loaned: {}", e))
					.into()
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
			let mut current_nonce = exec_ctx.skeleton_user_op.nonce;

			match loan_record.state {
				LoanState::HedgeOpened => {
					info!("Starting payback from HedgeOpened state");

					let hedge_open_cloid_str = loan_record
						.cloids
						.iter()
						.find(|(name, _)| name == "hedge_open")
						.map(|(_, cloid)| cloid.clone())
						.ok_or_else(|| {
							error!("hedge_open cloid not found in loan record");
							DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
								.with_reason("hedge_open cloid not found in loan_record")
						})?;
					let hedge_open_cloid = hedge_open_cloid_str.parse::<u128>().map_err(|e| {
						DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
							.with_reason(format!("Invalid hedge_open_cloid: {}", e))
							.into()
					})?;

					let close_ctx = precheck_close_hedge(
						&ctx,
						&hypercore_client,
						smart_wallet,
						&collateral_ticker,
						hedge_open_cloid,
						min_expected_account_value_f64,
						&loan_record,
					)
					.await?;

					let move_ctx = precheck_move_to_spot(
						&hypercore_client,
						smart_wallet,
						usdc_sold,
						usdc_loaned,
						&close_ctx.position_data,
					)
					.await?;

					let buy_ctx =
						precheck_buy_spot(&hypercore_client, &collateral_ticker, collateral_size)
							.await?;

					let (hedge_cancel_tx_hash, hedge_close_cloid_opt, hedge_close_tx_hash) =
						do_close_hedge(&exec_ctx, hedge_open_cloid, &close_ctx, &mut current_nonce)
							.await?;

					let to_spot_move_tx_hash =
						do_move_to_spot(&exec_ctx, &move_ctx, &mut current_nonce).await?;

					let (spot_buy_cloid, spot_buy_tx_hash) =
						do_buy_spot(&exec_ctx, collateral_size, &buy_ctx, current_nonce).await?;

					Ok(PaybackLoanTestResponse {
						collateral_ticker,
						collateral_size: collateral_size.to_string(),
						hedge_cancel_tx_hash,
						hedge_close_cloid: hedge_close_cloid_opt,
						hedge_close_tx_hash,
						to_spot_move_tx_hash,
						spot_buy_cloid: Some(spot_buy_cloid.to_string()),
						spot_buy_tx_hash,
					})
				},
				LoanState::HedgeClosed | LoanState::ToPerpMoved => {
					info!("Resuming from HedgeClosed or ToPerpMoved state");

					let move_ctx = precheck_move_to_spot(
						&hypercore_client,
						smart_wallet,
						usdc_sold,
						usdc_loaned,
						&None,
					)
					.await?;

					let buy_ctx =
						precheck_buy_spot(&hypercore_client, &collateral_ticker, collateral_size)
							.await?;

					let to_spot_move_tx_hash =
						do_move_to_spot(&exec_ctx, &move_ctx, &mut current_nonce).await?;

					let (spot_buy_cloid, spot_buy_tx_hash) =
						do_buy_spot(&exec_ctx, collateral_size, &buy_ctx, current_nonce).await?;

					Ok(PaybackLoanTestResponse {
						collateral_ticker,
						collateral_size: collateral_size.to_string(),
						hedge_cancel_tx_hash: None,
						hedge_close_cloid: None,
						hedge_close_tx_hash: None,
						to_spot_move_tx_hash,
						spot_buy_cloid: Some(spot_buy_cloid.to_string()),
						spot_buy_tx_hash,
					})
				},
				LoanState::ToSpotMoved | LoanState::SpotSold => {
					info!("Resuming from ToSpotMoved or SpotSold state");

					let buy_ctx =
						precheck_buy_spot(&hypercore_client, &collateral_ticker, collateral_size)
							.await?;

					let (spot_buy_cloid, spot_buy_tx_hash) =
						do_buy_spot(&exec_ctx, collateral_size, &buy_ctx, current_nonce).await?;

					Ok(PaybackLoanTestResponse {
						collateral_ticker,
						collateral_size: collateral_size.to_string(),
						hedge_cancel_tx_hash: None,
						hedge_close_cloid: None,
						hedge_close_tx_hash: None,
						to_spot_move_tx_hash: None,
						spot_buy_cloid: Some(spot_buy_cloid.to_string()),
						spot_buy_tx_hash,
					})
				},
				LoanState::SpotBought => {
					error!("Payback already completed");
					Err(DetailedError::new(INTERNAL_ERROR_CODE, "Payback already completed").into())
				},
			}
		})
		.expect("Failed to register omni_paybackLoanTest method");
}

fn precheck_params(params: &PaybackLoanTestParams) -> Result<(AccountId, u64, f64)> {
	let omni_account = to_omni_account(&params.omni_account)
		.map_err(|_| DetailedError::new(PARSE_ERROR_CODE, "Invalid omni account").into())?;

	let min_expected_account_value_f64 =
		params.min_expected_account_value.parse::<f64>().map_err(|e| {
			error!("Failed to parse min_expected_account_value: {}", e);
			DetailedError::new(PARSE_ERROR_CODE, "Invalid parameter")
				.with_reason(format!("Invalid min_expected_account_value value: {}", e))
				.into()
		})?;

	info!("Minimum expected account value threshold: {} USDC", min_expected_account_value_f64);

	Ok((omni_account, params.loan_nonce, min_expected_account_value_f64))
}

/// Helper to prepare UserOp with custom nonce, always clearing init_code
/// (payback_loan never needs init_code as account is already created)
fn prepare_userop_no_init(
	base: &SerializablePackedUserOperation,
	nonce: u128,
) -> SerializablePackedUserOperation {
	let mut op = base.clone();
	op.nonce = nonce;
	op.init_code = "0x".to_string(); // Always clear init_code for payback
	op
}

/// Helper to verify hedge position exists and is not liquidated
/// Returns: (position_size, account_value, unrealized_pnl, cum_funding_all_time, margin_used, position_value, withdrawable)
/// where account_value = crossMarginSummary.accountValue
async fn verify_hedge_position(
	hypercore_client: &HyperCoreClient,
	smart_wallet_address: &str,
	collateral_ticker: &str,
) -> Result<(f64, f64, f64, f64, f64, f64, f64)> {
	let perp_state = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address)
		.await
		.map_err(|e| {
			error!("Failed to get perp state: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query perp state: {}", e))
				.into()
		})?;

	let hedge_position = perp_state
		.asset_positions
		.iter()
		.find(|pos| pos.position.coin.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_else(|| {
			error!("Hedge position for {} not found", collateral_ticker);
			DetailedError::new(INTERNAL_ERROR_CODE, "Position not found")
				.with_reason(format!("Position for {} not found or liquidated", collateral_ticker))
				.into()
		})?;

	let position_size = hedge_position.position.szi.parse::<f64>().map_err(|e| {
		error!("Failed to parse position size: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Invalid position size: {}", e))
			.into()
	})?;

	if position_size <= 0.0 {
		error!("Position liquidated (size: {})", position_size);
		return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Position liquidated")
			.with_reason(format!("Position for {} has been liquidated", collateral_ticker))
			.into());
	}

	// Get account value from crossMarginSummary
	let account_value =
		perp_state.cross_margin_summary.account_value.parse::<f64>().map_err(|e| {
			error!("Failed to parse account value: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid account value: {}", e))
				.into()
		})?;

	// Parse additional position data
	let unrealized_pnl = hedge_position.position.unrealized_pnl.parse::<f64>().map_err(|e| {
		error!("Failed to parse unrealized PnL: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Invalid unrealized PnL: {}", e))
			.into()
	})?;

	let cum_funding_all_time =
		hedge_position.position.cum_funding.all_time.parse::<f64>().map_err(|e| {
			error!("Failed to parse cumulative funding: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Invalid cumulative funding: {}", e))
				.into()
		})?;

	let margin_used = hedge_position.position.margin_used.parse::<f64>().map_err(|e| {
		error!("Failed to parse margin used: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Invalid margin used: {}", e))
			.into()
	})?;

	let position_value = hedge_position.position.position_value.parse::<f64>().map_err(|e| {
		error!("Failed to parse position value: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Invalid position value: {}", e))
			.into()
	})?;

	let withdrawable = perp_state.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Invalid withdrawable: {}", e))
			.into()
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

async fn precheck_close_hedge<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	_ctx: &RpcContext<CrossChainIntentExecutor>,
	hypercore_client: &HyperCoreClient,
	smart_wallet: &str,
	collateral_ticker: &str,
	hedge_open_cloid: u128,
	min_expected_account_value_f64: f64,
	loan_record: &executor_storage::loan_record::LoanRecord,
) -> Result<CloseHedgeContext> {
	// Validate loan state is HedgeOpened
	if loan_record.state != LoanState::HedgeOpened {
		error!("Invalid loan state for payback: expected HedgeOpened, got {:?}", loan_record.state);
		return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Invalid loan state")
			.with_reason(format!(
				"Loan must be in HedgeOpened state for payback, current state: {:?}",
				loan_record.state
			))
			.into());
	}

	let usdc_loaned = loan_record.usdc_loaned.parse::<f64>().map_err(|e| {
		DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
			.with_reason(format!("Invalid usdc_loaned: {}", e))
			.into()
	})?;

	let loan_position_size = loan_record.position_size.parse::<f64>().map_err(|e| {
		DetailedError::new(INTERNAL_ERROR_CODE, "Invalid stored data")
			.with_reason(format!("Invalid position_size: {}", e))
			.into()
	})?;

	info!("Loan record position size (to be closed): {}", loan_position_size);

	// Check USDC balance in spot account
	info!("Checking USDC balance in spot account...");
	let usdc_balance =
		hypercore_client.get_spot_balance(smart_wallet, "USDC").await.map_err(|e| {
			error!("Failed to get USDC balance: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query USDC balance: {}", e))
				.into()
		})?;

	info!("User USDC balance: {}, loan amount: {}", usdc_balance, usdc_loaned);

	if usdc_balance < usdc_loaned {
		error!(
			"Insufficient USDC balance: user has {} but needs {} to pay back loan",
			usdc_balance, usdc_loaned
		);
		return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Insufficient USDC balance")
			.with_reason(format!(
				"User has {} USDC but needs {} USDC to pay back the loan",
				usdc_balance, usdc_loaned
			))
			.into());
	}

	info!("✓ USDC balance check passed: user has sufficient USDC");

	// Get perp market data
	let (perp_meta, perp_mark_price, perp_mid_price) =
		hypercore_client.get_perp_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get perp market prices for {}: {}", collateral_ticker, e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to get perp market prices: {}", e))
				.into()
		})?;

	let perp_asset_id = get_perp_asset_id(collateral_ticker, &perp_meta).map_err(|e| {
		error!("Failed to get perp asset ID: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e).into()
	})?;

	let perp_asset = perp_meta.universe.get(perp_asset_id as usize).ok_or_else(|| {
		error!("Perp asset {} not found in meta", perp_asset_id);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Perp asset {} not found", perp_asset_id))
			.into()
	})?;

	let perp_sz_decimals = perp_asset.sz_decimals;

	// Check hedge order status
	info!("Checking hedge order status with hedge_open_cloid {}", hedge_open_cloid);

	let order_status = hypercore_client
		.get_order_status(smart_wallet, &hedge_open_cloid.to_string())
		.await
		.map_err(|e| {
			error!("Failed to get order status for cloid {}: {}", hedge_open_cloid, e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Failed to get order status")
				.with_reason(format!(
					"Cannot retrieve order status for cloid {}: {}",
					hedge_open_cloid, e
				))
				.into()
		})?;

	let mut position_data: Option<(f64, f64, f64, f64, f64)> = None;

	// If the order is not found (unknownOid), skip closing
	if order_status.status == "unknownOid" {
		info!("Order with cloid {} not found (unknownOid), skipping close", hedge_open_cloid);
		return Ok(CloseHedgeContext {
			should_cancel: false,
			should_close: false,
			position_size_to_close: 0.0,
			position_data: None,
			perp_asset_id,
			perp_sz_decimals,
			perp_mark_price,
			perp_mid_price,
		});
	}

	let (should_cancel, should_close, position_size_to_close) = if let Some(order_info) =
		order_status.order
	{
		let status = order_info.status.as_str();
		info!("Hedge order status: {}", status);

		match status {
			"filled" => {
				info!("Case 1: Order fully filled, will close position");
				let (
					actual_position_size,
					account_value,
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				) = verify_hedge_position(hypercore_client, smart_wallet, collateral_ticker).await?;

				if actual_position_size < loan_position_size {
					error!(
						"Actual position size ({}) is less than loan position size ({})",
						actual_position_size, loan_position_size
					);
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Position size mismatch")
						.with_reason(format!(
							"Actual position size ({}) is less than expected from loan record ({})",
							actual_position_size, loan_position_size
						))
						.into());
				}
				info!(
					"✓ Position size check passed: actual {} >= loan record {}",
					actual_position_size, loan_position_size
				);

				if account_value < min_expected_account_value_f64 {
					error!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Account value too low").with_reason(
							format!(
								"Account value ({} USDC) is below minimum expected ({} USDC). Refusing to close position with unexpected loss.",
								account_value, min_expected_account_value_f64
							),
						).into()
					);
				}
				info!(
					"✓ Account value check passed: {} USDC >= {} USDC",
					account_value, min_expected_account_value_f64
				);

				position_data = Some((
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				));

				(false, true, loan_position_size)
			},
			"open" => {
				info!("Case 2: Order not filled, will cancel");
				(true, false, 0.0)
			},
			"partial_fill" => {
				info!(
					"Case 3: Order partially filled, will cancel order and close filled position"
				);
				let (
					actual_position_size,
					account_value,
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				) = verify_hedge_position(hypercore_client, smart_wallet, collateral_ticker).await?;

				if actual_position_size < loan_position_size {
					info!(
						"Partial fill: actual position size ({}) is less than expected ({}), will close actual size",
						actual_position_size, loan_position_size
					);
				} else {
					info!(
						"✓ Position size check: actual {} >= loan record {}",
						actual_position_size, loan_position_size
					);
				}

				if account_value < min_expected_account_value_f64 {
					error!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Account value too low").with_reason(
							format!(
								"Account value ({} USDC) is below minimum expected ({} USDC). Refusing to close position with unexpected loss.",
								account_value, min_expected_account_value_f64
							),
						).into()
					);
				}
				info!(
					"✓ Account value check passed: {} USDC >= {} USDC",
					account_value, min_expected_account_value_f64
				);

				position_data = Some((
					unrealized_pnl,
					cum_funding_all_time,
					margin_used,
					position_value,
					withdrawable,
				));

				let position_to_close = loan_position_size.min(actual_position_size);
				(true, true, position_to_close)
			},
			"canceled" | "rejected" | "expired" => {
				error!("Order already in terminal state: {}", status);
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Order not active")
					.with_reason(format!("Order is already {}", status))
					.into());
			},
			_ => {
				error!("Unexpected order status: {}", status);
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Unexpected order status")
					.with_reason(format!("Unknown status: {}", status))
					.into());
			},
		}
	} else {
		error!("Order not found for cloid {}", hedge_open_cloid);
		return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Order not found")
			.with_reason(format!("No order found with cloid {}", hedge_open_cloid))
			.into());
	};

	// Validate notional value if closing position
	if should_close && position_size_to_close > 0.0 {
		let clamped_size = clamp_size(position_size_to_close, perp_sz_decimals);
		let clamped_size_f64 = clamped_size.parse::<f64>().map_err(|e| {
			error!("Failed to parse clamped close size: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped close size: {}", e))
				.into()
		})?;

		// For closing long position (selling), we want to sell at the highest buy price (bid)
		let (perp_bid_price, _) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
		validate_notional_value(perp_bid_price, clamped_size_f64, "Perp close").map_err(|e| {
			error!("{}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Notional value too low")
				.with_reason(e)
				.into()
		})?;
	}

	info!("✓ Close position precheck passed");

	Ok(CloseHedgeContext {
		should_cancel,
		should_close,
		position_size_to_close,
		position_data,
		perp_asset_id,
		perp_sz_decimals,
		perp_mark_price,
		perp_mid_price,
	})
}
async fn precheck_move_to_spot(
	hypercore_client: &HyperCoreClient,
	smart_wallet: &str,
	usdc_sold: f64,
	usdc_loaned: f64,
	position_data: &Option<(f64, f64, f64, f64, f64)>,
) -> Result<MoveToSpotContext> {
	let initial_margin = usdc_sold - usdc_loaned;

	let perp_state =
		hypercore_client.get_perp_clearinghouse_state(smart_wallet).await.map_err(|e| {
			error!("Failed to get perp state: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query perp state: {}", e))
				.into()
		})?;

	let withdrawable_usdc = perp_state.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable USDC: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Failed to parse withdrawable USDC: {}", e))
			.into()
	})?;

	info!("Current withdrawable USDC from perp: {}", withdrawable_usdc);

	let transfer_amount = if let Some((
		unrealized_pnl,
		cum_funding_all_time,
		margin_used,
		position_value,
		stored_withdrawable,
	)) = position_data
	{
		let close_position_fee = position_value * 0.00045;
		let buffer = 0.01;
		let calculated_amount =
			initial_margin + unrealized_pnl - cum_funding_all_time - close_position_fee - buffer;

		let max_transferable = stored_withdrawable + margin_used;
		if calculated_amount > max_transferable {
			error!(
				"Calculated transfer amount ({}) exceeds max transferable ({} = {} withdrawable + {} margin_used)",
				calculated_amount, max_transferable, stored_withdrawable, margin_used
			);
			return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Invalid transfer amount")
				.with_reason(format!(
					"Calculated transfer amount ({}) exceeds available funds ({})",
					calculated_amount, max_transferable
				))
				.into());
		}

		info!(
			"Transfer amount calculation: initial_margin={}, unrealized_pnl={}, cum_funding_all_time={}, close_fee={}, buffer={}, withdrawable={}, margin_used={}, calculated={}",
			initial_margin, unrealized_pnl, cum_funding_all_time, close_position_fee, buffer, stored_withdrawable, margin_used, calculated_amount
		);

		calculated_amount
	} else {
		info!("No position closed, transferring initial margin: {}", initial_margin);
		initial_margin
	};

	info!("✓ Move to spot precheck passed");

	Ok(MoveToSpotContext { transfer_amount })
}

async fn precheck_buy_spot(
	hypercore_client: &HyperCoreClient,
	collateral_ticker: &str,
	collateral_size: f64,
) -> Result<BuySpotContext> {
	let (spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(collateral_ticker).await.map_err(|e| {
			error!("Failed to get spot market prices for {}: {}", collateral_ticker, e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to get spot market prices: {}", e))
				.into()
		})?;

	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta).map_err(|e| {
		error!("Failed to get spot asset ID: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(e)
	})?;

	let spot_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_else(|| {
			error!("Token {} not found in spot meta", collateral_ticker);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Token {} not found in spot meta", collateral_ticker))
				.into()
		})?;

	let spot_sz_decimals = spot_token.sz_decimals;

	// Validate notional value with clamped size
	let clamped_size = clamp_size(collateral_size, spot_sz_decimals);
	let clamped_size_f64 = clamped_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy size: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Failed to parse clamped buy size: {}", e))
			.into()
	})?;

	// For buying, we want to buy at the lowest sell price (ask)
	let (_, spot_ask_price) = get_bid_ask_prices(spot_mark_price, spot_mid_price);
	validate_notional_value(spot_ask_price, clamped_size_f64, "Spot buy").map_err(|e| {
		error!("{}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Notional value too low")
			.with_reason(e)
			.into()
	})?;

	info!("✓ Buy spot precheck passed");

	Ok(BuySpotContext { spot_asset_id, spot_sz_decimals, spot_mark_price, spot_mid_price })
}

async fn do_close_hedge<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	hedge_open_cloid: u128,
	close_ctx: &CloseHedgeContext,
	current_nonce: &mut u128,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
	let mut hedge_cancel_tx_hash: Option<String> = None;
	let mut hedge_close_cloid_opt: Option<String> = None;
	let mut hedge_close_tx_hash: Option<String> = None;

	// Action 1a: Cancel order if needed
	if close_ctx.should_cancel {
		info!("Action: Canceling unfilled order...");

		let cancel_action = build_cancel_order_by_cloid(close_ctx.perp_asset_id, hedge_open_cloid);
		let cancel_calldata = encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(cancel_action),
		);

		hedge_cancel_tx_hash = submit_corewriter_userop(
			exec_ctx.ctx.clone(),
			exec_ctx.omni_account,
			&prepare_userop_no_init(exec_ctx.skeleton_user_op, *current_nonce),
			exec_ctx.chain_id,
			exec_ctx.wallet_index,
			cancel_calldata,
		)
		.await?;

		info!("hedge_cancel submitted with tx_hash: {:?}", hedge_cancel_tx_hash);
		*current_nonce += 1;

		let order_canceled = exec_ctx
			.hypercore_client
			.wait_for_order(
				exec_ctx.smart_wallet,
				&hedge_open_cloid.to_string(),
				30,
				OrderWaitCondition::Canceled,
			)
			.await
			.map_err(|e| {
				error!("Order cancel did not complete: {}", e);
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Order cancel failed: {}", e))
					.into()
			})?;

		if !order_canceled {
			error!("Order cancel was rejected or expired");
			return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason("Order cancel was rejected or expired")
				.into());
		}

		info!("Order canceled successfully");
		exec_ctx
			.hypercore_client
			.print_account_state(exec_ctx.smart_wallet, "After Order Canceled")
			.await;
	}

	// Action 1b: Close position if needed
	if close_ctx.should_close {
		info!("Action: Closing hedge position...");

		let hedge_close_cloid = generate_cloid();
		hedge_close_cloid_opt = Some(hedge_close_cloid.to_string());
		let close_size_abs = close_ctx.position_size_to_close.abs();
		let clamped_close_size = clamp_size(close_size_abs, close_ctx.perp_sz_decimals);
		// For closing long position (selling), we want to sell at the highest buy price (bid)
		let (perp_bid_price, _perp_ask_price) =
			get_bid_ask_prices(close_ctx.perp_mark_price, close_ctx.perp_mid_price);
		let target_close_price = perp_bid_price * PERP_CLOSE_PRICE_RATIO;
		let clamped_close_price =
			clamp_price(target_close_price, close_ctx.perp_sz_decimals, false);

		info!(
			"Perp close pricing - markPx: {}, midPx: {}, bid (highest buy): {}, target (with {}x buffer): {}, clamped: {}",
			close_ctx.perp_mark_price, close_ctx.perp_mid_price, perp_bid_price, PERP_CLOSE_PRICE_RATIO, target_close_price, clamped_close_price
		);

		let clamped_close_size_f64 = clamped_close_size.parse::<f64>().map_err(|e| {
			error!("Failed to parse clamped close size: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped close size: {}", e))
				.into()
		})?;
		let clamped_close_price_f64 = clamped_close_price.parse::<f64>().map_err(|e| {
			error!("Failed to parse clamped close price: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to parse clamped close price: {}", e))
				.into()
		})?;

		let close_size_units = to_price_units(clamped_close_size_f64);
		let close_price_units = to_price_units(clamped_close_price_f64);

		let close_action = build_perp_close_order(
			close_ctx.perp_asset_id,
			close_size_units,
			close_price_units,
			hedge_close_cloid,
		);
		let close_calldata = encode_omni_account_execute(
			get_core_writer_address(),
			encode_send_raw_action(close_action),
		);

		hedge_close_tx_hash = submit_corewriter_userop(
			exec_ctx.ctx.clone(),
			exec_ctx.omni_account,
			&prepare_userop_no_init(exec_ctx.skeleton_user_op, *current_nonce),
			exec_ctx.chain_id,
			exec_ctx.wallet_index,
			close_calldata,
		)
		.await?;
		*current_nonce += 1;

		info!(
			"hedge_close submitted, price: {}, size: {}, tx_hash: {:?}",
			clamped_close_price_f64, clamped_close_size_f64, hedge_close_tx_hash
		);

		let order_filled = exec_ctx
			.hypercore_client
			.wait_for_order(
				exec_ctx.smart_wallet,
				&hedge_close_cloid.to_string(),
				30,
				OrderWaitCondition::Filled,
			)
			.await
			.map_err(|e| {
				error!("Hedge close order did not complete: {}", e);
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!("Hedge close order failed: {}", e))
					.into()
			})?;

		if !order_filled {
			error!("Hedge close order was rejected or canceled");
			return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason("Hedge close order was rejected or canceled")
				.into());
		}

		info!("Hedge position closed successfully");
		exec_ctx
			.hypercore_client
			.print_account_state(exec_ctx.smart_wallet, "After Hedge Closed")
			.await;
	}

	// Update state: HedgeClosed (if cancel or close happened) and populate txs/cloids
	if close_ctx.should_cancel || close_ctx.should_close {
		let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
			r.state = LoanState::HedgeClosed;
			if let Some(ref tx_hash) = hedge_cancel_tx_hash {
				r.txs.push(("hedge_cancel".to_string(), tx_hash.clone()));
			}
			if let Some(ref cloid) = hedge_close_cloid_opt {
				r.cloids.push(("hedge_close".to_string(), cloid.clone()));
			}
			if let Some(ref tx_hash) = hedge_close_tx_hash {
				r.txs.push(("hedge_close".to_string(), tx_hash.clone()));
			}
		});
	}

	Ok((hedge_cancel_tx_hash, hedge_close_cloid_opt, hedge_close_tx_hash))
}

async fn do_move_to_spot<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	move_ctx: &MoveToSpotContext,
	current_nonce: &mut u128,
) -> Result<Option<String>> {
	info!("Action: Transferring USDC from perp to spot...");

	// Get initial spot balance BEFORE submitting the transfer
	let initial_spot_usdc = exec_ctx
		.hypercore_client
		.get_spot_balance(exec_ctx.smart_wallet, "USDC")
		.await
		.map_err(|e| {
			error!("Failed to get initial spot USDC balance: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query spot USDC balance: {}", e))
				.into()
		})?;

	let perp_state = exec_ctx
		.hypercore_client
		.get_perp_clearinghouse_state(exec_ctx.smart_wallet)
		.await
		.map_err(|e| {
			error!("Failed to get perp state: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query perp state: {}", e))
				.into()
		})?;

	let withdrawable_usdc = perp_state.withdrawable.parse::<f64>().map_err(|e| {
		error!("Failed to parse withdrawable USDC: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Failed to parse withdrawable USDC: {}", e))
			.into()
	})?;

	let final_amount = move_ctx.transfer_amount.min(withdrawable_usdc);
	info!(
		"Calculated transfer_amount={}, withdrawable_usdc={}, final_amount={}",
		move_ctx.transfer_amount, withdrawable_usdc, final_amount
	);

	let transfer_amount_units = to_usdc_units(final_amount);
	let transfer_action = build_usd_class_transfer_to_spot(transfer_amount_units);
	let transfer_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(transfer_action),
	);

	let to_spot_move_tx_hash = submit_corewriter_userop(
		exec_ctx.ctx.clone(),
		exec_ctx.omni_account,
		&prepare_userop_no_init(exec_ctx.skeleton_user_op, *current_nonce),
		exec_ctx.chain_id,
		exec_ctx.wallet_index,
		transfer_calldata,
	)
	.await?;
	*current_nonce += 1;

	info!("to_spot_move submitted, size: {}, tx_hash: {:?}", final_amount, to_spot_move_tx_hash);

	exec_ctx
		.hypercore_client
		.wait_for_spot_balance_increase(
			exec_ctx.smart_wallet,
			"USDC",
			initial_spot_usdc,
			final_amount,
			30,
		)
		.await
		.map_err(|e| {
			error!("USD transfer to spot failed: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Transfer timeout")
				.with_reason(e)
				.into()
		})?;

	info!("to_spot_move completed");

	// Update state: ToSpotMoved and populate to_spot_move tx
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.state = LoanState::ToSpotMoved;
		if let Some(ref tx_hash) = to_spot_move_tx_hash {
			r.txs.push(("to_spot_move".to_string(), tx_hash.clone()));
		}
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After USD Transfer to Spot")
		.await;

	Ok(to_spot_move_tx_hash)
}

async fn do_buy_spot<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	collateral_size: f64,
	buy_ctx: &BuySpotContext,
	current_nonce: u128,
) -> Result<(u128, Option<String>)> {
	info!("Action: Buying back collateral in spot market...");

	// Get current USDC balance to determine how much collateral we can afford
	let current_spot_usdc = exec_ctx
		.hypercore_client
		.get_spot_balance(exec_ctx.smart_wallet, "USDC")
		.await
		.map_err(|e| {
			error!("Failed to get current spot USDC balance: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Failed to query spot USDC balance: {}", e))
				.into()
		})?;

	info!("Current spot USDC balance: {} USDC", current_spot_usdc);

	let spot_buy_cloid = generate_cloid();
	// For buying, we want to buy at the lowest sell price (ask)
	let (_spot_bid_price, spot_ask_price) =
		get_bid_ask_prices(buy_ctx.spot_mark_price, buy_ctx.spot_mid_price);
	let target_buy_price = spot_ask_price * SPOT_BUY_PRICE_RATIO;
	let clamped_buy_price = clamp_price(target_buy_price, buy_ctx.spot_sz_decimals, true);

	let clamped_buy_price_f64 = clamped_buy_price.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy price: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Failed to parse clamped buy price: {}", e))
			.into()
	})?;

	let affordable_collateral = current_spot_usdc / clamped_buy_price_f64;
	let actual_buy_size = collateral_size.min(affordable_collateral);
	let clamped_buy_size = clamp_size(actual_buy_size, buy_ctx.spot_sz_decimals);

	info!(
		"Spot buy calculation - desired: {}, affordable: {}, using: {} (clamped: {})",
		collateral_size, affordable_collateral, actual_buy_size, clamped_buy_size
	);

	info!(
		"Spot buy pricing - markPx: {}, midPx: {}, ask (lowest sell): {}, target (with {}x buffer): {}, clamped: {}",
		buy_ctx.spot_mark_price, buy_ctx.spot_mid_price, spot_ask_price, SPOT_BUY_PRICE_RATIO, target_buy_price, clamped_buy_price
	);

	let clamped_buy_size_f64 = clamped_buy_size.parse::<f64>().map_err(|e| {
		error!("Failed to parse clamped buy size: {}", e);
		DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason(format!("Failed to parse clamped buy size: {}", e))
			.into()
	})?;

	let buy_size_units = to_price_units(clamped_buy_size_f64);
	let buy_price_units = to_price_units(clamped_buy_price_f64);

	let spot_buy_action = build_spot_buy_order(
		buy_ctx.spot_asset_id,
		buy_size_units,
		buy_price_units,
		spot_buy_cloid,
	);
	let spot_buy_calldata = encode_omni_account_execute(
		get_core_writer_address(),
		encode_send_raw_action(spot_buy_action),
	);

	let spot_buy_tx_hash = submit_corewriter_userop(
		exec_ctx.ctx.clone(),
		exec_ctx.omni_account,
		&prepare_userop_no_init(exec_ctx.skeleton_user_op, current_nonce),
		exec_ctx.chain_id,
		exec_ctx.wallet_index,
		spot_buy_calldata,
	)
	.await?;

	info!(
		"spot_buy submitted, price: {}, size: {}, tx_hash: {:?}",
		clamped_buy_price_f64, clamped_buy_size_f64, spot_buy_tx_hash
	);

	let buy_order_filled = exec_ctx
		.hypercore_client
		.wait_for_order(
			exec_ctx.smart_wallet,
			&spot_buy_cloid.to_string(),
			30,
			OrderWaitCondition::Filled,
		)
		.await
		.map_err(|e| {
			error!("Spot buy order did not complete: {}", e);
			DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
				.with_reason(format!("Spot buy order failed: {}", e))
				.into()
		})?;

	if !buy_order_filled {
		error!("Spot buy order was rejected or canceled");
		return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
			.with_reason("Spot buy order was rejected or canceled")
			.into());
	}

	info!("Spot buy order filled successfully");

	// Update state: SpotBought (final state) and populate spot_buy tx and cloid
	let _ = exec_ctx.ctx.loan_record_storage.update(exec_ctx.storage_key, |r| {
		r.state = LoanState::SpotBought;
		r.cloids.push(("spot_buy".to_string(), spot_buy_cloid.to_string()));
		if let Some(ref tx_hash) = spot_buy_tx_hash {
			r.txs.push(("spot_buy".to_string(), tx_hash.clone()));
		}
	});

	exec_ctx
		.hypercore_client
		.print_account_state(exec_ctx.smart_wallet, "After Spot Buy Complete")
		.await;

	Ok((spot_buy_cloid, spot_buy_tx_hash))
}
