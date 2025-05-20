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

#![allow(unused_assignments)]
#![allow(clippy::too_many_arguments)]

use alloy::consensus::{SignableTransaction, TxLegacy};
use alloy::primitives::{Address, Signature, U256};
use async_trait::async_trait;
use base58::ToBase58;
use binance_api::spot_trading_api::types::{
	CreateOrderParams as BinanceCreateOrderParams, OrderSide as BinanceOrderSide,
	OrderStatus as BinanceOrderStatus, OrderType as BinanceOrderType,
};
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
use solana::{signer::RemoteSigner, SolanaClient as SolanaClientTrait};
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
use accounting_contract_client::AccountingContractApi;
use alloy::primitives::private::alloy_rlp::Decodable;
use binance_api::spot_trading_api::SpotTradingApi;
use binance_api::wallet_api::WalletApi;
use binance_api::BinanceApi;
use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use parentchain_rpc_client::metadata::Metadata;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::CustomConfig;
use parentchain_rpc_client::SubxtClient;
use parentchain_rpc_client::SubxtClientFactory;
use parentchain_signer::TxSigner;
use parity_scale_codec::Encode;
use pumpx::constants::*;
use pumpx::methods::common::{GasType, SwapType};
use pumpx::methods::create_cross_order::CreateCrossOrderBody;
use pumpx::methods::create_cross_order::CrossOrderInfo;
use pumpx::methods::create_limit_order::CreateLimitOrderBody;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use pumpx::methods::cross_fail::CrossFailBody;
use pumpx::methods::send_order_tx::SendOrderTxBody;
use pumpx::pubkey_to_address;
use pumpx::signer_client::ChainType;
use pumpx::signer_client::SignerClient;
use pumpx::PumpxApi;
use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error};

mod cross_chain;
mod single_chain;

#[cfg(test)]
mod cross_chain_swap_tests;
#[cfg(test)]
mod single_chain_swap_tests;

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
pub struct CrossChainIntentExecutor<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait> {
	// account_asset_lock: AccountAssetLocks<AlwaysUnlockedAssetsLock>,
	// rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pumpx_api: Arc<Box<dyn PumpxApi>>,
	storage_db: Arc<StorageDB>,
	binance_api: Arc<BinanceClient>,
	solana_client: Arc<SolanaClient>,
	accounting_contract_client: Arc<Box<dyn AccountingContractApi>>,
}

impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait>
	CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		_rpc_endpoint_registry: RpcEndpointRegistry,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		storage_db: Arc<StorageDB>,
		binance_api: Arc<BinanceClient>,
		solana_client: Arc<SolanaClient>,
		accounting_contract_client: Arc<Box<dyn AccountingContractApi>>,
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
impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait> IntentExecutor
	for CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
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

				let mut amount = std::str::from_utf8(&pumpx_config.from_amount)
					.map_err(|_| {
						error!("Failed to parse from_amount");
					})
					.map(|v| v.to_string())?;

				let storage = PumpxJwtStorage::new(self.storage_db.clone());
				let Ok(Some(access_token)) =
					storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					error!("Failed to get access token from storage");
					return Err(());
				};

				let should_notify_parentchain = pumpx_config.order_type != PumpxOrderType::Limit;

				// do cross-chain swap first, if required
				if pumpx_config.is_cross_chain() {
					amount = match self
						.execute_cross_chain_swap(
							*account_id.as_ref(),
							intent_id,
							&access_token,
							swap_order,
							pumpx_config,
						)
						.await
					{
						Ok(amount) => amount,
						Err(_) => {
							error!("Error executing cross chain swap for intent_id: {}", intent_id);
							let body = CrossFailBody {
								request_id: intent_id,
								fail_reason: "".to_string(), // TODO: `execute_cross_chain_swap` should return concrete reasons
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								error!("Failed to notify pumpx-signer");
							})?;
							return Err(());
						},
					}
				}

				// then do a single (native) chain swap
				let res = self
					.execute_single_chain_swap(
						*account_id.as_ref(),
						intent_id,
						&access_token,
						amount,
						pumpx_config,
					)
					.await?;

				// self.account_asset_lock.release(
				// 	account_id.clone(),
				// 	swap_order.from_asset.clone(),
				// 	AmountType::from_str_radix(&from_amount_string, 10).map_err(|_| {
				// 		tracing::error!("Failed to parse from_amount_string");
				// 	})?,
				// )?;

				Ok((Some(res), should_notify_parentchain))
			},
			_ => {
				error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				Err(())
			},
		}
	}

	async fn name(&self) -> &'static str {
		"cross-chain"
	}
}
