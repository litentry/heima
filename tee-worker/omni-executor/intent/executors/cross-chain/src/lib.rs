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
use alloy::primitives::{Address, PrimitiveSignature, U256};
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
use log::error;
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
// use log::error;
use accounting_contract_client::AccountingContractApi;
use alloy::primitives::private::alloy_rlp::Decodable;
use binance_api::spot_trading_api::SpotTradingApi;
use binance_api::wallet_api::WalletApi;
use binance_api::BinanceApi;
use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use intent_asset_lock::precise::PreciseAssetsLock;
use intent_asset_lock::AccountAssetLocks;
use intent_asset_lock::AmountType;
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
use pumpx::methods::create_market_order_tx::CreateMarketOrderTxBody;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use pumpx::methods::cross_fail::CrossFailBody;
use pumpx::methods::send_order_tx::SendOrderTxBody;
use pumpx::pubkey_to_address;
use pumpx::signer_client::ChainType;
use pumpx::signer_client::SignerClient;
use pumpx::PumpxApi;
use std::collections::HashMap;
use std::sync::Arc;

use log::debug;

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
	account_asset_lock: Arc<AccountAssetLocks<PreciseAssetsLock>>,
	rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pumpx_api: Arc<Box<dyn PumpxApi>>,
	storage_db: Arc<StorageDB>,
	binance_api: Arc<BinanceClient>,
	solana_client: Arc<SolanaClient>,
	accounting_contract_client: Arc<Box<dyn AccountingContractApi>>,
	instant_payout_threshold: Decimal,
}

impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait>
	CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		account_asset_lock: Arc<AccountAssetLocks<PreciseAssetsLock>>,
		rpc_endpoint_registry: RpcEndpointRegistry,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		storage_db: Arc<StorageDB>,
		binance_api: Arc<BinanceClient>,
		solana_client: Arc<SolanaClient>,
		accounting_contract_client: Arc<Box<dyn AccountingContractApi>>,
		instant_payout_threshold: Decimal,
	) -> Result<Self, ()> {
		Ok(Self {
			account_asset_lock,
			rpc_endpoint_registry,
			pumpx_signer_client,
			pumpx_api,
			storage_db,
			binance_api,
			solana_client,
			accounting_contract_client,
			instant_payout_threshold,
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

				// TODO: update this when we have more providers
				let SingleChainSwapProvider::Pumpx(pumpx_config) = scsp;

				let mut amount = std::str::from_utf8(&pumpx_config.from_amount)
					.map_err(|_| {
						log::error!("Failed to parse from_amount");
					})
					.map(|v| v.to_string())?;

				let Some(from_chain_type) =
					ChainType::from_pumpx_chain_id(pumpx_config.from_chain_id)
				else {
					log::error!("Unsupported from_chain_id: {}", pumpx_config.from_chain_id);
					return Err(());
				};
				let from_wallet: Vec<u8> = self
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

				let mut from_address = pubkey_to_address(from_chain_type, &from_wallet)?;

				let storage = PumpxJwtStorage::new(self.storage_db.clone());
				let Ok(Some(access_token)) =
					storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get access token from storage");
					return Err(());
				};

				let mut instant_flow_details: Option<InstantFlowDetails> = None;

				let should_notify_parentchain = pumpx_config.order_type != PumpxOrderType::Limit;

				// do cross-chain swap first, if required
				if pumpx_config.is_cross_chain() {
					(amount, from_address, instant_flow_details) = match self
						.execute_cross_chain_swap(
							account_id,
							*account_id.as_ref(),
							intent_id,
							&access_token,
							swap_order,
							from_address,
							from_wallet,
							pumpx_config,
						)
						.await
					{
						Ok((amount, address, instant_flow_details)) => {
							(amount, address, instant_flow_details)
						},
						Err(_) => {
							log::error!(
								"Error executing cross chain swap for intent_id: {}",
								intent_id
							);
							let body = CrossFailBody {
								request_id: intent_id,
								fail_reason: "".to_string(), // TODO: `execute_cross_chain_swap` should return concrete reasons
							};
							self.pumpx_api.cross_fail(&access_token, body).await.map_err(|_| {
								log::error!("Failed to notify pumpx-signer");
							})?;
							return Err(());
						},
					};
				}

				// then do a single (native) chain swap
				let res = self
					.execute_single_chain_swap(
						*account_id.as_ref(),
						intent_id,
						&access_token,
						amount,
						pumpx_config,
						from_address,
					)
					.await?;

				// depost here and unlock assets
				if let Some(details) = instant_flow_details {
					self.do_binance_deposit(
						details.omni_account,
						details.from_asset,
						details.from_amount,
						details.from_address,
						details.wallet_index,
						false,
					)
					.await?;

					self.account_asset_lock.release(
						account_id.clone(),
						swap_order.from_asset.clone(),
						details.locked_amount,
					)?;
				}

				Ok((Some(res), should_notify_parentchain))
			},
			_ => {
				log::error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				Err(())
			},
		}
	}

	async fn name(&self) -> &'static str {
		"cross-chain"
	}
}

pub(crate) struct InstantFlowDetails {
	pub omni_account: [u8; 32],
	pub from_asset: ChainAsset,
	pub from_amount: Decimal,
	pub from_address: String,
	pub wallet_index: u32,
	pub locked_amount: AmountType,
}

impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait>
	CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
	async fn do_binance_deposit(
		&self,
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: Decimal,
		from_address: String,
		wallet_index: u32,
		should_wait_for_deposit_confirm: bool,
	) -> Result<(), ()> {
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

		debug!("Binance deposit address: {}, from_amount: {}", deposit_address, from_amount);
		let amount_to_transfer_decimal =
			Self::calculate_amount_decimal(from_amount, &binance_coin_name)?;
		let Some(amount_to_transfer) = amount_to_transfer_decimal.to_u64() else {
			error!("Failed to convert amount to transfer to u64");
			return Err(());
		};

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
		}

		Ok(())
	}

	fn calculate_amount_decimal(
		from_amount: Decimal,
		binance_coin_name: &str,
	) -> Result<Decimal, ()> {
		let asset_decimal_multiplier = match binance_coin_name {
			"USDC" => Decimal::from(1_000_000),    // 10^6
			"USDT" => Decimal::from(1_000_000),    // 10^6
			"SOL" => Decimal::from(1_000_000_000), // 10^9  TODO: double check this
			_ => {
				error!("Unsupported asset: {:?}", binance_coin_name);
				return Err(());
			},
		};
		Ok(from_amount * asset_decimal_multiplier)
	}
}
