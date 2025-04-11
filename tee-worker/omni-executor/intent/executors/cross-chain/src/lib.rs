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
use executor_primitives::ChainAsset;
use executor_primitives::Intent;
use executor_primitives::IntentId;
use executor_primitives::PumpxOrderType;
use executor_primitives::SingleChainSwapProvider;
use executor_primitives::SolanaToken;
use executor_primitives::{utils::hex::ToHexPrefixed, HeimaMultiAddress};
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
use pumpx::types::ChainId;
use pumpx::types::CreateCrossOrderData;
use pumpx::types::CrossOrderInfo;
use pumpx::types::GasType;
use pumpx::types::MarketOrderTx;
use pumpx::types::NewLimitOrder;
use pumpx::types::NewMarketOrder;
use pumpx::types::SwapType;
use pumpx::PumpxApi;
use pumpx::{signer_client::ChainType, types::CrossOrderFailData};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use executor_primitives::AccountId;
use parentchain_rpc_client::metadata::Metadata;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::CustomConfig;
use parentchain_rpc_client::SubstrateRpcClient;
use parentchain_rpc_client::SubstrateRpcClientFactory;
use parentchain_rpc_client::SubxtClient;
use parentchain_rpc_client::SubxtClientFactory;
use parentchain_rpc_client::ToSubxtType;
use parentchain_signer::TxSigner;
use pumpx::signer_client::SignerClient;

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
	async fn execute(
		&self,
		account_id: &AccountId,
		intent_id: IntentId,
		intent: Intent,
	) -> Result<Option<Vec<u8>>, ()> {
		match intent {
			Intent::Swap(ref swap_order, ref _ccsp, ref scsp) => {
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

				let from_amount_string = std::str::from_utf8(&swap_order.from_amount)
					.map_err(|_| {
						log::error!("Failed to parse from_amount_string");
					})
					.map(|v| v.to_string())?;

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

				// TODO: update this when we have more providers
				let SingleChainSwapProvider::Pumpx(pumpx_config) = scsp;

				let chain_id = match swap_order.to_asset {
					ChainAsset::Ethereum(..) => ChainId::EVM,
					ChainAsset::Solana(_) => ChainId::Solana,
				};
				let usd_worth = std::str::from_utf8(&pumpx_config.usd_worth)
					.map_err(|_| {
						log::error!("Failed to parse usd_worth");
					})
					.map(|v| v.to_string())?;
				let token_ca = std::str::from_utf8(&pumpx_config.token_ca)
					.map_err(|_| {
						log::error!("Failed to parse token_ca");
					})
					.map(|v| v.to_string())?;
				let Some(chain_type) = ChainType::from_pumpx_chain_id(chain_id.to_number() as u32)
				else {
					log::error!("Unsupported chain id: {:?}", chain_id);
					return Err(());
				};

				let wallet_address = self
					.pumpx_signer_client
					.request_wallet(chain_type, pumpx_config.wallet_index, *account_id.as_ref())
					.await
					.map_err(|e| log::error!("Could not get wallet from pumpx-signer: {:?}", e))?;

				let storage = PumpxJwtStorage::new(self.storage_db.clone());
				let Some(access_token) = storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get access token from storage");
					return Err(());
				};

				let pumpx_order_response: Option<Vec<u8>>;

				if swap_order.from_asset.is_same_chain(&swap_order.to_asset) {
					let user_trade_info =
						self.pumpx_api.get_user_trade_info(&access_token).await.map_err(|_| {
							log::error!("Failed to get user trade info");
						})?;

					let order_response = match pumpx_config.order_type {
						PumpxOrderType::Market => {
							let new_market_order = NewMarketOrder {
								request_id: intent_id,
								chain_id: chain_id.clone(),
								token_ca,
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
								amount_in: from_amount_string.clone(),
								double_out: pumpx_config.double_out,
								is_one_click: pumpx_config.is_one_click,
								address: wallet_address.to_hex(),
								is_anti_mev: user_trade_info.data.is_anti_mev,
								is_auto_slippage: user_trade_info.data.is_auto_slippage,
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
								slippage: user_trade_info.data.slippage,
								wallet_index: pumpx_config.wallet_index,
							};
							let market_order_unsigned_tx = self
								.pumpx_api
								.create_market_order_unsigned_tx(&access_token, new_market_order)
								.await
								.map_err(|_| {
									log::error!("Failed to create market order unsigned tx");
								})?;

							let tx_data = market_order_unsigned_tx.data.tx_data;
							let mut messages_to_sign = Vec::new();
							for tx in tx_data {
								let tx_cleaned = tx.strip_prefix("0x").unwrap_or(&tx);
								let tx_bytes = match hex::decode(tx_cleaned) {
									Ok(bytes) => bytes,
									Err(e) => {
										log::error!("Failed to decode hex string: {:?}", e);
										return Err(());
									},
								};
								messages_to_sign.push(tx_bytes);
							}

							let Some(chain_type) =
								ChainType::from_pumpx_chain_id(chain_id.to_number() as u32)
							else {
								log::error!("Unsupported chain id: {:?}", chain_id);
								return Err(());
							};

							let signatures = match self
								.pumpx_signer_client
								.request_signatures(
									chain_type,
									pumpx_config.wallet_index,
									*account_id.as_ref(),
									messages_to_sign,
								)
								.await
							{
								Ok(sigs) => sigs,
								Err(e) => {
									log::error!(
										"Failed to get signatures from pumpx-signer: {:?}",
										e
									);
									return Err(());
								},
							};
							let signed_tx_data: Vec<String> = signatures
								.into_iter()
								.map(|signature| signature.to_hex())
								.collect();

							let market_order_tx = MarketOrderTx {
								order_id: market_order_unsigned_tx.data.order_id,
								tx_data: signed_tx_data,
								chain_id: chain_id.clone(),
							};
							let market_order_tx_res = self
								.pumpx_api
								.send_market_order_tx(&access_token, market_order_tx)
								.await
								.map_err(|_| {
									log::error!("Failed to send market order tx");
								})?;
							market_order_tx_res.encode()
						},
						PumpxOrderType::Limit => {
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

							let new_limit_order = NewLimitOrder {
								request_id: intent_id,
								chain_id: chain_id.clone(),
								token_ca,
								amount: from_amount_string.clone(),
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
								address: wallet_address.to_hex(),
								is_anti_mev: user_trade_info.data.is_anti_mev,
								is_auto_slippage: user_trade_info.data.is_auto_slippage,
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
								slippage: user_trade_info.data.slippage,
								wallet_index: pumpx_config.wallet_index,
							};
							let limit_order_response = self
								.pumpx_api
								.create_limit_order(&access_token, new_limit_order)
								.await
								.map_err(|_| {
									log::error!("Failed to create limit order");
								})?;

							limit_order_response.encode()
						},
					};
					pumpx_order_response = Some(order_response);
				} else {
					// todo: should we allow for bsc testnet aswell ?

					if !matches!(
						swap_order.to_asset,
						ChainAsset::Ethereum(pumpx::constants::BSC_CHAIN_ID, _)
					) {
						log::error!("Only BSC payout supported");
					}

					let payout_address = match swap_order.to_address {
						Some(HeimaMultiAddress::Address20(address)) => {
							Address::from_slice(address.as_ref())
						},
						_ => {
							log::error!("Invalid payout address type");
							return Err(());
						},
					};

					// 1. Notify the backend
					// TODO: update params (the endpoint params have changed)
					let cross_order_data = CreateCrossOrderData {
						request_id: intent_id,
						chain_id: chain_id.clone(),
						info: vec![CrossOrderInfo {
							chain_id: chain_id.clone(),
							wallet_index: pumpx_config.wallet_index,
							address: wallet_address.to_hex(),
							amount: from_amount_string.clone(),
							usd: usd_worth,
							token_ca: token_ca.clone(),
						}],
					};
					self.pumpx_api
						.create_cross_order(&access_token, cross_order_data)
						.await
						.map_err(|_| {
							log::error!("Failed to create cross order");
						})?;

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
					let from_amount_decimal =
						Decimal::from_str(&from_amount_string).map_err(|_| {
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
						self.solana_client
							.transfer_sol(&deposit_address, amount_to_transfer, &remote_signer)
							.await
							.map_err(|_| {
								log::error!("Failed to transfer SOL");
							})?;
					} else {
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
					let Ok(binance_order) =
						self.binance_api.spot_trading().create_order(binance_order_params).await
					else {
						log::error!("Failed to create binance order");
						let data = CrossOrderFailData {
							request_id: intent_id,
							// TODO: is this a user facing error? what should we return?
							fail_reason: "Failed to create binance order".to_string(),
						};
						self.pumpx_api.cross_order_failed(&access_token, data).await.map_err(
							|_| {
								log::error!("Failed to notify pumpx-signer");
							},
						)?;
						// TODO: Figure out how to transfer back the asset to the omni account
						// check https://developers.binance.com/docs/wallet/capital/withdraw

						return Err(());
					};

					let mut trade_success = false;
					let mut to_amount = 0;
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
								trade_success = true;
								to_amount = 0; // TODO: figure out the amount received
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
						sleep(Duration::from_millis(500)).await;
					}
					if !trade_success {
						log::error!("Binance order failed");
						let data = CrossOrderFailData {
							request_id: intent_id,
							// TODO: is this a user facing error? what should we return?
							fail_reason: "Binance order failed".to_string(),
						};
						self.pumpx_api.cross_order_failed(&access_token, data).await.map_err(
							|_| {
								log::error!("Failed to notify pumpx-signer");
							},
						)?;
						// TODO: Figure out how to transfer back the asset to the omni account
						// check https://developers.binance.com/docs/wallet/capital/withdraw

						return Err(());
					}
					// 4. Call accounting contract on BSC
					let user_nonce =
						self.accounting_contract_client.get_nonce(payout_address).await.map_err(
							|_| {
								log::error!("Failed to get nonce");
							},
						)?;

					self.accounting_contract_client
						.execute_pay_out_request(payout_address, user_nonce, U256::from(to_amount))
						.await
						.map_err(|_| {
							log::error!("Failed to execute pay out request");
						})?;

					// 5. when it’s done, call pumpx API to submit the native trade (here it should be market order only.

					todo!()
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
