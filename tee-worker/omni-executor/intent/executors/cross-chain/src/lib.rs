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

use async_trait::async_trait;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::ChainAsset;
use executor_primitives::Intent;
use executor_primitives::IntentId;
use executor_primitives::PumpxConfig;
use executor_primitives::PumpxOrderType;
use executor_primitives::SingleChainSwapProvider;
use executor_primitives::SwapOrder;
use executor_storage::StorageDB;
use executor_storage::{PumpxJwtStorage, Storage};
use heima_authentication::auth_token::AUTH_TOKEN_ACCESS_TYPE;
use intent_asset_lock::AmountType;
use intent_token_query::query_ethereum;
use intent_token_query::query_solana;
use intent_token_query::EthereumAddress;
use intent_token_query::SolanaPubkey;
use log::error;
use pumpx::signer_client::ChainType;
use pumpx::signer_client::SignerClient;
use pumpx::types::ChainId;
use pumpx::types::CreateCrossOrderData;
use pumpx::types::CrossOrderInfo;
use pumpx::types::GasType;
use pumpx::types::MarketOrderTx;
use pumpx::types::NewMarketOrder;
use pumpx::types::SwapType;
use pumpx::PumpxApi;
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

use intent_asset_lock::always_unlocked::AlwaysUnlockedAssetsLock;
use intent_asset_lock::AccountAssetLocks;

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

pub struct CrossChainIntentExecutor<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
> {
	parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	transaction_signer: Arc<ParentchainTxSigner>,
	account_asset_lock: AccountAssetLocks<AlwaysUnlockedAssetsLock>,
	rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<SignerClient>,
	pumpx_api: Arc<PumpxApi>,
	storage_db: Arc<StorageDB>,
	phantom: PhantomData<(Header, RpcClient)>,
}

impl<
		Header,
		RpcClient: SubstrateRpcClient<Header>,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	> CrossChainIntentExecutor<Header, RpcClient, RpcClientFactory>
{
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
		rpc_endpoint_registry: RpcEndpointRegistry,
		pumpx_signer_client: Arc<SignerClient>,
		pumpx_api: Arc<PumpxApi>,
		storage_db: Arc<StorageDB>,
	) -> Result<Self, ()> {
		// there is no need for account/assets locks if we guarantee the dest-chain payout happens after the source chain finalisation
		let account_asset_lock = AccountAssetLocks::<AlwaysUnlockedAssetsLock>::empty();
		Ok(Self {
			parentchain_rpc_client_factory,
			transaction_signer,
			account_asset_lock,
			rpc_endpoint_registry,
			pumpx_signer_client,
			pumpx_api,
			storage_db,
			phantom: PhantomData,
		})
	}
}

