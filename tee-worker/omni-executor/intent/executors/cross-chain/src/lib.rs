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
use executor_core::intent_executor::IntentExecutor;
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
use std::marker::PhantomData;
use std::sync::Arc;

use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use parentchain_rpc_client::metadata::Metadata;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::CustomConfig;
use parentchain_rpc_client::SubstrateRpcClient;
use parentchain_rpc_client::SubstrateRpcClientFactory;
use parentchain_rpc_client::SubxtClient;
use parentchain_rpc_client::SubxtClientFactory;
use parentchain_rpc_client::ToSubxtType;
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
pub struct CrossChainIntentExecutor<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	Provider: EthereumRpcProvider<Transaction = TransactionRequest>,
> {
	parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	transaction_signer: Arc<ParentchainTxSigner>,
	// account_asset_lock: AccountAssetLocks<AlwaysUnlockedAssetsLock>,
	// rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pumpx_api: Arc<PumpxApi>,
	storage_db: Arc<StorageDB>,
	binance_api: Arc<BinanceApi>,
	solana_client: Arc<SolanaClient>,
	accounting_contract_client: Arc<AccountingContractClient<Provider>>,
	phantom: PhantomData<(Header, RpcClient)>,
}

impl<
		Header,
		RpcClient: SubstrateRpcClient<Header>,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
		Provider: EthereumRpcProvider<Transaction = TransactionRequest>,
	> CrossChainIntentExecutor<Header, RpcClient, RpcClientFactory, Provider>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
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
			parentchain_rpc_client_factory,
			transaction_signer,
			// account_asset_lock,
			// rpc_endpoint_registry,
			pumpx_signer_client,
			pumpx_api,
			storage_db,
			binance_api,
			solana_client,
			accounting_contract_client,
			phantom: PhantomData,
		})
	}
}

