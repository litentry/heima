// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use accounting_contract_client::AccountingContractClient;
use alloy::{
	primitives::{Address, U256},
	rpc::types::TransactionRequest,
};
use async_trait::async_trait;
use base58::ToBase58;
use binance_api::{
	spot_trading_api::types::{
		CreateOrderParams as BinanceCreateOrderParams, OrderSide as BinanceOrderSide,
		OrderStatus as BinanceOrderStatus, OrderType as BinanceOrderType,
	},
	BinanceApi,
};
use ethereum_rpc::RpcProvider as EthereumRpcProvider;
use executor_core::intent_executor::{IntentExecutionResult, IntentExecutor};
use executor_primitives::Intent;
use executor_primitives::IntentId;
use executor_primitives::PumpxOrderType;
use executor_primitives::SingleChainSwapProvider;
use executor_primitives::SolanaToken;
use executor_storage::StorageDB;
use executor_storage::{PumpxJwtStorage, Storage};
use heima_authentication::auth_token::AUTH_TOKEN_ACCESS_TYPE;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use solana::{signer::RemoteSigner, SolanaClient};
use std::str::FromStr;
use tokio::{
	runtime::Handle,
	time::{sleep, Duration},
};
// use intent_asset_lock::AmountType;
// use intent_token_query::query_ethereum;
// use intent_token_query::query_solana;
// use intent_token_query::EthereumAddress;
// use intent_token_query::SolanaPubkey;
// use log::error;
use parity_scale_codec::Encode;
use pumpx::signer_client::ChainType;
use pumpx::signer_client::SignerClient;
use pumpx::types::CrossOrderInfo;
use pumpx::types::GasType;
use pumpx::types::SwapType;
use pumpx::types::{
	CreateCrossOrderBody, CreateLimitOrderBody, CreateMarketOrderTxBody, CrossFailBody,
};
use pumpx::PumpxApi;
use pumpx::{pubkey_to_evm_address, pubkey_to_solana_address};
use std::collections::HashMap;
use std::sync::Arc;

use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use parentchain_rpc_client::metadata::Metadata;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::CustomConfig;
use parentchain_rpc_client::SubxtClient;
use parentchain_rpc_client::SubxtClientFactory;
use parentchain_signer::TxSigner;

use log::debug;

// use intent_asset_lock::always_unlocked::AlwaysUnlockedAssetsLock;
// use intent_asset_lock::AccountAssetLocks;

#[derive(PartialEq, Hash, Eq)]
pub enum Chain {
	Ethereum(u32),
	Solana,
}

pub type RpcEndpointRegistry = HashMap<Chain, String>;

pub type ParentchainTxSigner = TxSigner<
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

// TODO: temporary solution
const SOLANA_USDC_MINT_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const SOLANA_USDT_MINT_ADDRESS: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

// TODO: should we rename this to something like MultiChainIntentExecutor?
pub struct CrossChainIntentExecutor<Provider: EthereumRpcProvider<Transaction = TransactionRequest>>
{
	// account_asset_lock: AccountAssetLocks<AlwaysUnlockedAssetsLock>,
	// rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pumpx_api: Arc<PumpxApi>,
	storage_db: Arc<StorageDB>,
	binance_api: Arc<BinanceApi>,
	solana_client: Arc<SolanaClient>,
	accounting_contract_client: Arc<AccountingContractClient<Provider>>,
}

impl<Provider: EthereumRpcProvider<Transaction = TransactionRequest>>
	CrossChainIntentExecutor<Provider>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		_rpc_endpoint_registry: RpcEndpointRegistry,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		pumpx_api: Arc<PumpxApi>,
		storage_db: Arc<StorageDB>,
		binance_api: Arc<BinanceApi>,
		solana_client: Arc<SolanaClient>,
		accounting_contract_client: Arc<AccountingContractClient<Provider>>,
	) -> Result<Self, ()> {
		// there is no need for account/assets locks if we guarantee the dest-chain payout happens after the source chain finalisation
		// let account_asset_lock = AccountAssetLocks::<AlwaysUnlockedAssetsLock>::empty();
		Ok(Self {
			// account_asset_lock,
			// rpc_endpoint_registry,
			pumpx_signer_client,
			pumpx_api,
			storage_db,
			binance_api,
			solana_client,
			accounting_contract_client,
		})
	}
}

