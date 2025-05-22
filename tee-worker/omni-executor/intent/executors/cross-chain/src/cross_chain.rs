// TODO: put it into a mod for potential different providers (other than binance)

use super::*;
use executor_primitives::{PumpxConfig, SwapOrder};
use intent_token_query::query_ethereum;
use intent_token_query::query_solana;
use intent_token_query::EthereumAddress;
use intent_token_query::SolanaPubkey;
use std::ops::Deref;
use tracing::{debug, error, info};

impl<
		BinanceClient: BinanceApi,
		EthereumClient: EthereumClientTrait,
		SolanaClient: SolanaClientTrait,
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
	) -> Result<(String, String, Option<InstantFlowDetails>), ()> {
		debug!("executing cross chain swap");

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

		let Some(to_chain_type) = ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id) else {
			error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
			return Err(());
		};

		let from_asset_binance_coin_name =
			BinanceAsset::from_chain_asset(&swap_order.from_asset)?.coin.name();

		let from_amount_decimal = Decimal::from_str(&from_amount).map_err(|_| {
			error!("Failed to parse from_amount_string");
		})?;

		let usdt_trade_symbol = match &swap_order.from_asset {
			ChainAsset::Solana(_) => "SOLUSDT",
			ChainAsset::Ethereum(_, _) => "BNBUSDT",
		};

		let estimated_from_amount_in_usdt = estimate_asset_value_in_usdt(
			&self.binance_api,
			usdt_trade_symbol,
			from_asset_binance_coin_name,
			from_amount_decimal,
		)
		.await?;

		let instant = estimated_from_amount_in_usdt <= self.instant_payout_threshold;

		debug!(
			"Instant: {}, threshold: {:?}, estimated usdt amount: {:?}",
			instant, self.instant_payout_threshold, estimated_from_amount_in_usdt
		);

		let amount_to_lock = AmountType::from_str(
			&Self::calculate_amount_decimal(from_amount_decimal, from_asset_binance_coin_name)?
				.to_string(),
		)
		.unwrap();

		if instant {
			let available_amount =
				self.get_token_available_amount(&swap_order.from_asset, from_wallet).await?;

			self.account_asset_lock.check_and_insert(
				account_id.clone(),
				swap_order.from_asset.clone(),
				amount_to_lock,
				available_amount,
			)?;
		}

		let to_wallet = self
			.pumpx_signer_client
			.request_wallet(to_chain_type, pumpx_config.wallet_index, omni_account)
			.await
			.map_err(|e| error!("Could not get to_wallet from pumpx-signer: {:?}", e))?;

		let to_address = pubkey_to_address(to_chain_type, &to_wallet)?;

		self.pumpx_create_cross_order(
			intent_id,
			from_token_ca.clone(),
			to_token_ca.clone(),
			from_amount.clone(),
			usd_worth,
			from_address.clone(),
			access_token,
			pumpx_config,
		)
		.await?;

		match (&swap_order.from_asset, &swap_order.to_asset) {
			// SOL to BSC
			(ChainAsset::Solana(_), ChainAsset::Ethereum(pumpx::constants::BSC_CHAIN_ID, _)) => {
				let payout_address = Address::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("cross chain swap details: intent_id: {}, from_token_ca: {}, to_token_ca: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, from_token_ca, to_token_ca, from_amount, from_address, payout_address);

				if instant {
					// todo: can we reuse existing code ?

					let (_, binance_asset, from_amount_decimal, _) =
						Self::get_binance_deposit_info(
							self.binance_api.clone(),
							&swap_order.from_asset,
							&from_amount,
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
					// TODO: this should be abstracted away
					let (payout_amount, payout_amount_u256) = self
						.do_binance_swap_sol_to_bsc(
							omni_account,
							swap_order.from_asset.clone(),
							from_amount,
							from_address,
							pumpx_config.wallet_index,
							false,
						)
						.await?;

					self.do_payout_sol_to_bsc(&payout_address, &payout_amount_u256).await?;

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
			(ChainAsset::Ethereum(pumpx::constants::BSC_CHAIN_ID, _), ChainAsset::Solana(_)) => {
				let payout_address = Pubkey::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("cross chain swap details: intent_id: {}, from_token_ca: {}, to_token_ca: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, from_token_ca, to_token_ca, from_amount, from_address, payout_address);

				// TODO: this should be abstracted away
				let (payout_amount, payout_amount_u256) = self
					.do_binance_swap_bsc_to_sol(
						omni_account,
						swap_order.from_asset.clone(),
						from_amount,
						from_address,
						pumpx_config.wallet_index,
						false,
					)
					.await?;

				self.do_payout_bsc_to_sol(payout_address, payout_amount_u256)?;

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
	async fn do_binance_swap_sol_to_bsc(
		&self,
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		wallet_index: u32,
		should_wait_for_deposit_confirm: bool,
	) -> Result<(String, U256), ()> {
		let (deposit_address, binance_asset, from_amount_decimal, amount_to_transfer_decimal) =
			Self::get_binance_deposit_info(self.binance_api.clone(), &from_asset, &from_amount)
				.await?;

		let binance_network = binance_asset.network;
		let binance_coin = binance_asset.coin;
		let binance_address = binance_asset.address;

		let Some(amount_to_transfer) = amount_to_transfer_decimal.to_u64() else {
			error!("Failed to convert amount to transfer to u64");
			return Err(());
		};

		let (trade_symbol, order_side) = determine_trade_symbol_and_order_side(
			binance_network.clone(),
			binance_coin.clone(),
			BinanceNetwork::Bsc,
		)?;

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_payout_amount(
			&self.binance_api,
			&trade_symbol,
			binance_coin.clone(),
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Bnb.decimals())?;

		// Fetch contract balance
		let balance = self.evm_accounting_contract_client.get_balance().await?;
		if balance < payout_amount_u256 {
			error!(
				"There is not enough balance in the accounting contract, {} < {}",
				balance, payout_amount_u256
			);
			return Err(());
		}

		let remote_signer: Box<dyn solana_sdk::signer::Signer + Send + Sync> =
			Box::new(RemoteSolanaSigner::new(
				self.pumpx_signer_client.clone(),
				wallet_index,
				omni_account,
				Handle::current(),
			));

		let mut tx_id: Option<String> = None;
		// TODO: change this when adding support for more tokens/chains
		if binance_coin.clone() == BinanceCoin::Sol {
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
				amount_to_transfer, binance_address, deposit_address
			);
			// SPL transfer
			let signature = self
				.solana_client
				.transfer_spl(
					&deposit_address,
					amount_to_transfer,
					&binance_address,
					&remote_signer,
				)
				.await
				.map_err(|_| {
					error!("Failed to transfer SPL");
				})?;
			tx_id = Some(signature);
		}

		if should_wait_for_deposit_confirm {
			let result = Self::wait_for_deposit_confirm(
				from_address,
				tx_id.clone(),
				binance_network.clone(),
				binance_coin.clone(),
				trade_symbol.clone(),
				order_side,
				from_amount,
				BinanceCoin::Bnb,
				self.binance_api.clone(),
			)
			.await?;
			payout_amount = result.0;
			payout_amount_u256 = result.1;
		}

		Ok((payout_amount, payout_amount_u256))
	}

	async fn do_payout_sol_to_bsc(
		&self,
		payout_address: &Address,
		payout_amount: &U256,
	) -> Result<(), ()> {
		debug!("Getting {:?} nonce for payout request", payout_address);
		let user_nonce = self
			.evm_accounting_contract_client
			.get_nonce(*payout_address)
			.await
			.map_err(|_| {
				error!("Failed to get nonce");
			})?;

		debug!("Received {:?} nonce", user_nonce);
		let user_nonce = user_nonce + U256::from(1u64);
		debug!(
			"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
			payout_address, user_nonce, payout_amount
		);

		self.evm_accounting_contract_client
			.execute_pay_out_request(*payout_address, user_nonce, *payout_amount)
			.await
			.map_err(|_| {
				error!("Failed to execute pay out request");
			})
	}

	// TODO: move `should_wait_for_deposit_confirm` to provider config
	async fn do_binance_swap_bsc_to_sol(
		&self,
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		wallet_index: u32,
		should_wait_for_deposit_confirm: bool,
	) -> Result<(String, U256), ()> {
		let (deposit_address, binance_asset, from_amount_decimal, amount_to_transfer_decimal) =
			Self::get_binance_deposit_info(self.binance_api.clone(), &from_asset, &from_amount)
				.await?;

		let binance_network = binance_asset.network;
		let binance_coin = binance_asset.coin;
		let binance_address = binance_asset.address;

		let amount_to_transfer = U256::from_str_radix(&amount_to_transfer_decimal.to_string(), 10)
			.map_err(|err| {
				error!("Failed to convert amount_to_transfer_decimal to U256: {:?}", err);
			})?;

		let (trade_symbol, order_side) = determine_trade_symbol_and_order_side(
			binance_network.clone(),
			binance_coin.clone(),
			BinanceNetwork::Sol,
		)?;

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_payout_amount(
			&self.binance_api,
			&trade_symbol,
			binance_coin.clone(),
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Sol.decimals())?;

		// Fetch contract balance
		let balance = self.solana_accounting_contract_client.get_balance()?;
		if balance < payout_amount_u256 {
			error!(
				"There is not enough balance in the accounting contract, {} < {}",
				balance, payout_amount_u256
			);
			return Err(());
		}

		let remote_signer =
			RemoteEvmSigner::new(self.pumpx_signer_client.clone(), wallet_index, omni_account)
				.await
				.map_err(|err| {
					error!("Failed to create RemoteEvmSigner: {}", err);
				})?;
		let remote_signer: Box<dyn AlloyTxSigner<Signature> + Send + Sync> =
			Box::new(remote_signer);

		let mut tx_id: Option<String> = None;
		// TODO: change this when adding support for more tokens/chains
		if binance_coin.clone() == BinanceCoin::Bnb {
			// Native transfer
			debug!("Transfering {:?} BNB to {:?}", amount_to_transfer, deposit_address);
			let signature = self
				.bsc_client
				.transfer(&deposit_address, amount_to_transfer, remote_signer)
				.await
				.map_err(|_| {
					error!("Failed to transfer BNB");
				})?;
			tx_id = Some(signature);
		} else {
			debug!(
				"Transfering {:?} {:?} to {:?}",
				amount_to_transfer, binance_address, deposit_address
			);
			// SPL transfer
			let signature = self
				.bsc_client
				.transfer_erc20(
					&deposit_address,
					amount_to_transfer,
					&binance_address,
					remote_signer,
				)
				.await
				.map_err(|_| {
					error!("Failed to transfer ERC20");
				})?;
			tx_id = Some(signature);
		}

		if should_wait_for_deposit_confirm {
			let result = Self::wait_for_deposit_confirm(
				from_address,
				tx_id.clone(),
				binance_network.clone(),
				binance_coin,
				trade_symbol.clone(),
				order_side,
				from_amount,
				BinanceCoin::Sol,
				self.binance_api.clone(),
			)
			.await?;
			payout_amount = result.0;
			payout_amount_u256 = result.1;
		}

		Ok((payout_amount, payout_amount_u256))
	}

	fn do_payout_bsc_to_sol(&self, payout_address: Pubkey, payout_amount: U256) -> Result<(), ()> {
		debug!("Getting {:?} nonce for payout request", payout_address);
		let user_nonce =
			self.solana_accounting_contract_client.get_nonce(payout_address).map_err(|_| {
				error!("Failed to get nonce");
			})?;

		debug!("Received {:?} nonce", user_nonce);
		let user_nonce = user_nonce + 1u64;
		debug!(
			"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
			payout_address, user_nonce, payout_amount
		);

		self.solana_accounting_contract_client
			.execute_pay_out_request(payout_address, user_nonce, payout_amount)
			.map_err(|_| {
				error!("Failed to execute pay out request");
			})
	}

	pub(crate) async fn get_binance_deposit_info(
		binance_api: Arc<BinanceClient>,
		swap_order_from_asset: &ChainAsset,
		from_amount: &str,
	) -> Result<(String, BinanceAsset, Decimal, Decimal), ()> {
		let coins_info =
			WalletApi::new(binance_api.as_ref()).get_all_coins_info().await.map_err(|e| {
				error!("Failed to get all coins info, {:?}", e);
			})?;

		let binance_asset = BinanceAsset::from_chain_asset(swap_order_from_asset)?;

		debug!(
			"from_network_name: {}, binance_coin_name: {}, token_address: {}",
			binance_asset.network.name(),
			binance_asset.coin.name(),
			binance_asset.address
		);

		let Some(binance_coin_info) =
			coins_info.iter().find(|c| c.coin == binance_asset.coin.name())
		else {
			error!(
				"Failed to find binance network list for asset: {:?}",
				binance_asset.coin.name()
			);
			return Err(());
		};
		let Some(_binance_network_info) = binance_coin_info
			.network_list
			.iter()
			.find(|n| n.network == binance_asset.network.name())
		else {
			error!(
				"Failed to find binance network list for asset: {:?}",
				binance_asset.coin.name()
			);
			return Err(());
		};

		let deposit_address = WalletApi::new(binance_api.as_ref())
			.get_deposit_address(binance_asset.coin.name(), binance_asset.network.name())
			.await
			.map_err(|_| {
				error!("Failed to get deposit address");
			})?;

		let from_amount_decimal = Decimal::from_str(from_amount).map_err(|_| {
			error!("Failed to parse from_amount_string");
		})?;

		debug!("Binance deposit address: {}, from_amount: {}", deposit_address, from_amount);

		let asset_decimal_multiplier = binance_asset.coin.decimal_value();

		// Convert from human readable amount to raw token amount
		let amount_to_transfer = from_amount_decimal * asset_decimal_multiplier;

		Ok((deposit_address, binance_asset, from_amount_decimal, amount_to_transfer))
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

	async fn get_token_available_amount(
		&self,
		asset: &ChainAsset,
		address: Vec<u8>,
	) -> Result<U256, ()> {
		match &asset {
			ChainAsset::Ethereum(chain_id, token) => {
				let rpc_url = self
					.rpc_endpoint_registry
					.get(&Chain::Ethereum(*chain_id))
					.ok_or(error!("No RPC endpoint in registry for Ethereum chain"))?;
				query_ethereum(rpc_url, EthereumAddress::from_slice(&address), token).await
			},
			ChainAsset::Solana(token) => {
				let pubkey = SolanaPubkey::try_from(address).map_err(|e| {
					error!("Could not create solana pubkey from wallet address: {:?}", e)
				})?;
				query_solana(self.solana_client.deref(), &pubkey, token)
					.await
					.map(|v| AmountType::from(v))
			},
		}
	}
}

pub(crate) fn determine_trade_symbol_and_order_side(
	binance_network: BinanceNetwork,
	binance_coin: BinanceCoin,
	to_network: BinanceNetwork,
) -> Result<(String, BinanceOrderSide), ()> {
	match (binance_network.clone(), binance_coin.clone(), to_network.clone()) {
		(BinanceNetwork::Sol, BinanceCoin::Usdc, BinanceNetwork::Bsc) => {
			Ok(("BNBUSDC".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Sol, BinanceCoin::Usdt, BinanceNetwork::Bsc) => {
			Ok(("BNBUSDT".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Sol, BinanceCoin::Sol, BinanceNetwork::Bsc) => {
			Ok(("SOLBNB".to_string(), BinanceOrderSide::SELL))
		},
		(BinanceNetwork::Bsc, BinanceCoin::Usdc, BinanceNetwork::Sol) => {
			Ok(("SOLUSDC".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Bsc, BinanceCoin::Usdt, BinanceNetwork::Sol) => {
			Ok(("SOLUSDT".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Bsc, BinanceCoin::Bnb, BinanceNetwork::Sol) => {
			Ok(("SOLBNB".to_string(), BinanceOrderSide::BUY))
		},
		_ => {
			error!("Unsupported binance_network_name: {:?}, binance_coin_name: {:?}, to_network_name: {:?}", binance_network.name(), binance_coin.name(), to_network.name());
			Err(())
		},
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

async fn estimate_asset_value_in_usdt<BinanceClient: BinanceApi>(
	binance_api: &Arc<BinanceClient>,
	trade_symbol: &str,
	binance_coin_name: &str,
	from_amount_decimal: Decimal,
) -> Result<Decimal, ()> {
	let price_str = SpotTradingApi::new(binance_api.as_ref())
		.get_symbol_price(trade_symbol)
		.await
		.map_err(|_| {
			error!("Failed to get symbol price for {}", trade_symbol);
		})?;

	let price = if binance_coin_name == "SOL" {
		// For SOL, we sell SOL to get BNB, so get SOLBNB price
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

	let amount = from_amount_decimal * price;

	debug!("From amount in usdt: {}", amount);
	Ok(amount)
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

pub(crate) fn str_to_u256(amount: &str, decimals: u32) -> Result<U256, ()> {
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