#[async_trait]
impl<
		Header: Send + Sync,
		RpcClient: SubstrateRpcClient<Header> + Send + Sync,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync,
	> IntentExecutor for CrossChainIntentExecutor<Header, RpcClient, RpcClientFactory>
{
	async fn execute(
		&self,
		account_id: &AccountId,
		intent_id: IntentId,
		intent: Intent,
	) -> Result<(), ()> {
		match intent {
			Intent::Swap(ref swap_order, ref ccsp, ref scsp) => {
				let Ok(mut rpc_client) = self.parentchain_rpc_client_factory.new_client().await
				else {
					log::error!("Failed to create rpc client");
					return Err(());
				};
				let available_amount = match &swap_order.from_asset {
					ChainAsset::Ethereum(chain_id, token) => {
						let rpc_url =
							self.rpc_endpoint_registry.get(&Chain::Ethereum(*chain_id)).ok_or(())?;
						let address = self
							.pumpx_signer_client
							.request_wallet(
								pumpx::signer_client::ChainType::Evm,
								0,
								*account_id.as_ref(),
							)
							.await
							.map_err(|e| {
								error!("Could not get wallet from pumpx-signer: {:?}", e)
							})?;
						query_ethereum(rpc_url, EthereumAddress::from_slice(&address), token)
							.await?
					},
					ChainAsset::Solana(token) => {
						let rpc_url = self.rpc_endpoint_registry.get(&Chain::Solana).ok_or(())?;
						let address = self
							.pumpx_signer_client
							.request_wallet(
								pumpx::signer_client::ChainType::Solana,
								0,
								*account_id.as_ref(),
							)
							.await
							.map_err(|e| {
								error!("Could not get wallet from pumpx-signer: {:?}", e)
							})?;
						let pubkey = SolanaPubkey::try_from(address).map_err(|e| {
							error!("Could not create solana pubkey from wallet address: {:?}", e)
						})?;
						query_solana(rpc_url, &pubkey, token).await.map(|v| AmountType::from(v))?
					},
				};
				self.account_asset_lock.check_and_insert(
					account_id.clone(),
					swap_order.from_asset.clone(),
					AmountType::from(swap_order.from_amount),
					available_amount,
				)?;

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

				let storage = PumpxJwtStorage::new(self.storage_db.clone());
				let Some(access_token) = storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get access token from storage");
					return Err(());
				};
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

				let cross_order_data = CreateCrossOrderData {
					request_id: intent_id,
					chain_id: chain_id.clone(),
					info: CrossOrderInfo {
						chain_id: chain_id.clone(), // TODO: is this the same chain_id as the one above?
						wallet_index: pumpx_config.wallet_index,
						address: "todo: what's this??".to_string(),
						amount: "what amount to use?".to_string(),
						usd: usd_worth,
						token_ca: token_ca.clone(),
					},
				};
				// TODO: should this be called regardless of the kind of swap? (cross-chain |
				// single-chain)
				self.pumpx_api
					.create_cross_order(&access_token, cross_order_data)
					.await
					.map_err(|_| {
						log::error!("Failed to create cross order");
					})?;

				if swap_order.from_asset.is_same_chain(&swap_order.to_asset) {
					match pumpx_config.order_type {
						PumpxOrderType::Market => {
							// - call `/v3/account/get_user_trade_info` to get user config, mainly `gasType`, `isAntiMev`, `isAutoSlippage`, `slippage` - for gasType we need to select the right one based on the chain
							let user_trade_info =
								self.pumpx_api.get_user_trade_info(&access_token).await.map_err(
									|_| {
										log::error!("Failed to get user trade info");
									},
								)?;

							// - call `/v3/trade/create_market_order_unsigned_tx` , for parameters that are not returned from `get_user_trade_info` , they should be passed in via RPC already
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
								amount_in: swap_order.from_amount.to_string(),
								double_out: pumpx_config.double_out,
								is_one_click: pumpx_config.is_one_click,
								address: "todo: what's this??".to_string(),
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

							// - ask pumpx-tee-signer to sign the returned payload (I assume it’s `txData`)
							// TODO: do we need to sign the tx here?
							//
							// let tx_data = market_order_unsigned_tx.data.tx_data;
							// let mut messages_to_sign = Vec::new();
							// for tx in tx_data {
							// 	let tx_cleaned = tx.strip_prefix("0x").unwrap_or(&tx);
							// 	let tx_bytes = match hex::decode(tx_cleaned) {
							// 		Ok(bytes) => bytes,
							// 		Err(e) => {
							// 			log::error!("Failed to decode hex string: {:?}", e);
							// 			return Err(());
							// 		},
							// 	};
							// 	messages_to_sign.push(tx_bytes);
							// }
							//
							// let Some(chain_type) =
							// 	ChainType::from_pumpx_chain_id(chain_id.to_number() as u32)
							// else {
							// 	log::error!("Unsupported chain id: {:?}", chain_id);
							// 	return Err(());
							// };
							//
							// let signatures = match self
							// 	.pumpx_signer_client
							// 	.request_signatures(
							// 		chain_type,
							// 		pumpx_config.wallet_index,
							// 		*account_id.as_ref(),
							// 		messages_to_sign,
							// 	)
							// 	.await
							// {
							// 	Ok(sigs) => sigs,
							// 	Err(e) => {
							// 		log::error!(
							// 			"Failed to get signatures from pumpx-signer: {:?}",
							// 			e
							// 		);
							// 		return Err(());
							// 	},
							// };
							// let signed_tx_data: Vec<String> =
							// 	signatures.into_iter().map(hex::encode).collect();

							let market_order_tx = MarketOrderTx {
								order_id: market_order_unsigned_tx.data.order_id,
								tx_data: market_order_unsigned_tx.data.tx_data,
								chain_id,
							};
							// - call `/v3/trade/send_order_tx` to submit it
							let market_order_tx_res = self
								.pumpx_api
								.send_market_order_tx(&access_token, market_order_tx)
								.await
								.map_err(|_| {
									log::error!("Failed to send market order tx");
								})?;
							// - return the result to F/E
							// TODO: figure out how to send this to the frontend
						},
						PumpxOrderType::Limit => {
							todo!()
						},
					}
				} else {
					//TODO: execute cross-chain swap
					// to binance swap, If it fails, notify the backend via /v3/trade/cross_fail
					todo!()
				}

				// TODO:
				// 3. Swap assets:
				//    - Call accounting contract (e.g Swap SOL to TRUMP)
				//    - Call Binance convert via binance account (e.g Swap USDC to SOL)
				// 4. Send locked balance to binance account (refill)
				self.account_asset_lock.release(
					account_id.clone(),
					swap_order.from_asset.clone(),
					AmountType::from(swap_order.from_amount),
				)?;
				todo!("CrossChainSwap is not implemented yet");
			},
			_ => {
				log::error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}
	}
}