#[async_trait]
impl<
		Header: Send + Sync,
		RpcClient: SubstrateRpcClient<Header> + Send + Sync,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync,
		Provider: EthereumRpcProvider<Transaction = TransactionRequest> + Send + Sync,
	> IntentExecutor for CrossChainIntentExecutor<Header, RpcClient, RpcClientFactory, Provider>
{
	#[allow(unused_assignments)]
	async fn execute(
		&self,
		account_id: &AccountId,
		intent_id: IntentId,
		intent: Intent,
	) -> Result<Option<Vec<u8>>, ()> {
		match intent {
			Intent::Swap(ref swap_order, ref _ccsp, ref scsp) => {
				debug!("Started processing SwapOrder intent, order: {:?}, signle chain swap provider: {:?}", swap_order, scsp);
				let Ok(mut rpc_client) = self.parentchain_rpc_client_factory.new_client().await
				else {
					log::error!("Failed to create rpc client");
					return Err(());
				};
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

				let intent_accepted_event_emit_call = parentchain_api_interface::tx()
					.omni_account()
					.intent_accepted(account_id.to_subxt_type(), intent_id, intent.to_subxt_type());

				let tx = self.transaction_signer.sign(intent_accepted_event_emit_call).await;

				match rpc_client.submit_tx(&tx).await {
					Ok(report) => report,
					Err(e) => {
						log::error!("Failed to submit and watch tx: {:?}", e);
						return Err(());
					},
				};

				debug!("Submitted intent accepted parentchain call");

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
				let Some(access_token) = storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get access token from storage");
					return Err(());
				};

				let pumpx_order_response: Option<Vec<u8>>;

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

					let order_response = match pumpx_config.order_type {
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
							debug!("Sending market order: {:?}", body);
							let res = self
								.pumpx_api
								.create_market_order_tx(&access_token, body)
								.await
								.map_err(|_| {
									log::error!("Failed to create market order tx");
								})?;

							debug!("Received create_market_order_tx response: {:?}", res);
							res.encode()
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
							debug!("Sending limit order: {:?}", new_limit_order);
							let limit_order_res = self
								.pumpx_api
								.create_limit_order(&access_token, new_limit_order)
								.await
								.map_err(|_| {
									log::error!("Failed to create limit order");
								})?;

							debug!("Received limit order response: {:?}", limit_order_res);

							limit_order_res.encode()
						},
					};
					pumpx_order_response = Some(order_response);
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

					let from_address = match from_chain_type {
						ChainType::Evm => pubkey_to_evm_address(&from_wallet_address)?,
						ChainType::Solana => pubkey_to_solana_address(&from_wallet_address)?,
						_ => {
							log::error!("Unsupported {:?} wallet address", from_chain_type);
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
					debug!("Creating cross order with data: {:?}", body);
					let response =
						self.pumpx_api.create_cross_order(&access_token, body).await.map_err(
							|_| {
								log::error!("Failed to create cross order");
							},
						)?;
					debug!("Received response: {:?}", response);

					// 2. transfer from_asset to binance deposit address
					let coins_info =
						self.binance_api.wallet().get_all_coins_info().await.map_err(|_| {
							log::error!("Failed to get all coins info");
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

					// TODO: change this when adding support for more tokens/chains
					if binance_coin_name == "SOL" {
						// Native transfer
						debug!("Transfering {:?} SOL to {:?}", amount_to_transfer, deposit_address);
						self.solana_client
							.transfer_sol(&deposit_address, amount_to_transfer, &remote_signer)
							.await
							.map_err(|_| {
								log::error!("Failed to transfer SOL");
							})?;
					} else {
						debug!(
							"Transfering {:?} {:?} to {:?}",
							amount_to_transfer, token_address, deposit_address
						);
						// SPL transfer
						self.solana_client
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
					}

					debug!("Waiting for deposit to be confirmed on Binance...");
					let mut deposit_confirmed = false;
					let start_time = std::time::Instant::now();
					let timeout = Duration::from_secs(300); // 5 minute timeout

					while !deposit_confirmed && start_time.elapsed() < timeout {
						let Ok(deposit_history) = self
							.binance_api
							.wallet()
							.get_deposit_history(Some(binance_coin_name.clone()))
							.await
						else {
							log::error!("Failed to get deposit history");
							continue;
						};

						// Check if there's a recent successful deposit
						for deposit in deposit_history {
							let deposit_amount: u64 = match Decimal::from_str(&deposit.amount) {
								Ok(deposit_amount) => deposit_amount.to_u64().unwrap_or(0),
								Err(_) => {
									log::error!("Failed to parse deposit amount");
									continue;
								},
							};
							if deposit.status == 1 && // 1 = success
							   deposit.coin == binance_coin_name &&
							   deposit.network == binance_network_info.network &&
                               deposit.source_address == Some(from_address.clone()) &&
                               deposit_amount == amount_to_transfer
							{
								deposit_confirmed = true;
								debug!(
									"Deposit confirmed on Binance for {} {}",
									deposit.amount, binance_coin_name
								);
								break;
							}
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

					let (trade_symbol, order_side) = match binance_coin_name.as_str() {
						"USDC" => ("BNBUSDC".to_string(), BinanceOrderSide::BUY),
						"USDT" => ("BNBUSDT".to_string(), BinanceOrderSide::BUY),
						"SOL" => ("SOLBNB".to_string(), BinanceOrderSide::SELL),
						_ => {
							log::error!("Unsupported asset: {:?}", binance_coin_name);
							return Err(());
						},
					};

					// 3. Make the trade using binance spot trading api from_asset => BNB, If it fails, notify the backend via /v3/trade/cross_fail
					let binance_order_params = BinanceCreateOrderParams {
						symbol: trade_symbol.clone(),
						side: order_side,
						order_type: BinanceOrderType::MARKET,
						..Default::default()
					};
					debug!("Creating binance order with params: {:?}", binance_order_params);
					let Ok(binance_order) =
						self.binance_api.spot_trading().create_order(binance_order_params).await
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
						let body = CrossFailBody {
							request_id: intent_id,
							// TODO: is this a user facing error? what should we return?
							fail_reason: "Binance order failed".to_string(),
						};
						self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
							log::error!("Failed to notify pumpx-signer");
						})?;
						// TODO: Figure out how to transfer back the asset to the omni account
						// check https://developers.binance.com/docs/wallet/capital/withdraw

						return Err(());
					}

					debug!("Total received {} bnb", bnb_received);

					let payout_address: Address = Address::from_slice(&to_wallet_address);
					let payout_amount = match str_to_u256(&bnb_received, 18) {
						Some(a) => a,
						None => {
							log::error!("Fail to convert bnb amount {} to U256", bnb_received);
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
					debug!("Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}", payout_address, user_nonce, payout_amount);

					self.accounting_contract_client
						.execute_pay_out_request(payout_address, user_nonce, payout_amount)
						.await
						.map_err(|_| {
							log::error!("Failed to execute pay out request");
						})?;

					debug!("Quering gas info");
					let gas_info = self
						.pumpx_api
						.get_gas_info(&access_token, pumpx_config.to_chain_id)
						.await
						.map_err(|_| {
							log::error!("Failed to get gas info");
						})?;
					let gas_fee = match pumpx_config.gas_type {
						1 => gas_info.data.gas_info.normal,
						2 => gas_info.data.gas_info.fast,
						3 => gas_info.data.gas_info.super_fast,
						_ => {
							log::error!("Unsupported gas type: {}", pumpx_config.gas_type);
							return Err(());
						},
					};
					debug!("Gas fee for chain_id {} is {}", pumpx_config.to_chain_id, gas_fee);

					let amount_in = match calculate_amount_in(&bnb_received, &gas_fee) {
						Some(a) => a,
						None => {
							log::error!(
								"Fail to calculate amount_in from amount {}, gas {}",
								bnb_received,
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
					debug!("Sending market order: {:?}", body);
					let res =
						self.pumpx_api.create_market_order_tx(&access_token, body).await.map_err(
							|_| {
								log::error!("Failed to create market order tx");
							},
						)?;

					debug!("Received create_market_order_tx response: {:?}", res);
					pumpx_order_response = Some(res.encode())
				}

				// self.account_asset_lock.release(
				// 	account_id.clone(),
				// 	swap_order.from_asset.clone(),
				// 	AmountType::from_str_radix(&from_amount_string, 10).map_err(|_| {
				// 		log::error!("Failed to parse from_amount_string");
				// 	})?,
				// )?;

				return Ok(pumpx_order_response);
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