#[async_trait]
impl<Provider: EthereumRpcProvider<Transaction = TransactionRequest> + Send + Sync> IntentExecutor
	for CrossChainIntentExecutor<Provider>
{
	#[allow(unused_assignments)]
	async fn execute(
		&self,
		account_id: &AccountId,
		intent_id: IntentId,
		intent: Intent,
	) -> Result<IntentExecutionResult, ()> {
		match intent {
			Intent::Swap(ref swap_order, ref _ccsp, ref scsp) => {
				debug!("Started processing SwapOrder intent, order: {:?}, signle chain swap provider: {:?}", swap_order, scsp);

				// let available_amount = match &swap_order.from_asset {
				// 	ChainAsset::Ethereum(chain_id, token) => {
				// 		let rpc_url =
				// 			self.rpc_endpoint_registry.get(&Chain::Ethereum(*chain_id)).ok_or(())?;
				// 		let address = self
				// 			.pumpx_signer_client
				// 			.request_wallet(
				// 				pumpx::signer_client::ChainType::Evm,
				// 				0,
				// 				*account_id.as_ref(),
				// 			)
				// 			.await
				// 			.map_err(|e| {
				// 				error!("Could not get wallet from pumpx-signer: {:?}", e)
				// 			})?;
				// 		query_ethereum(rpc_url, EthereumAddress::from_slice(&address), token)
				// 			.await?
				// 	},
				// 	ChainAsset::Solana(token) => {
				// 		let rpc_url = self.rpc_endpoint_registry.get(&Chain::Solana).ok_or(())?;
				// 		let address = self
				// 			.pumpx_signer_client
				// 			.request_wallet(
				// 				pumpx::signer_client::ChainType::Solana,
				// 				0,
				// 				*account_id.as_ref(),
				// 			)
				// 			.await
				// 			.map_err(|e| {
				// 				error!("Could not get wallet from pumpx-signer: {:?}", e)
				// 			})?;
				// 		let pubkey = SolanaPubkey::try_from(address).map_err(|e| {
				// 			error!("Could not create solana pubkey from wallet address: {:?}", e)
				// 		})?;
				// 		query_solana(rpc_url, &pubkey, token).await.map(|v| AmountType::from(v))?
				// 	},
				// };

				// self.account_asset_lock.check_and_insert(
				// 	account_id.clone(),
				// 	swap_order.from_asset.clone(),
				// 	AmountType::from(from_amount),
				// 	available_amount,
				// )?;
				//

				// TODO: update this when we have more providers
				let SingleChainSwapProvider::Pumpx(pumpx_config) = scsp;

				let usd_worth = std::str::from_utf8(&pumpx_config.usd_worth)
					.map_err(|_| {
						log::error!("Failed to parse usd_worth");
					})
					.map(|v| v.to_string())?;

				let from_token_ca = std::str::from_utf8(&pumpx_config.from_token_ca)
					.map_err(|_| {
						log::error!("Failed to parse from_token_ca");
					})
					.map(|v| v.to_string())?;

				let to_token_ca = std::str::from_utf8(&pumpx_config.to_token_ca)
					.map_err(|_| {
						log::error!("Failed to parse to_token_ca");
					})
					.map(|v| v.to_string())?;

				let from_amount = std::str::from_utf8(&pumpx_config.from_amount)
					.map_err(|_| {
						log::error!("Failed to parse from_amount");
					})
					.map(|v| v.to_string())?;

				let storage = PumpxJwtStorage::new(self.storage_db.clone());
				let Ok(Some(access_token)) =
					storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get access token from storage");
					return Err(());
				};

				let result: IntentExecutionResult;

				if pumpx_config.from_chain_id == pumpx_config.to_chain_id {
					debug!("from and to chain are equal, performing single chain swap");
					let Some(chain_type) = ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id)
					else {
						log::error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
						return Err(());
					};

					let wallet_address = self
						.pumpx_signer_client
						.request_wallet(chain_type, pumpx_config.wallet_index, *account_id.as_ref())
						.await
						.map_err(|e| {
							log::error!("Could not get wallet from pumpx-signer: {:?}", e)
						})?;

					let (order_response, should_notify_parentchain) = match pumpx_config.order_type
					{
						PumpxOrderType::Market => {
							debug!("Doing market order");
							let body = CreateMarketOrderTxBody {
								request_id: intent_id,
								chain_id: pumpx_config.to_chain_id,
								token_ca: to_token_ca.clone(),
								swap_type: match pumpx_config.swap_type {
									1 => SwapType::Buy,
									2 => SwapType::Sell,
									_ => {
										log::error!(
											"Unsupported swap type: {}",
											pumpx_config.swap_type
										);
										return Err(());
									},
								},
								amount_in: from_amount.clone(),
								double_out: pumpx_config.double_out,
								is_one_click: pumpx_config.is_one_click,
								address: match chain_type {
									ChainType::Evm => pubkey_to_evm_address(&wallet_address)?,
									ChainType::Solana => pubkey_to_solana_address(&wallet_address)?,
									_ => {
										log::error!("Unsupported {:?} wallet address", chain_type);
										return Err(());
									},
								},
								is_anti_mev: pumpx_config.is_anti_mev,
								is_auto_slippage: pumpx_config.is_auto_slippage,
								gas_type: match pumpx_config.gas_type {
									1 => GasType::Slow,
									2 => GasType::Medium,
									3 => GasType::Fast,
									_ => {
										log::error!(
											"Unsupported gas type: {}",
											pumpx_config.gas_type
										);
										return Err(());
									},
								},
								slippage: pumpx_config.slippage,
								wallet_index: pumpx_config.wallet_index,
							};
							debug!("Calling pumpx create_market_order_tx, body: {:?}", body);
							let response = self
								.pumpx_api
								.create_market_order_tx(&access_token, body)
								.await
								.map_err(|_| {
									log::error!("Failed to create market order tx");
								})?;

							debug!("Response create_market_order_tx: {:?}", response);
							(response.encode(), true)
						},
						PumpxOrderType::Limit => {
							debug!("Doing limit order");
							let token_cap = match pumpx_config.token_cap {
								Some(ref token_cap) => Some(
									std::str::from_utf8(token_cap)
										.map_err(|_| {
											log::error!("Failed to parse token_cap");
										})
										.map(|v| v.to_string())?,
								),
								None => None,
							};
							let price_usd = match pumpx_config.price_usd {
								Some(ref price_usd) => Some(
									std::str::from_utf8(price_usd)
										.map_err(|_| {
											log::error!("Failed to parse price_usd");
										})
										.map(|v| v.to_string())?,
								),
								None => None,
							};

							let new_limit_order = CreateLimitOrderBody {
								request_id: intent_id,
								chain_id: pumpx_config.to_chain_id,
								token_ca: to_token_ca,
								amount: from_amount,
								swap_type: match pumpx_config.swap_type {
									1 => SwapType::Buy,
									2 => SwapType::Sell,
									_ => {
										log::error!(
											"Unsupported swap type: {}",
											pumpx_config.swap_type
										);
										return Err(());
									},
								},
								double_out: pumpx_config.double_out,
								token_cap,
								price_usd,
								trailing_percent: pumpx_config
									.trailing_percent
									.map(|v| v.to_string()),
								address: match chain_type {
									ChainType::Evm => pubkey_to_evm_address(&wallet_address)?,
									ChainType::Solana => pubkey_to_solana_address(&wallet_address)?,
									_ => {
										log::error!("Unsupported {:?} wallet address", chain_type);
										return Err(());
									},
								},
								is_anti_mev: pumpx_config.is_anti_mev,
								is_auto_slippage: pumpx_config.is_auto_slippage,
								gas_type: match pumpx_config.gas_type {
									1 => GasType::Slow,
									2 => GasType::Medium,
									3 => GasType::Fast,
									_ => {
										log::error!(
											"Unsupported gas type: {}",
											pumpx_config.gas_type
										);
										return Err(());
									},
								},
								slippage: pumpx_config.slippage,
								wallet_index: pumpx_config.wallet_index,
							};
							debug!(
								"Calling pumpx create_limit_order, order: {:?}",
								new_limit_order
							);
							let response = self
								.pumpx_api
								.create_limit_order(&access_token, new_limit_order)
								.await
								.map_err(|_| {
									log::error!("Failed to create limit order");
								})?;

							debug!("Response create_limit_order: {:?}", response);

							(response.encode(), false)
						},
					};
					result = (Some(order_response), should_notify_parentchain);
				} else {
					debug!("from and to chain are different, performing cross chain swap");
					if !matches!(
						swap_order.to_asset,
						ChainAsset::Ethereum(pumpx::constants::BSC_CHAIN_ID, _)
					) {
						log::error!("Only BSC payout supported");
					}

					// notify backend about it
					let Some(from_chain_type) =
						ChainType::from_pumpx_chain_id(pumpx_config.from_chain_id)
					else {
						log::error!("Unsupported from_chain_id: {}", pumpx_config.from_chain_id);
						return Err(());
					};

					let Some(to_chain_type) =
						ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id)
					else {
						log::error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
						return Err(());
					};

					let from_wallet_address = self
						.pumpx_signer_client
						.request_wallet(
							from_chain_type,
							pumpx_config.wallet_index,
							*account_id.as_ref(),
						)
						.await
						.map_err(|e| {
							log::error!("Could not get from_wallet from pumpx-signer: {:?}", e)
						})?;

					let from_address = match from_chain_type {
						ChainType::Evm => pubkey_to_evm_address(&from_wallet_address)?,
						ChainType::Solana => pubkey_to_solana_address(&from_wallet_address)?,
						_ => {
							log::error!("Unsupported {:?} wallet address", from_chain_type);
							return Err(());
						},
					};

					let to_wallet_address = self
						.pumpx_signer_client
						.request_wallet(
							to_chain_type,
							pumpx_config.wallet_index,
							*account_id.as_ref(),
						)
						.await
						.map_err(|e| {
							log::error!("Could not get to_wallet from pumpx-signer: {:?}", e)
						})?;

					let to_address = match to_chain_type {
						ChainType::Evm => pubkey_to_evm_address(&to_wallet_address)?,
						ChainType::Solana => pubkey_to_solana_address(&to_wallet_address)?,
						_ => {
							log::error!("Unsupported {:?} wallet address", to_chain_type);
							return Err(());
						},
					};

					let body = CreateCrossOrderBody {
						request_id: intent_id,
						chain_id: pumpx_config.to_chain_id,
						token_ca: to_token_ca.clone(),
						swap_type: match pumpx_config.swap_type {
							1 => SwapType::Buy,
							2 => SwapType::Sell,
							_ => {
								log::error!("Unsupported swap type: {}", pumpx_config.swap_type);
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
						self.pumpx_api.create_cross_order(&access_token, body).await.map_err(
							|_| {
								log::error!("Failed to create cross order");
							},
						)?;
					debug!("Response create_cross_order: {:?}", response);

					// 2. transfer from_asset to binance deposit address
					let should_wait_for_deposit_confirm = false; // Switch for strategy

					let coins_info =
						self.binance_api.wallet().get_all_coins_info().await.map_err(|e| {
							log::error!("Failed to get all coins info, {:?}", e);
						})?;

					// TODO: create an util function to convert ChainAsset to binance names
					// and create constants for SOL, USDC, USDT, etc
					let (from_network_name, binance_coin_name, token_address) =
						match swap_order.from_asset {
							ChainAsset::Solana(ref token) => {
								let (asset, token_address) = match token {
									SolanaToken::Native => ("SOL", ""),
									SolanaToken::SPL(mint_address) => {
										let mint_address_string = mint_address.as_ref().to_base58();
										match mint_address_string.as_str() {
											SOLANA_USDC_MINT_ADDRESS => {
												("USDC", SOLANA_USDT_MINT_ADDRESS)
											},
											SOLANA_USDT_MINT_ADDRESS => {
												("USDT", SOLANA_USDC_MINT_ADDRESS)
											},
											_ => {
												log::error!(
													"Unsupported SPL token: {:?}",
													mint_address
												);
												return Err(());
											},
										}
									},
								};
								("SOL".to_string(), asset.to_string(), token_address.to_string())
							},
							ChainAsset::Ethereum(..) => {
								log::error!("Unsupported from_asset: {:?}", swap_order.from_asset);
								return Err(());
							},
						};

					log::debug!(
						"from_network_name: {}, binance_coin_name: {}, token_address: {}",
						from_network_name,
						binance_coin_name,
						token_address
					);

					let Some(binance_coin_info) =
						coins_info.iter().find(|c| c.coin == binance_coin_name)
					else {
						log::error!(
							"Failed to find binance network list for asset: {:?}",
							binance_coin_name
						);
						return Err(());
					};
					let Some(binance_network_info) = binance_coin_info
						.network_list
						.iter()
						.find(|n| n.network == from_network_name)
					else {
						log::error!(
							"Failed to find binance network list for asset: {:?}",
							binance_coin_name
						);
						return Err(());
					};

					let deposit_address = self
						.binance_api
						.wallet()
						.get_deposit_address(&binance_coin_name, &binance_network_info.network)
						.await
						.map_err(|_| {
							log::error!("Failed to get deposit address");
						})?;

					let from_amount_decimal = Decimal::from_str(&from_amount).map_err(|_| {
						log::error!("Failed to parse from_amount_string");
					})?;

					log::debug!(
						"Binance deposit address: {}, from_amount: {}",
						deposit_address,
						from_amount
					);

					let asset_decimal_multiplier = match binance_coin_name.as_str() {
						"USDC" => Decimal::from(1_000_000),    // 10^6
						"USDT" => Decimal::from(1_000_000),    // 10^6
						"SOL" => Decimal::from(1_000_000_000), // 10^9  TODO: double check this
						_ => {
							log::error!("Unsupported asset: {:?}", binance_coin_name);
							return Err(());
						},
					};
					let amount_to_transfer_decimal = from_amount_decimal * asset_decimal_multiplier;
					let Some(amount_to_transfer) = amount_to_transfer_decimal.to_u64() else {
						log::error!("Failed to convert amount to transfer to u64");
						return Err(());
					};

					let remote_signer = RemoteSigner::new(
						self.pumpx_signer_client.clone(),
						pumpx_config.wallet_index,
						*account_id.as_ref(),
						Handle::current(),
					);

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
								log::error!("Failed to transfer SOL");
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
							.transfer_spl(
								&deposit_address,
								amount_to_transfer,
								&token_address,
								&remote_signer,
							)
							.await
							.map_err(|_| {
								log::error!("Failed to transfer SPL");
							})?;
						tx_id = Some(signature);
					}

					let (trade_symbol, order_side) = match binance_coin_name.as_str() {
						"USDC" => ("BNBUSDC".to_string(), BinanceOrderSide::BUY),
						"USDT" => ("BNBUSDT".to_string(), BinanceOrderSide::BUY),
						"SOL" => ("SOLBNB".to_string(), BinanceOrderSide::SELL),
						_ => {
							log::error!("Unsupported asset: {:?}", binance_coin_name);
							return Err(());
						},
					};

					let mut bnb_to_receive = "".to_string();

					if should_wait_for_deposit_confirm {
						debug!("Waiting for deposit to be confirmed on Binance...");
						debug!("Deposit tx_id: {:?}", tx_id);
						debug!("Source address: {:?}", from_address);
						let mut deposit_confirmed = false;
						let start_time = std::time::Instant::now();
						let timeout = Duration::from_secs(300); // 5 minute timeout

						while !deposit_confirmed && start_time.elapsed() < timeout {
							let Ok(deposit_history) = self
								.binance_api
								.wallet()
								.get_deposit_history(Some(binance_coin_name.clone()), tx_id.clone())
								.await
							else {
								log::error!("Failed to get deposit history");
								continue;
							};

							// Check if there's a recent successful deposit
							for deposit in deposit_history {
								debug!("Deposit: {:?}", deposit);
								if deposit.status == 2 || deposit.status == 7 {
									// 2 = rejected, 7 = Wrong Deposit
									log::error!("Deposit failed with status: {}", deposit.status);
									let body = CrossFailBody {
										request_id: intent_id,
										fail_reason: format!(
											"Deposit failed with status: {}",
											deposit.status
										),
									};
									self.pumpx_api.cross_fail(&access_token, body).await.map_err(
										|_| {
											log::error!("Failed to notify pumpx-signer");
										},
									)?;
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
							log::error!("Deposit not confirmed within timeout period");
							let body = CrossFailBody {
								request_id: intent_id,
								fail_reason: "Deposit not confirmed on Binance".to_string(),
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;
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
						let Ok(binance_order) = self
							.binance_api
							.spot_trading()
							.create_order(binance_order_params)
							.await
						else {
							log::error!("Failed to create binance order");
							self.binance_api
								.wallet()
								.withdraw(
									&binance_coin_name,
									&from_address,
									from_amount,
									Some(&binance_network_info.network),
								)
								.await
								.map_err(|e| {
									log::error!(
									"Failed to withdraw asset back to omni account, error: {:?}",
									e
								);
								})?;
							log::debug!("Withdrawed asset back to omni account");

							let body = CrossFailBody {
								request_id: intent_id,
								// TODO: is this a user facing error? what should we return?
								fail_reason: "Failed to create binance order".to_string(),
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;

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
						let mut bnb_received = "".to_string();
						loop {
							let trade_order = self
								.binance_api
								.spot_trading()
								.get_order(&trade_symbol, Some(binance_order.order_id), None, None)
								.await
								.map_err(|_| {
									log::error!("Failed to get binance order");
								})?;

							match trade_order.status {
								BinanceOrderStatus::FILLED => {
									log::info!("Binance order filled");
									bnb_received = match trade_order.side {
										BinanceOrderSide::BUY => trade_order.executed_qty,
										BinanceOrderSide::SELL => trade_order.cummulative_quote_qty,
									};
									trade_success = true;
									break;
								},
								BinanceOrderStatus::CANCELED => {
									log::error!("Binance order canceled");
								},
								BinanceOrderStatus::REJECTED => {
									log::error!("Binance order rejected");
								},
								BinanceOrderStatus::EXPIRED => {
									log::error!("Binance order expired");
								},
								BinanceOrderStatus::EXPIRED_IN_MATCH => {
									log::error!("Binance order expired in matching");
								},
								_ => {
									log::debug!("Binance order status: {:?}", trade_order.status);
								},
							}
							//todo: how long we wait ?
							sleep(Duration::from_millis(500)).await;
						}
						if !trade_success {
							log::error!("Binance order failed");
							self.binance_api
								.wallet()
								.withdraw(
									&binance_coin_name,
									&from_address,
									from_amount,
									Some(&binance_network_info.network),
								)
								.await
								.map_err(|e| {
									log::error!(
									"Failed to withdraw asset back to omni account, error: {:?}",
									e
								);
								})?;
							log::debug!("Withdrawed asset back to omni account");

							let body = CrossFailBody {
								request_id: intent_id,
								// TODO: is this a user facing error? what should we return?
								fail_reason: "Binance order failed".to_string(),
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;

							return Err(());
						}

						debug!("Total received {} bnb", bnb_received);
					} else {
						// Estimate BNB payout (simulate spot trade, apply service fee)
						let price_str = self
							.binance_api
							.spot_trading()
							.get_symbol_price(&trade_symbol)
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
						let service_fee_rate =
							Decimal::from_str("0.001").expect("Failed to parse service fee rate");
						let decimal_bnb_to_receive =
							bnb_estimated * (Decimal::ONE - service_fee_rate);
						bnb_to_receive = decimal_bnb_to_receive.to_string();

						debug!(
							"BNB estimated: {}, after 0.1% fee: {}",
							bnb_estimated, bnb_to_receive
						);
					}

					let payout_address = Address::from_str(&to_address).map_err(|_| {
						log::error!("Failed to parse payout address");
					})?;
					let payout_amount = match str_to_u256(&bnb_to_receive, 18) {
						Some(a) => a,
						None => {
							log::error!("Fail to convert bnb amount {} to U256", bnb_to_receive);
							let body = CrossFailBody {
								request_id: intent_id,
								fail_reason:
									"Fail to construct payout request due to U256 conversion error"
										.to_string(),
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;
							// TODO: what to do with user asset?
							return Err(());
						},
					};

					debug!("Getting {:?} nonce for payout request", payout_address);
					// 4. Call accounting contract on BSC
					let user_nonce =
						self.accounting_contract_client.get_nonce(payout_address).await.map_err(
							|_| {
								log::error!("Failed to get nonce");
							},
						)?;

					debug!("Received {:?} nonce", user_nonce);
					let user_nonce = user_nonce + U256::from(1u64);
					debug!("Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}", payout_address, user_nonce, payout_amount);

					self.accounting_contract_client
						.execute_pay_out_request(payout_address, user_nonce, payout_amount)
						.await
						.map_err(|_| {
							log::error!("Failed to execute pay out request");
						})?;

					debug!("Calling pumpx get_gas_info, chain_id: {}", pumpx_config.to_chain_id);
					let res = self
						.pumpx_api
						.get_gas_info(&access_token, pumpx_config.to_chain_id)
						.await
						.map_err(|_| {
							log::error!("Failed to get gas info");
						})?;
					debug!("Response get_gas_info: {:?}", res);

					let Some(gas_info_data) = res.data else {
						log::error!("Response data of call get gas info is none");
						return Err(());
					};
					let Some(gas_info) = gas_info_data
						.gas_info
						.iter()
						.find(|g| g.chain_id == pumpx_config.to_chain_id.to_string())
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

					let amount_in = match calculate_amount_in(&bnb_to_receive, gas_fee) {
						Some(a) => a,
						None => {
							log::error!(
								"Fail to calculate amount_in from amount {}, gas {}",
								bnb_to_receive,
								gas_fee
							);
							let body = CrossFailBody {
								request_id: intent_id,
								fail_reason: "Fail to calculate amount_in".to_string(),
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;
							// TODO: what to do with user asset?
							return Err(());
						},
					};

					debug!("Doing market order");
					let body = CreateMarketOrderTxBody {
						request_id: intent_id,
						chain_id: pumpx_config.to_chain_id,
						token_ca: to_token_ca.clone(),
						swap_type: match pumpx_config.swap_type {
							1 => SwapType::Buy,
							2 => SwapType::Sell,
							_ => {
								log::error!("Unsupported swap type: {}", pumpx_config.swap_type);
								return Err(());
							},
						},
						amount_in,
						double_out: pumpx_config.double_out,
						is_one_click: pumpx_config.is_one_click,
						address: pubkey_to_evm_address(&to_wallet_address)?,
						is_anti_mev: pumpx_config.is_anti_mev,
						is_auto_slippage: pumpx_config.is_auto_slippage,
						gas_type: match pumpx_config.gas_type {
							1 => GasType::Slow,
							2 => GasType::Medium,
							3 => GasType::Fast,
							_ => {
								log::error!("Unsupported gas type: {}", pumpx_config.gas_type);
								return Err(());
							},
						},
						slippage: pumpx_config.slippage,
						wallet_index: pumpx_config.wallet_index,
					};
					debug!("Calling pumpx create_market_order_tx, body: {:?}", body);
					let response =
						self.pumpx_api.create_market_order_tx(&access_token, body).await.map_err(
							|_| {
								log::error!("Failed to create market order tx");
							},
						)?;

					debug!("Response create_market_order_tx: {:?}", response);
					result = (Some(response.encode()), true);
				}

				// self.account_asset_lock.release(
				// 	account_id.clone(),
				// 	swap_order.from_asset.clone(),
				// 	AmountType::from_str_radix(&from_amount_string, 10).map_err(|_| {
				// 		log::error!("Failed to parse from_amount_string");
				// 	})?,
				// )?;

				return Ok(result);
			},
			_ => {
				log::error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}
	}
}

fn str_to_u256(amount: &str, decimals: u32) -> Option<U256> {
	let amount = Decimal::from_str(amount).ok()?;
	let factor = Decimal::from(10u64.pow(decimals));
	let scaled = amount * factor;
	let int_str = scaled.trunc().to_string();
	U256::from_str(&int_str).ok()
}

fn calculate_amount_in(amount: &str, gas: &str) -> Option<String> {
	let amount = Decimal::from_str(amount).ok()?;
	let gas = Decimal::from_str(gas).ok()?;
	if amount <= gas {
		None
	} else {
		Some((amount - gas).to_string())
	}
}
