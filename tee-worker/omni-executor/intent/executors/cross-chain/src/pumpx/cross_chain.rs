// TODO: put it into a mod for potential different providers (other than binance)

use crate::utils::{
	calculate_amount_in, determine_trade_symbol_and_order_side, estimate_asset_value_in_usdt,
	get_binance_deposit_info, get_token_available_amount, str_to_u256,
};
use crate::*;
use ::pumpx::methods::common::SwapType;
use ::pumpx::methods::create_cross_order::{CreateCrossOrderBody, CrossOrderInfo};
use accounting_contract_client::Plus;
use executor_primitives::SwapOrder;
use executor_storage::PumpxProfileStorage;
use heima_primitives::PumpxConfig;
use std::str::FromStr;
use tracing::{debug, error, info};

impl<
		BinanceClient: BinanceApi,
		EthereumClient: EthereumClientTrait + 'static,
		SolanaClient: SolanaClientTrait + 'static,
	> CrossChainIntentExecutor<BinanceClient, EthereumClient, SolanaClient>
{
	pub async fn execute_cross_chain_swap(
		&self,
		account_id: &AccountId,
		omni_account: [u8; 32],
		intent_id: IntentId,
		access_token: &str,
		swap_order: &SwapOrder,
		from_address: String,
		from_wallet: Vec<u8>,
		pumpx_config: &PumpxConfig,
		to_address: String,
	) -> Result<(String, String, Option<InstantFlowDetails>), ()> {
		debug!("executing cross chain swap");

		if pumpx_config.order_type != PumpxOrderType::Market {
			error!("Only market order supported");
			return Err(());
		}

		let from_asset_binance_coin = BinanceAsset::from_chain_asset(&swap_order.from_asset)?.coin;

		let from_amount_decimal = Decimal::from_str(&pumpx_config.from_amount).map_err(|_| {
			error!("Failed to parse from_amount_string");
		})?;

		let usdt_trade_symbol = match &swap_order.from_asset {
			ChainAsset::Solana(_) => "SOLUSDT",
			ChainAsset::Ethereum(_, _) => "BNBUSDT",
		};

		let profile_storage = PumpxProfileStorage::new(self.storage_db.clone());
		let has_exported_wallet = if let Ok(maybe_profile) = profile_storage.get(account_id) {
			maybe_profile.map(|p| p.wallet_exported).unwrap_or(false)
		} else {
			// let's continue but pessimistically assume user has exported his wallet
			error!("Could not get pumpx account profile, assuming wallet has been exported");
			true
		};

		let instant = if has_exported_wallet {
			debug!("Wallet has been exported, skipping instant payout flow");
			false
		} else {
			let estimated_from_amount_in_usdt = estimate_asset_value_in_usdt(
				&self.binance_api,
				usdt_trade_symbol,
				from_asset_binance_coin.name(),
				from_amount_decimal,
			)
			.await?;
			debug!(
				"Checking instant payout :threshold: {:?}, estimated usdt amount: {:?}",
				self.instant_payout_threshold, estimated_from_amount_in_usdt
			);
			let instant = estimated_from_amount_in_usdt <= self.instant_payout_threshold;
			debug!("Instant: {:?}", instant);
			instant
		};

		let amount_to_lock = AmountType::from_str(
			&(from_amount_decimal * from_asset_binance_coin.decimal_value())
				.normalize()
				.to_string(),
		)
		.unwrap();

		if instant {
			let available_amount = get_token_available_amount(
				&swap_order.from_asset,
				from_wallet,
				&self.rpc_endpoint_registry,
				&self.solana_client,
			)
			.await?;

			self.account_asset_lock.check_and_insert(
				account_id.clone(),
				swap_order.from_asset.clone(),
				amount_to_lock,
				available_amount,
			)?;
		}

		self.pumpx_create_cross_order(
			intent_id,
			pumpx_config.from_token_ca.clone(),
			pumpx_config.to_token_ca.clone(),
			pumpx_config.from_amount.clone(),
			pumpx_config.usd_worth.clone(),
			from_address.clone(),
			access_token,
			pumpx_config,
		)
		.await?;

		match (&swap_order.from_asset, &swap_order.to_asset) {
			// SOL to BSC
			(ChainAsset::Solana(_), ChainAsset::Ethereum(::pumpx::constants::BSC_CHAIN_ID, _)) => {
				let payout_address = Address::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("cross chain swap details: intent_id: {}, from_token_ca: {}, to_token_ca: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, pumpx_config.from_token_ca, pumpx_config.to_token_ca, pumpx_config.from_amount, from_address, payout_address);

				if instant {
					// todo: can we reuse existing code ?

					let (_, binance_asset, from_amount_decimal, _) = get_binance_deposit_info(
						self.binance_api.clone(),
						&swap_order.from_asset,
						&pumpx_config.from_amount,
					)
					.await?;

					let binance_network = binance_asset.network;
					let binance_coin = binance_asset.coin;

					let (trade_symbol, _) = determine_trade_symbol_and_order_side(
						binance_network.clone(),
						binance_coin.clone(),
						BinanceNetwork::Bsc,
					)?;

					// init `payout_amount` with estimated-amount-to-receive
					let payout_amount = estimate_payout_amount(
						&self.binance_api,
						&trade_symbol,
						binance_coin.clone(),
						from_amount_decimal,
					)
					.await?;

					Ok((
						self.apply_gas_fee(
							&payout_amount,
							BinanceCoin::Bnb,
							access_token,
							pumpx_config,
						)
						.await?,
						self.evm_accounting_contract_client.get_signer_address().await.to_string(),
						Some(InstantFlowDetails {
							omni_account,
							from_asset: swap_order.from_asset.clone(),
							from_amount: from_amount_decimal,
							from_address: from_address.clone(),
							wallet_index: pumpx_config.wallet_index,
							locked_amount: amount_to_lock,
						}),
					))
				} else {
					let chain_transfer_client = Arc::new(Box::new(SolanaTransferClient::new(
						self.solana_client.clone(),
						RemoteSolanaSigner::new(
							self.pumpx_signer_client.clone(),
							pumpx_config.wallet_index,
							omni_account,
							Handle::current(),
						),
					)) as Box<dyn ChainTransferClient>);
					let (payout_amount, payout_amount_u256) = self
						.do_binance_swap(
							&self.evm_accounting_contract_client,
							&chain_transfer_client,
							swap_order.from_asset.clone(),
							pumpx_config.from_amount.clone(),
							from_address,
							BinanceNetwork::Bsc,
							BinanceCoin::Bnb,
							false,
						)
						.await?;

					self.do_payout(
						&self.evm_accounting_contract_client,
						payout_address,
						payout_amount_u256,
					)
					.await?;

					Ok((
						self.apply_gas_fee(
							&payout_amount,
							BinanceCoin::Bnb,
							access_token,
							pumpx_config,
						)
						.await?,
						to_address,
						None,
					))
				}
			},
			// BSC to SOL
			(ChainAsset::Ethereum(::pumpx::constants::BSC_CHAIN_ID, _), ChainAsset::Solana(_)) => {
				let payout_address = Pubkey::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("cross chain swap details: intent_id: {}, from_token_ca: {}, to_token_ca: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, pumpx_config.from_token_ca, pumpx_config.to_token_ca, pumpx_config.from_amount, from_address, payout_address);

				let remote_signer = RemoteEvmSigner::new(
					self.pumpx_signer_client.clone(),
					pumpx_config.wallet_index,
					omni_account,
				)
				.await
				.map_err(|err| {
					error!("Failed to create RemoteEvmSigner: {}", err);
				})?;
				let chain_transfer_client = Arc::new(Box::new(EvmTransferClient::new(
					self.bsc_client.clone(),
					remote_signer,
				)) as Box<dyn ChainTransferClient>);
				let (payout_amount, payout_amount_u256) = self
					.do_binance_swap(
						&self.solana_accounting_contract_client,
						&chain_transfer_client,
						swap_order.from_asset.clone(),
						pumpx_config.from_amount.clone(),
						from_address,
						BinanceNetwork::Sol,
						BinanceCoin::Sol,
						false,
					)
					.await?;

				self.do_payout(
					&self.solana_accounting_contract_client,
					payout_address,
					payout_amount_u256,
				)
				.await?;

				Ok((
					self.apply_gas_fee(
						&payout_amount,
						BinanceCoin::Sol,
						access_token,
						pumpx_config,
					)
					.await?,
					to_address,
					None,
				))
			},
			_ => {
				error!(
					"Unsupported cross chain swap from {:?} to {:?}",
					swap_order.from_asset, swap_order.to_asset
				);
				Err(())
			},
		}
	}

	async fn pumpx_create_cross_order(
		&self,
		intent_id: IntentId,
		from_token_ca: String,
		to_token_ca: String,
		from_amount: String,
		usd_worth: String,
		from_address: String,
		access_token: &str,
		pumpx_config: &PumpxConfig,
	) -> Result<(), ()> {
		let body = CreateCrossOrderBody {
			request_id: intent_id,
			chain_id: pumpx_config.to_chain_id,
			token_ca: to_token_ca.clone(),
			swap_type: match pumpx_config.swap_type {
				1 => SwapType::Buy,
				2 => SwapType::Sell,
				_ => {
					error!("Unsupported swap type: {}", pumpx_config.swap_type);
					return Err(());
				},
			},
			is_one_click: pumpx_config.is_one_click,
			cross_info: vec![CrossOrderInfo {
				chain_id: pumpx_config.from_chain_id,
				wallet_index: pumpx_config.wallet_index,
				address: from_address.clone(),
				amount: from_amount.clone(),
				usd: usd_worth,
				token_ca: from_token_ca,
			}],
		};
		debug!("Calling pumpx create_cross_order, body: {:?}", body);
		let response =
			self.pumpx_api.create_cross_order(access_token, body).await.map_err(|_| {
				error!("Failed to create cross order");
			})?;
		debug!("Response create_cross_order: {:?}", response);
		Ok(())
	}

	// TODO: move `should_wait_for_deposit_confirm` to provider config
	async fn do_binance_swap<A, N>(
		&self,
		contract_client: &Arc<Box<dyn AccountingContractApi<A, N>>>,
		chain_transfer_client: &Arc<Box<dyn ChainTransferClient>>,
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		to_network: BinanceNetwork,
		payout_coin: BinanceCoin,
		should_wait_for_deposit_confirm: bool,
	) -> Result<(String, U256), ()>
	where
		A: Send + Sync + std::fmt::Debug + std::marker::Copy,
		N: Plus<u64, Output = N> + std::fmt::Debug,
	{
		let (deposit_address, binance_asset, from_amount_decimal, amount_to_transfer_decimal) =
			get_binance_deposit_info(self.binance_api.clone(), &from_asset, &from_amount).await?;

		let binance_network = binance_asset.network;
		let binance_coin = binance_asset.coin;
		let binance_address = binance_asset.address;

		let (trade_symbol, order_side) = determine_trade_symbol_and_order_side(
			binance_network.clone(),
			binance_coin.clone(),
			to_network,
		)?;

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_payout_amount(
			&self.binance_api,
			&trade_symbol,
			binance_coin.clone(),
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, payout_coin.decimals())?;

		// Fetch contract balance
		let balance = contract_client.get_balance().await?;
		if balance < payout_amount_u256 {
			error!(
				"There is not enough balance in the accounting contract, {} < {}",
				balance, payout_amount_u256
			);
			return Err(());
		}

		let amount_to_transfer = decimal_to_u256(amount_to_transfer_decimal).map_err(|err| {
			error!("Failed to convert amount_to_transfer_decimal to U256: {:?}", err);
		})?;

		// Handle transfer based on network and coin type
		let tx_id: Option<String> = match (binance_network.clone(), binance_coin.clone()) {
			(BinanceNetwork::Sol, BinanceCoin::Sol) | (BinanceNetwork::Bsc, BinanceCoin::Bnb) => {
				debug!(
					"Transfering {:?} {:?} to {:?}",
					amount_to_transfer,
					binance_coin.clone(),
					deposit_address
				);
				let signature = chain_transfer_client
					.transfer_native(&deposit_address, amount_to_transfer)
					.await?;
				Some(signature)
			},
			_ => {
				debug!(
					"Transfering {:?} {:?} to {:?}",
					amount_to_transfer, binance_address, deposit_address
				);
				let signature = chain_transfer_client
					.transfer(&deposit_address, amount_to_transfer, &binance_address)
					.await?;
				Some(signature)
			},
		};

		if should_wait_for_deposit_confirm {
			let result = Self::wait_for_deposit_confirm(
				from_address,
				tx_id.clone(),
				binance_network.clone(),
				binance_coin.clone(),
				trade_symbol.clone(),
				order_side,
				from_amount,
				payout_coin,
				self.binance_api.clone(),
			)
			.await?;
			payout_amount = result.0;
			payout_amount_u256 = result.1;
		}

		Ok((payout_amount, payout_amount_u256))
	}

	async fn do_payout<A, N>(
		&self,
		client: &Arc<Box<dyn AccountingContractApi<A, N>>>,
		payout_address: A,
		payout_amount: U256,
	) -> Result<(), ()>
	where
		A: Send + Sync + std::fmt::Debug + std::marker::Copy,
		N: Plus<u64, Output = N> + std::fmt::Debug,
	{
		debug!("Getting {:?} nonce for payout request", payout_address);
		let user_nonce = client.get_nonce(payout_address).await.map_err(|_| {
			error!("Failed to get nonce");
		})?;

		debug!("Received {:?} nonce", user_nonce);
		let user_nonce = user_nonce.plus(1u64);
		debug!(
			"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
			payout_address, user_nonce, payout_amount
		);

		client
			.execute_pay_out_request(payout_address, user_nonce, payout_amount)
			.await
			.map_err(|_| {
				error!("Failed to execute pay out request");
			})
	}

	#[allow(clippy::too_many_arguments)]
	pub(crate) async fn wait_for_deposit_confirm(
		from_address: String,
		tx_id: Option<String>,
		binance_network: BinanceNetwork,
		binance_coin: BinanceCoin,
		trade_symbol: String,
		order_side: BinanceOrderSide,
		from_amount: String,
		payout_coin: BinanceCoin,
		binance_api: Arc<BinanceClient>,
	) -> Result<(String, U256), ()> {
		debug!("Waiting for deposit to be confirmed on Binance...");
		debug!("Deposit tx_id: {:?}", tx_id);
		debug!("Source address: {:?}", from_address);
		let mut deposit_confirmed = false;
		let start_time = std::time::Instant::now();
		let timeout = Duration::from_secs(300); // 5 minute timeout

		while !deposit_confirmed && start_time.elapsed() < timeout {
			let Ok(deposit_history) = WalletApi::new(binance_api.as_ref())
				.get_deposit_history(Some(binance_coin.name().into()), tx_id.clone())
				.await
			else {
				error!("Failed to get deposit history");
				continue;
			};

			// Check if there's a recent successful deposit
			for deposit in deposit_history {
				debug!("Deposit: {:?}", deposit);
				if deposit.status == 2 || deposit.status == 7 {
					// 2 = rejected, 7 = Wrong Deposit
					error!("Deposit failed with status: {}", deposit.status);

					return Err(());
				}
				// 1 = success
				if deposit.status == 1
					&& deposit.coin == binance_coin.name()
					&& deposit.network == binance_network.name()
					&& deposit.source_address == Some(from_address.clone())
				{
					deposit_confirmed = true;
					debug!(
						"Deposit confirmed on Binance for {} {}",
						deposit.amount,
						binance_coin.name()
					);
					break;
				}
				debug!("Deposit not confirmed yet, status: {}", deposit.status);
			}

			if !deposit_confirmed {
				debug!("Deposit not confirmed yet, waiting 5 seconds...");
				sleep(Duration::from_secs(5)).await;
			}
		}

		if !deposit_confirmed {
			error!("Deposit not confirmed within timeout period");

			return Err(());
		}

		// 3.
		// SOL to BSC: Make the trade using binance spot trading api from_asset => BNB, If it fails, notify the backend via /v3/trade/cross_fail
		// BSC to SOL: Make the trade using binance spot trading api from_asset => SOL, If it fails, notify the backend via /v3/trade/cross_fail
		let binance_order_params = BinanceCreateOrderParams {
			symbol: trade_symbol.clone(),
			side: order_side.clone(),
			order_type: BinanceOrderType::MARKET,
			quote_order_qty: match order_side {
				BinanceOrderSide::BUY => Some(from_amount.clone()),
				BinanceOrderSide::SELL => None,
			},
			quantity: match order_side {
				BinanceOrderSide::BUY => None,
				BinanceOrderSide::SELL => Some(from_amount.clone()),
			},
			..Default::default()
		};
		debug!("Creating binance order with params: {:?}", binance_order_params);
		let spot_trading_api = SpotTradingApi::new(binance_api.as_ref());

		if let Ok(commission_rates) =
			spot_trading_api.get_commission_rates(&binance_order_params.symbol).await
		{
			info!("Commission rates for {}: {:?}", binance_order_params.symbol, commission_rates);
		}
		let Ok(binance_order) = spot_trading_api.create_order(binance_order_params).await else {
			error!("Failed to create binance order");
			WalletApi::new(binance_api.as_ref())
				.withdraw(
					binance_coin.name(),
					&from_address,
					from_amount,
					Some(binance_network.name()),
				)
				.await
				.map_err(|e| {
					error!("Failed to withdraw asset back to omni account, error: {:?}", e);
				})?;
			debug!("Withdrawed asset back to omni account");

			return Err(());
		};

		// Binance trade pair format:
		// <base-asset><quote-asset>, e.g. BNBUSDT, SOLBNB...
		//
		// SELL: sell the "base-asset" to get "quote-asset"
		// BUY:  buy the "base-asset" with "quote-asset"
		//
		// executedQty: quantity of "base-asset"
		// cummulativeQuoteQty: quantity of "quote-asset"
		//
		// so, in SELL orders:
		// - `executedQty` reflects the amount of the base-asset sold
		// - `cummulativeQuoteQty` reflects the amount of the quote-asset received
		//
		// in BUY orders:
		// - `executedQty` indicates the amount of the base-asset bought
		// - `cummulativeQuoteQty`` shows the total amount of the quote-asset spent
		let (trade_success, payout_amount) = loop {
			let trade_order = SpotTradingApi::new(binance_api.as_ref())
				.get_order(&trade_symbol, Some(binance_order.order_id), None, None)
				.await
				.map_err(|_| {
					error!("Failed to get binance order");
				})?;

			match trade_order.status {
				BinanceOrderStatus::FILLED => {
					info!("Binance order filled");
					let payout_amount = match trade_order.side {
						BinanceOrderSide::BUY => trade_order.executed_qty,
						BinanceOrderSide::SELL => trade_order.cummulative_quote_qty,
					};
					break (true, payout_amount);
				},
				// BinanceOrderStatus::CANCELED => {
				// 	error!("Binance order canceled");
				// },
				// BinanceOrderStatus::REJECTED => {
				// 	error!("Binance order rejected");
				// },
				// BinanceOrderStatus::EXPIRED => {
				// 	error!("Binance order expired");
				// },
				// BinanceOrderStatus::EXPIRED_IN_MATCH => {
				// 	error!("Binance order expired in matching");
				// },
				_ => {
					debug!("Binance order status: {:?}", trade_order.status);
				},
			}
			//todo: how long we wait ?
			sleep(Duration::from_millis(500)).await;
		};
		if !trade_success {
			error!("Binance order failed");
			WalletApi::new(binance_api.as_ref())
				.withdraw(
					binance_coin.name(),
					&from_address,
					from_amount,
					Some(binance_network.name()),
				)
				.await
				.map_err(|e| {
					error!("Failed to withdraw asset back to omni account, error: {:?}", e);
				})?;
			debug!("Withdrawed asset back to omni account");

			return Err(());
		}

		debug!("Total acquired {} {}", payout_amount, payout_coin.name());

		let payout_amount_u256 = str_to_u256(&payout_amount, payout_coin.decimals())?;

		Ok((payout_amount, payout_amount_u256))
	}

	async fn apply_gas_fee(
		&self,
		payout_amount: &str,
		payout_coin: BinanceCoin,
		access_token: &str,
		pumpx_config: &PumpxConfig,
	) -> Result<String, ()> {
		debug!("Calling pumpx get_gas_info, chain_id: {}", pumpx_config.to_chain_id);
		let res = self
			.pumpx_api
			.get_gas_info(access_token, pumpx_config.to_chain_id)
			.await
			.map_err(|_| {
				error!("Failed to get gas info");
			})?;
		debug!("Response get_gas_info: {:?}", res);

		let Some(gas_info_vec) = res.data.gas_info else {
			error!("Response data.gas_info of call get gas info is none");
			return Err(());
		};
		let Some(gas_info) =
			gas_info_vec.iter().find(|g| g.chain_id == pumpx_config.to_chain_id.to_string())
		else {
			error!("Could not find matching gas_info with chain_id {}", pumpx_config.to_chain_id);
			return Err(());
		};

		let gas_fee = match pumpx_config.gas_type {
			1 => &gas_info.normal,
			2 => &gas_info.fast,
			3 => &gas_info.super_fast,
			_ => {
				error!("Unsupported gas type: {}", pumpx_config.gas_type);
				return Err(());
			},
		};
		debug!("Gas fee for chain_id {} is {}", pumpx_config.to_chain_id, gas_fee);

		calculate_amount_in(payout_amount, gas_fee, payout_coin.decimals())
	}
}

pub(crate) async fn estimate_payout_amount<BinanceClient: BinanceApi>(
	binance_api: &Arc<BinanceClient>,
	trade_symbol: &str,
	binance_coin: BinanceCoin,
	from_amount_decimal: Decimal,
) -> Result<String, ()> {
	let price_str = SpotTradingApi::new(binance_api.as_ref())
		.get_symbol_price(trade_symbol)
		.await
		.map_err(|_| {
			error!("Failed to get symbol price for {}", trade_symbol);
		})?;

	let price = if trade_symbol == "SOLBNB" && binance_coin == BinanceCoin::Sol {
		// For SOL coin and SOLBNB trade symbol, we sell SOL to get BNB, so get SOLBNB price
		Decimal::from_str(&price_str).map_err(|_| {
			error!("Failed to parse symbol price {}", price_str);
		})?
	} else {
		// For USDC/USDT, we buy BNB with USDC/USDT, so get BNBUSDC/BNBUSDT price and invert
		let price = Decimal::from_str(&price_str).map_err(|_| {
			error!("Failed to parse symbol price {}", price_str);
		})?;
		if price.is_zero() {
			error!("Symbol price is zero for {}", trade_symbol);
			return Err(());
		}
		Decimal::ONE / price
	};

	let estimated = from_amount_decimal * price;

	// Apply 0.1% service fee
	let service_fee_rate = Decimal::from_str("0.001").expect("Failed to parse service fee rate");
	let payout_amount = estimated * (Decimal::ONE - service_fee_rate);

	debug!("estimated: {}, after 0.1% fee: {}", estimated, payout_amount);

	Ok(payout_amount.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_calculate_amount_in() {
		let bnb_to_receive = "0.0050773374896233926847036101";
		let gas = "0.0004731804";
		let decimals = 18;

		let result = calculate_amount_in(bnb_to_receive, gas, decimals);
		assert_eq!(result, Ok("0.004604157089623392".to_string()));
	}

	#[test]
	fn test_str_to_u256() {
		let amount = "0.0050773374896233926847036101";
		let decimals = 18;

		let result = str_to_u256(amount, decimals);
		assert_eq!(result, Ok(U256::from_str("5077337489623392").unwrap()));
	}
}
