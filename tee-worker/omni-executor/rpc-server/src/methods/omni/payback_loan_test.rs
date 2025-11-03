use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_corewriter_user_ops;
use crate::utils::validation::{parse_as, parse_rpc_params};
use crate::RpcResult;
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
			let params = parse_rpc_params::<PaybackLoanTestParams>(params)?;

			debug!("Received omni_paybackLoanTest, params: {:?}", params);

			let (omni_account, loan_nonce, min_expected_account_value_f64) =
				precheck_params(&params)?;

			let ctx = Arc::clone(&ctx);
			let smart_wallet = &params.user_operation.sender;
			let storage_key = executor_storage::loan_record::Key {
				account_id: omni_account.clone(),
				nonce: loan_nonce,
			};

			let hypercore_client = HyperCoreClient::new(params.chain_id).map_err(|_| {
				DetailedError::internal_error("HyperCore client error").to_rpc_error()
			})?;

			hypercore_client.print_account_state(smart_wallet, "Before Payback").await;

			// Retrieve and parse loan record
			let loan_record = ctx
				.loan_record_storage
				.get(&storage_key)
				.map_err(|_| {
					let msg = format!("Failed to retrieve loan record from storage");
					error!(msg);
					DetailedError::internal_error(&msg).to_rpc_error()
				})?
				.ok_or_else(|| {
					let msg = format!("Loan record not found for nonce {}", loan_nonce);
					error!(msg);
					DetailedError::internal_error(&msg).to_rpc_error()
				})?;

			info!("Retrieved loan record: {:?}", loan_record);

			let collateral_ticker = loan_record.collateral_ticker.to_uppercase();
			let collateral_size: f64 = parse_as(&loan_record.collateral_size, "collateral_size")?;
			let usdc_sold: f64 = parse_as(&loan_record.usdc_sold, "usdc_sold")?;
			let usdc_loaned: f64 = parse_as(&loan_record.usdc_loaned, "usdc_loaned")?;

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
							let msg = format!("hedge_open cloid not found in loan record");
							error!(msg);
							DetailedError::internal_error(&msg).to_rpc_error()
						})?;
					let hedge_open_cloid: u128 =
						parse_as(&hedge_open_cloid_str, "hedge_open_cloid")?;

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
					let msg = format!("Payback already completed");
					error!(msg);
					Err(DetailedError::internal_error(&msg).to_rpc_error())
				},
			}
		})
		.expect("Failed to register omni_paybackLoanTest method");
}

