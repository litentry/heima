// TODO: put it into a mod for potential different providers (other than binance)

use super::*;
use executor_primitives::{PumpxConfig, SwapOrder};
use log::{debug, error, info};

impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait>
	CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
	pub async fn execute_cross_chain_swap(
		&self,
		omni_account: [u8; 32],
		intent_id: IntentId,
		access_token: &str,
		swap_order: &SwapOrder,
		pumpx_config: &PumpxConfig,
	) -> Result<String, ()> {
		// payout_amount
		debug!("executing cross chain swap");

		if pumpx_config.to_chain_id != BSC_CHAIN_ID {
			error!("Only bsc payout supported");
			return Err(());
		}

		if pumpx_config.order_type != PumpxOrderType::Market {
			error!("Only market order supported");
			return Err(());
		}

		// extract values from pumpx_config
		let usd_worth = std::str::from_utf8(&pumpx_config.usd_worth)
			.map_err(|_| {
				error!("Failed to parse usd_worth");
			})
			.map(|v| v.to_string())?;

		let from_token_ca = std::str::from_utf8(&pumpx_config.from_token_ca)
			.map_err(|_| {
				error!("Failed to parse from_token_ca");
			})
			.map(|v| v.to_string())?;

		let to_token_ca = std::str::from_utf8(&pumpx_config.to_token_ca)
			.map_err(|_| {
				error!("Failed to parse to_token_ca");
			})
			.map(|v| v.to_string())?;

		let from_amount = std::str::from_utf8(&pumpx_config.from_amount)
			.map_err(|_| {
				error!("Failed to parse from_amount");
			})
			.map(|v| v.to_string())?;

		let Some(from_chain_type) = ChainType::from_pumpx_chain_id(pumpx_config.from_chain_id)
		else {
			error!("Unsupported from_chain_id: {}", pumpx_config.from_chain_id);
			return Err(());
		};

		let Some(to_chain_type) = ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id) else {
			error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
			return Err(());
		};

		let from_wallet = self
			.pumpx_signer_client
			.request_wallet(from_chain_type, pumpx_config.wallet_index, omni_account)
			.await
			.map_err(|e| error!("Could not get from_wallet from pumpx-signer: {:?}", e))?;

		let from_address = pubkey_to_address(from_chain_type, &from_wallet)?;

		let to_wallet = self
			.pumpx_signer_client
			.request_wallet(to_chain_type, pumpx_config.wallet_index, omni_account)
			.await
			.map_err(|e| error!("Could not get to_wallet from pumpx-signer: {:?}", e))?;

		let to_address = pubkey_to_address(to_chain_type, &to_wallet)?;

		let payout_address = Address::from_str(&to_address).map_err(|_| {
			error!("Failed to parse payout address");
		})?;

		self.pumpx_create_cross_order(
			intent_id,
			from_token_ca,
			to_token_ca,
			from_amount.clone(),
			usd_worth,
			from_address.clone(),
			access_token,
			pumpx_config,
		)
		.await?;

		// TODO: this should be abstracted away
		let (payout_amount, payout_amount_u256) = self
			.do_binance_swap(
				omni_account,
				swap_order.from_asset.clone(),
				from_amount,
				from_address,
				pumpx_config.wallet_index,
				false,
			)
			.await?;

		self.do_payout(&payout_address, &payout_amount_u256).await?;

		self.apply_gas_fee(&payout_amount, access_token, pumpx_config).await
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
	async fn do_binance_swap(
		&self,
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		wallet_index: u32,
		should_wait_for_deposit_confirm: bool,
	) -> Result<(String, U256), ()> {
		let coins_info = WalletApi::new(self.binance_api.as_ref())
			.get_all_coins_info()
			.await
			.map_err(|e| {
				error!("Failed to get all coins info, {:?}", e);
			})?;

		// TODO: create an util function to convert ChainAsset to binance names
		// and create constants for SOL, USDC, USDT, etc
		let (from_network_name, binance_coin_name, token_address) = match from_asset {
			ChainAsset::Solana(ref token) => {
				let (asset, token_address) = match token {
					SolanaToken::Native => ("SOL", ""),
					SolanaToken::SPL(mint_address) => {
						let mint_address_string = mint_address.as_ref().to_base58();
						match mint_address_string.as_str() {
							SOLANA_USDC_MINT_ADDRESS => ("USDC", SOLANA_USDC_MINT_ADDRESS),
							SOLANA_USDT_MINT_ADDRESS => ("USDT", SOLANA_USDT_MINT_ADDRESS),
							_ => {
								error!("Unsupported SPL token: {:?}", mint_address);
								return Err(());
							},
						}
					},
				};
				("SOL".to_string(), asset.to_string(), token_address.to_string())
			},
			ChainAsset::Ethereum(..) => {
				error!("Unsupported from_asset: {:?}", from_asset);
				return Err(());
			},
		};

		debug!(
			"from_network_name: {}, binance_coin_name: {}, token_address: {}",
			from_network_name, binance_coin_name, token_address
		);

		let Some(binance_coin_info) = coins_info.iter().find(|c| c.coin == binance_coin_name)
		else {
			error!("Failed to find binance network list for asset: {:?}", binance_coin_name);
			return Err(());
		};
		let Some(binance_network_info) =
			binance_coin_info.network_list.iter().find(|n| n.network == from_network_name)
		else {
			error!("Failed to find binance network list for asset: {:?}", binance_coin_name);
			return Err(());
		};

		let deposit_address = WalletApi::new(self.binance_api.as_ref())
			.get_deposit_address(&binance_coin_name, &binance_network_info.network)
			.await
			.map_err(|_| {
				error!("Failed to get deposit address");
			})?;

		let from_amount_decimal = Decimal::from_str(&from_amount).map_err(|_| {
			error!("Failed to parse from_amount_string");
		})?;

		debug!("Binance deposit address: {}, from_amount: {}", deposit_address, from_amount);

		let asset_decimal_multiplier = match binance_coin_name.as_str() {
			"USDC" => Decimal::from(1_000_000),    // 10^6
			"USDT" => Decimal::from(1_000_000),    // 10^6
			"SOL" => Decimal::from(1_000_000_000), // 10^9  TODO: double check this
			_ => {
				error!("Unsupported asset: {:?}", binance_coin_name);
				return Err(());
			},
		};
		let amount_to_transfer_decimal = from_amount_decimal * asset_decimal_multiplier;
		let Some(amount_to_transfer) = amount_to_transfer_decimal.to_u64() else {
			error!("Failed to convert amount to transfer to u64");
			return Err(());
		};

		let (trade_symbol, order_side) = match binance_coin_name.as_str() {
			"USDC" => ("BNBUSDC".to_string(), BinanceOrderSide::BUY),
			"USDT" => ("BNBUSDT".to_string(), BinanceOrderSide::BUY),
			"SOL" => ("SOLBNB".to_string(), BinanceOrderSide::SELL),
			_ => {
				error!("Unsupported asset: {:?}", binance_coin_name);
				return Err(());
			},
		};

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_bnb_amount(
			&self.binance_api,
			&trade_symbol,
			&binance_coin_name,
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, 18)?;

		// Fetch contract balance
		let balance = self.accounting_contract_client.get_balance().await?;
		if balance < payout_amount_u256 {
			error!(
				"There is not enough balance in the accounting contract, {} < {}",
				balance, payout_amount_u256
			);
			return Err(());
		}

		let remote_signer: Box<dyn solana_sdk::signer::Signer + Send + Sync> =
			Box::new(RemoteSigner::new(
				self.pumpx_signer_client.clone(),
				wallet_index,
				omni_account,
				Handle::current(),
			));

		let mut tx_id: Option<String> = None;
		// TODO: change this when adding support for more tokens/chains
		if binance_coin_name == "SOL" {
			// Native transfer
			debug!("Transfering {:?} SOL to {:?}", amount_to_transfer, deposit_address);
			let signature = self
				.solana_client
				.transfer_sol(&deposit_address, amount_to_transfer, &remote_signer)
				.await
				.map_err(|_| {
					error!("Failed to transfer SOL");
				})?;
			tx_id = Some(signature);
		} else {
			debug!(
				"Transfering {:?} {:?} to {:?}",
				amount_to_transfer, token_address, deposit_address
			);
			// SPL transfer
			let signature = self
				.solana_client
				.transfer_spl(&deposit_address, amount_to_transfer, &token_address, &remote_signer)
				.await
				.map_err(|_| {
					error!("Failed to transfer SPL");
				})?;
			tx_id = Some(signature);
		}

		if should_wait_for_deposit_confirm {
			debug!("Waiting for deposit to be confirmed on Binance...");
			debug!("Deposit tx_id: {:?}", tx_id);
			debug!("Source address: {:?}", from_address);
			let mut deposit_confirmed = false;
			let start_time = std::time::Instant::now();
			let timeout = Duration::from_secs(300); // 5 minute timeout

			while !deposit_confirmed && start_time.elapsed() < timeout {
				let Ok(deposit_history) = WalletApi::new(self.binance_api.as_ref())
					.get_deposit_history(Some(binance_coin_name.clone()), tx_id.clone())
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
					if deposit.status == 1 && // 1 = success
							   deposit.coin == binance_coin_name &&
							   deposit.network == binance_network_info.network &&
                               deposit.source_address == Some(from_address.clone())
					{
						deposit_confirmed = true;
						debug!(
							"Deposit confirmed on Binance for {} {}",
							deposit.amount, binance_coin_name
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

			// 3. Make the trade using binance spot trading api from_asset => BNB, If it fails, notify the backend via /v3/trade/cross_fail
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
			let Ok(binance_order) = SpotTradingApi::new(self.binance_api.as_ref())
				.create_order(binance_order_params)
				.await
			else {
				error!("Failed to create binance order");
				WalletApi::new(self.binance_api.as_ref())
					.withdraw(
						&binance_coin_name,
						&from_address,
						from_amount,
						Some(&binance_network_info.network),
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
			let mut trade_success = false;
			let mut bnb_acquired = "".to_string();
			loop {
				let trade_order = SpotTradingApi::new(self.binance_api.as_ref())
					.get_order(&trade_symbol, Some(binance_order.order_id), None, None)
					.await
					.map_err(|_| {
						error!("Failed to get binance order");
					})?;

				match trade_order.status {
					BinanceOrderStatus::FILLED => {
						info!("Binance order filled");
						bnb_acquired = match trade_order.side {
							BinanceOrderSide::BUY => trade_order.executed_qty,
							BinanceOrderSide::SELL => trade_order.cummulative_quote_qty,
						};
						trade_success = true;
						break;
					},
					BinanceOrderStatus::CANCELED => {
						error!("Binance order canceled");
					},
					BinanceOrderStatus::REJECTED => {
						error!("Binance order rejected");
					},
					BinanceOrderStatus::EXPIRED => {
						error!("Binance order expired");
					},
					BinanceOrderStatus::EXPIRED_IN_MATCH => {
						error!("Binance order expired in matching");
					},
					_ => {
						debug!("Binance order status: {:?}", trade_order.status);
					},
				}
				//todo: how long we wait ?
				sleep(Duration::from_millis(500)).await;
			}
			if !trade_success {
				error!("Binance order failed");
				WalletApi::new(self.binance_api.as_ref())
					.withdraw(
						&binance_coin_name,
						&from_address,
						from_amount,
						Some(&binance_network_info.network),
					)
					.await
					.map_err(|e| {
						error!("Failed to withdraw asset back to omni account, error: {:?}", e);
					})?;
				debug!("Withdrawed asset back to omni account");
				return Err(());
			}

			debug!("Total acquired {} bnb", bnb_acquired);

			// Update payout amount based on the order filled
			// TODO: shall we apply 0.1% service fee here?
			payout_amount = bnb_acquired;
			payout_amount_u256 = str_to_u256(&payout_amount, 18)?;
		}

		Ok((payout_amount, payout_amount_u256))
	}

	async fn do_payout(&self, payout_address: &Address, payout_amount: &U256) -> Result<(), ()> {
		debug!("Getting {:?} nonce for payout request", payout_address);
		let user_nonce =
			self.accounting_contract_client.get_nonce(*payout_address).await.map_err(|_| {
				log::error!("Failed to get nonce");
			})?;

		debug!("Received {:?} nonce", user_nonce);
		let user_nonce = user_nonce + U256::from(1u64);
		debug!(
			"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
			payout_address, user_nonce, payout_amount
		);

		self.accounting_contract_client
			.execute_pay_out_request(*payout_address, user_nonce, *payout_amount)
			.await
			.map_err(|_| {
				log::error!("Failed to execute pay out request");
			})
	}

	async fn apply_gas_fee(
		&self,
		payout_amount: &str,
		access_token: &str,
		pumpx_config: &PumpxConfig,
	) -> Result<String, ()> {
		debug!("Calling pumpx get_gas_info, chain_id: {}", pumpx_config.to_chain_id);
		let res = self
			.pumpx_api
			.get_gas_info(access_token, pumpx_config.to_chain_id)
			.await
			.map_err(|_| {
				log::error!("Failed to get gas info");
			})?;
		debug!("Response get_gas_info: {:?}", res);

		let Some(gas_info_vec) = res.data.gas_info else {
			log::error!("Response data.gas_info of call get gas info is none");
			return Err(());
		};
		let Some(gas_info) =
			gas_info_vec.iter().find(|g| g.chain_id == pumpx_config.to_chain_id.to_string())
		else {
			log::error!(
				"Could not find matching gas_info with chain_id {}",
				pumpx_config.to_chain_id
			);
			return Err(());
		};

		let gas_fee = match pumpx_config.gas_type {
			1 => &gas_info.normal,
			2 => &gas_info.fast,
			3 => &gas_info.super_fast,
			_ => {
				log::error!("Unsupported gas type: {}", pumpx_config.gas_type);
				return Err(());
			},
		};
		debug!("Gas fee for chain_id {} is {}", pumpx_config.to_chain_id, gas_fee);

		calculate_amount_in(payout_amount, gas_fee, 18)
	}
}

async fn estimate_bnb_amount<BinanceClient: BinanceApi>(
	binance_api: &Arc<BinanceClient>,
	trade_symbol: &str,
	binance_coin_name: &str,
	from_amount_decimal: Decimal,
) -> Result<String, ()> {
	let price_str = SpotTradingApi::new(binance_api.as_ref())
		.get_symbol_price(trade_symbol)
		.await
		.map_err(|_| {
			log::error!("Failed to get symbol price for {}", trade_symbol);
		})?;

	let price = if binance_coin_name == "SOL" {
		// For SOL, we sell SOL to get BNB, so get SOLBNB price
		Decimal::from_str(&price_str).map_err(|_| {
			log::error!("Failed to parse symbol price {}", price_str);
		})?
	} else {
		// For USDC/USDT, we buy BNB with USDC/USDT, so get BNBUSDC/BNBUSDT price and invert
		let price = Decimal::from_str(&price_str).map_err(|_| {
			log::error!("Failed to parse symbol price {}", price_str);
		})?;
		if price.is_zero() {
			log::error!("Symbol price is zero for {}", trade_symbol);
			return Err(());
		}
		Decimal::ONE / price
	};

	let bnb_estimated = from_amount_decimal * price;

	// Apply 0.1% service fee
	let service_fee_rate = Decimal::from_str("0.001").expect("Failed to parse service fee rate");
	let decimal_bnb_to_receive = bnb_estimated * (Decimal::ONE - service_fee_rate);

	debug!("BNB estimated: {}, after 0.1% fee: {}", bnb_estimated, decimal_bnb_to_receive);

	Ok(decimal_bnb_to_receive.to_string())
}

fn calculate_amount_in(amount: &str, gas: &str, decimals: u32) -> Result<String, ()> {
	let amount = Decimal::from_str(amount).map_err(|_| ())?;
	let gas = Decimal::from_str(gas).map_err(|_| ())?;
	if amount <= gas {
		Err(())
	} else {
		let amount = amount - gas;
		Ok(amount.trunc_with_scale(decimals).to_string())
	}
}

fn str_to_u256(amount: &str, decimals: u32) -> Result<U256, ()> {
	let amount = Decimal::from_str(amount).map_err(|_| ())?;
	let factor = Decimal::from(10u64.pow(decimals));
	let scaled = amount * factor;
	let int_str = scaled.trunc().to_string();
	U256::from_str(&int_str).map_err(|_| ())
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