fn precheck_params(params: &PaybackLoanTestParams) -> RpcResult<(AccountId, u64, f64)> {
	let omni_account = to_omni_account(&params.omni_account)
		.map_err(|_| DetailedError::internal_error("Invalid omni account").to_rpc_error())?;

	let min_expected_account_value_f64: f64 =
		parse_as(&params.min_expected_account_value, "min_expected_account_value")?;

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
) -> RpcResult<(f64, f64, f64, f64, f64, f64, f64)> {
	let perp_state = hypercore_client
		.get_perp_clearinghouse_state(smart_wallet_address)
		.await
		.map_err(|e| {
			let msg = format!("Failed to get perp state: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let hedge_position = perp_state
		.asset_positions
		.iter()
		.find(|pos| pos.position.coin.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_else(|| {
			let msg = format!("Hedge position for {} not found", collateral_ticker);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let position_size: f64 = parse_as(&hedge_position.position.szi, "position_size")?;

	if position_size <= 0.0 {
		let msg = format!("Position liquidated (size: {})", position_size);
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	}

	// Get account value from crossMarginSummary
	let account_value: f64 =
		parse_as(&perp_state.cross_margin_summary.account_value, "account_value")?;

	// Parse additional position data
	let unrealized_pnl: f64 = parse_as(&hedge_position.position.unrealized_pnl, "unrealized_pnl")?;

	let cum_funding_all_time: f64 =
		parse_as(&hedge_position.position.cum_funding.all_time, "cum_funding_all_time")?;

	let margin_used: f64 = parse_as(&hedge_position.position.margin_used, "margin_used")?;

	let position_value: f64 = parse_as(&hedge_position.position.position_value, "position_value")?;

	let withdrawable: f64 = parse_as(&perp_state.withdrawable, "withdrawable")?;

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
) -> RpcResult<CloseHedgeContext> {
	// Validate loan state is HedgeOpened
	if loan_record.state != LoanState::HedgeOpened {
		let msg = format!(
			"Invalid loan state for payback: expected HedgeOpened, got {:?}",
			loan_record.state
		);
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	}

	let usdc_loaned: f64 = parse_as(&loan_record.usdc_loaned, "usdc_loaned")?;

	let loan_position_size: f64 = parse_as(&loan_record.position_size, "loan_position_size")?;

	info!("Loan record position size (to be closed): {}", loan_position_size);

	// Check USDC balance in spot account
	info!("Checking USDC balance in spot account...");
	let usdc_balance =
		hypercore_client.get_spot_balance(smart_wallet, "USDC").await.map_err(|e| {
			let msg = format!("Failed to get USDC balance: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	info!("User USDC balance: {}, loan amount: {}", usdc_balance, usdc_loaned);

	if usdc_balance < usdc_loaned {
		let msg = format!("Insufficient USDC balance: {} < {}", usdc_balance, usdc_loaned);
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	}

	info!("✓ USDC balance check passed: user has sufficient USDC");

	// Get perp market data
	let (perp_meta, perp_mark_price, perp_mid_price) =
		hypercore_client.get_perp_market_prices(collateral_ticker).await.map_err(|e| {
			let msg = format!("Failed to get perp market prices for {}: {}", collateral_ticker, e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let perp_asset_id = get_perp_asset_id(collateral_ticker, &perp_meta).map_err(|e| {
		let msg = format!("Failed to get perp asset ID: {}", e);
		error!(msg);
		DetailedError::internal_error(&msg).to_rpc_error()
	})?;

	let perp_asset = perp_meta.universe.get(perp_asset_id as usize).ok_or_else(|| {
		let msg = format!("Perp asset {} not found in meta", perp_asset_id);
		error!(msg);
		DetailedError::internal_error(&msg).to_rpc_error()
	})?;

	let perp_sz_decimals = perp_asset.sz_decimals;

	// Check hedge order status
	info!("Checking hedge order status with hedge_open_cloid {}", hedge_open_cloid);

	let order_status = hypercore_client
		.get_order_status(smart_wallet, &hedge_open_cloid.to_string())
		.await
		.map_err(|e| {
			let msg = format!("Failed to get order status for cloid {}: {}", hedge_open_cloid, e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
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
					let msg = format!(
						"Actual position size ({}) < loan position size ({})",
						actual_position_size, loan_position_size
					);
					error!(msg);
					return Err(DetailedError::internal_error(&msg).to_rpc_error());
				}
				info!(
					"✓ Position size check passed: actual {} >= loan record {}",
					actual_position_size, loan_position_size
				);

				if account_value < min_expected_account_value_f64 {
					let msg = format!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					error!(msg);
					return Err(DetailedError::internal_error(&msg).to_rpc_error());
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
					let msg = format!(
						"Account value ({} USDC) is below minimum expected account value ({} USDC)",
						account_value, min_expected_account_value_f64
					);
					error!(msg);
					return Err(DetailedError::internal_error(&msg).to_rpc_error());
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
				let msg = format!("Order already in terminal state: {}", status);
				error!(msg);
				return Err(DetailedError::internal_error(&msg).to_rpc_error());
			},
			_ => {
				let msg = format!("Unexpected order status: {}", status);
				error!(msg);
				return Err(DetailedError::internal_error(&msg).to_rpc_error());
			},
		}
	} else {
		let msg = format!("Order not found for cloid {}", hedge_open_cloid);
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
	};

	// Validate notional value if closing position
	if should_close && position_size_to_close > 0.0 {
		let clamped_size = clamp_size(position_size_to_close, perp_sz_decimals);
		let clamped_size_f64: f64 = parse_as(&clamped_size, "clamped_close_size")?;

		// For closing long position (selling), we want to sell at the highest buy price (bid)
		let (perp_bid_price, _) = get_bid_ask_prices(perp_mark_price, perp_mid_price);
		validate_notional_value(perp_bid_price, clamped_size_f64, "Perp close").map_err(|e| {
			let msg = format!("Notional value too low: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
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
) -> RpcResult<MoveToSpotContext> {
	let initial_margin = usdc_sold - usdc_loaned;

	let perp_state =
		hypercore_client.get_perp_clearinghouse_state(smart_wallet).await.map_err(|e| {
			let msg = format!("Failed to get perp state: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let withdrawable_usdc: f64 = parse_as(&perp_state.withdrawable, "withdrawable_usdc")?;

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
			let msg = format!(
				"Calculated transfer amount ({}) exceeds max transferable ({} = {} withdrawable + {} margin_used)",
				calculated_amount, max_transferable, stored_withdrawable, margin_used
			);
			error!(msg);
			return Err(DetailedError::internal_error(&msg).to_rpc_error());
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
) -> RpcResult<BuySpotContext> {
	let (spot_meta, spot_mark_price, spot_mid_price) =
		hypercore_client.get_spot_market_prices(collateral_ticker).await.map_err(|e| {
			let msg = format!("Failed to get spot market prices for {}: {}", collateral_ticker, e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta).map_err(|e| {
		let msg = format!("Failed to get spot asset ID: {}", e);
		error!(msg);
		DetailedError::internal_error(&msg).to_rpc_error()
	})?;

	let spot_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(collateral_ticker))
		.ok_or_else(|| {
			let msg = format!("Token {} not found in spot meta", collateral_ticker);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let spot_sz_decimals = spot_token.sz_decimals;

	// Validate notional value with clamped size
	let clamped_size = clamp_size(collateral_size, spot_sz_decimals);
	let clamped_size_f64: f64 = parse_as(&clamped_size, "clamped_buy_size")?;

	// For buying, we want to buy at the lowest sell price (ask)
	let (_, spot_ask_price) = get_bid_ask_prices(spot_mark_price, spot_mid_price);
	validate_notional_value(spot_ask_price, clamped_size_f64, "Spot buy").map_err(|e| {
		let msg = format!("Notional value too low: {}", e);
		error!(msg);
		DetailedError::internal_error(&msg).to_rpc_error()
	})?;

	info!("✓ Buy spot precheck passed");

	Ok(BuySpotContext { spot_asset_id, spot_sz_decimals, spot_mark_price, spot_mid_price })
}

async fn do_close_hedge<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	exec_ctx: &ExecutionContext<'_, CrossChainIntentExecutor>,
	hedge_open_cloid: u128,
	close_ctx: &CloseHedgeContext,
	current_nonce: &mut u128,
) -> RpcResult<(Option<String>, Option<String>, Option<String>)> {
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

		hedge_cancel_tx_hash = submit_corewriter_user_ops(
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
				let msg = format!("Order cancel did not complete: {}", e);
				error!(msg);
				DetailedError::internal_error(&msg).to_rpc_error()
			})?;

		if !order_canceled {
			let msg = "Order cancel was rejected or expired";
			error!(msg);
			return Err(DetailedError::internal_error(&msg).to_rpc_error());
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

		let clamped_close_size_f64: f64 = parse_as(&clamped_close_size, "clamped_close_size")?;
		let clamped_close_price_f64: f64 = parse_as(&clamped_close_price, "clamped_close_price")?;

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

		hedge_close_tx_hash = submit_corewriter_user_ops(
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
				let msg = format!("Hedge close order did not complete: {}", e);
				error!(msg);
				DetailedError::internal_error(&msg).to_rpc_error()
			})?;

		if !order_filled {
			let msg = "Hedge close order was rejected or canceled";
			error!(msg);
			return Err(DetailedError::internal_error(&msg).to_rpc_error());
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
) -> RpcResult<Option<String>> {
	info!("Action: Transferring USDC from perp to spot...");

	// Get initial spot balance BEFORE submitting the transfer
	let initial_spot_usdc = exec_ctx
		.hypercore_client
		.get_spot_balance(exec_ctx.smart_wallet, "USDC")
		.await
		.map_err(|e| {
			let msg = format!("Failed to get initial spot USDC balance: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let perp_state = exec_ctx
		.hypercore_client
		.get_perp_clearinghouse_state(exec_ctx.smart_wallet)
		.await
		.map_err(|e| {
			let msg = format!("Failed to get perp state: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	let withdrawable_usdc: f64 = parse_as(&perp_state.withdrawable, "withdrawable_usdc")?;

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

	let to_spot_move_tx_hash = submit_corewriter_user_ops(
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
			let msg = format!("USD transfer to spot failed: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
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
) -> RpcResult<(u128, Option<String>)> {
	info!("Action: Buying back collateral in spot market...");

	// Get current USDC balance to determine how much collateral we can afford
	let current_spot_usdc = exec_ctx
		.hypercore_client
		.get_spot_balance(exec_ctx.smart_wallet, "USDC")
		.await
		.map_err(|e| {
			let msg = format!("Failed to get current spot USDC balance: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	info!("Current spot USDC balance: {} USDC", current_spot_usdc);

	let spot_buy_cloid = generate_cloid();
	// For buying, we want to buy at the lowest sell price (ask)
	let (_spot_bid_price, spot_ask_price) =
		get_bid_ask_prices(buy_ctx.spot_mark_price, buy_ctx.spot_mid_price);
	let target_buy_price = spot_ask_price * SPOT_BUY_PRICE_RATIO;
	let clamped_buy_price = clamp_price(target_buy_price, buy_ctx.spot_sz_decimals, true);

	let clamped_buy_price_f64: f64 = parse_as(&clamped_buy_price, "clamped_buy_price")?;

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

	let clamped_buy_size_f64: f64 = parse_as(&clamped_buy_size, "clamped_buy_size")?;

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

	let spot_buy_tx_hash = submit_corewriter_user_ops(
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
			let msg = format!("Spot buy order did not complete: {}", e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})?;

	if !buy_order_filled {
		let msg = "Spot buy order was rejected or canceled";
		error!(msg);
		return Err(DetailedError::internal_error(&msg).to_rpc_error());
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
