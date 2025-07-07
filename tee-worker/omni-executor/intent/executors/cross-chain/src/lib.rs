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

mod types;

use ::pumpx::methods::common::{GasType, SwapType};
use ::pumpx::methods::create_limit_order::CreateLimitOrderBody;
use ::pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use ::pumpx::methods::cross_fail::CrossFailBody;
use ::pumpx::methods::send_order_tx::SendOrderTxBody;
use ::pumpx::signer_client::PumpxChainId;
use aa_contracts_client::calculate_omni_account_address;
use accounting_contract_client::{
	solana::AccountingContractApi as SolanaAccountingContractApi,
	AccountingContractApi as EvmAccountingContractApi,
};
use alloy::consensus::{SignableTransaction, TxLegacy};
use alloy::network::TxSigner as AlloyTxSigner;
use alloy::primitives::private::alloy_rlp::Decodable;
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{Address, Signature, U256};
use async_trait::async_trait;
use binance_api::spot_trading_api::types::{
	CreateOrderParams as BinanceCreateOrderParams, OrderSide as BinanceOrderSide,
	OrderStatus as BinanceOrderStatus, OrderType as BinanceOrderType,
};
use binance_api::spot_trading_api::SpotTradingApi;
use binance_api::wallet_api::WalletApi;
use binance_api::BinanceApi;
use ethereum_rpc::{
	client::EthereumClient as EthereumClientTrait, signer::RemoteSigner as RemoteEvmSigner,
};
use executor_core::intent_executor::{IntentExecutionResult, IntentExecutor};
use executor_primitives::AccountId;
use executor_primitives::ChainAsset;
use executor_primitives::Intent;
use executor_primitives::IntentId;
use executor_primitives::PumpxOrderType;
use executor_primitives::SingleChainSwapProvider;
use executor_storage::StorageDB;
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::constants::{AUTH_TOKEN_ACCESS_TYPE, CLIENT_ID_HEIMA};
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
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use signer_client::{ChainType, SignerClient};
use solana::{signer::RemoteSigner as RemoteSolanaSigner, SolanaClient as SolanaClientTrait};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
	runtime::Handle,
	time::{sleep, Duration},
};
pub use types::*;

use tracing::{debug, error};

mod omni;
mod pumpx;
mod utils;
use utils::{
	determine_trade_symbol_and_order_side, estimate_payout_amount, get_binance_deposit_info,
	str_to_u256,
};
// use intent_asset_lock::always_unlocked::AlwaysUnlockedAssetsLock;
// use intent_asset_lock::AccountAssetLocks;

pub type ParentchainTxSigner = TxSigner<
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

// TODO: should we rename this to something like MultiChainIntentExecutor?
pub struct CrossChainIntentExecutor<
	BinanceClient: BinanceApi,
	EthereumClient: EthereumClientTrait,
	SolanaClient: SolanaClientTrait,
> {
	account_asset_lock: Arc<AccountAssetLocks<PreciseAssetsLock>>,
	rpc_endpoint_registry: RpcEndpointRegistry,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pumpx_api: Arc<Box<dyn ::pumpx::PumpxApi>>,
	storage_db: Arc<StorageDB>,
	binance_api: Arc<BinanceClient>,
	bsc_client: Arc<EthereumClient>,
	solana_client: Arc<SolanaClient>,
	evm_accounting_contract_client: Arc<Box<dyn EvmAccountingContractApi>>,
	solana_accounting_contract_client: Arc<Box<dyn SolanaAccountingContractApi>>,
	instant_payout_threshold: Decimal,
	// AA contracts configuration
	omni_account_factory_address: Address,
	omni_account_implementation_address: Address,
}

impl<
		BinanceClient: BinanceApi,
		EthereumClient: EthereumClientTrait,
		SolanaClient: SolanaClientTrait,
	> CrossChainIntentExecutor<BinanceClient, EthereumClient, SolanaClient>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		account_asset_lock: Arc<AccountAssetLocks<PreciseAssetsLock>>,
		rpc_endpoint_registry: RpcEndpointRegistry,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		pumpx_api: Arc<Box<dyn ::pumpx::PumpxApi>>,
		storage_db: Arc<StorageDB>,
		binance_api: Arc<BinanceClient>,
		bsc_client: Arc<EthereumClient>,
		solana_client: Arc<SolanaClient>,
		evm_accounting_contract_client: Arc<Box<dyn EvmAccountingContractApi>>,
		solana_accounting_contract_client: Arc<Box<dyn SolanaAccountingContractApi>>,
		instant_payout_threshold: Decimal,
		omni_account_factory_address: Address,
		omni_account_implementation_address: Address,
	) -> Result<Self, ()> {
		Ok(Self {
			account_asset_lock,
			rpc_endpoint_registry,
			pumpx_signer_client,
			pumpx_api,
			storage_db,
			binance_api,
			bsc_client,
			solana_client,
			evm_accounting_contract_client,
			solana_accounting_contract_client,
			instant_payout_threshold,
			omni_account_factory_address,
			omni_account_implementation_address,
		})
	}

	/// Generate omni account address using AA contracts
	fn generate_omni_account_address(
		&self,
		omni_account: [u8; 32],
		client_id: &[u8],
		root_address: Address,
	) -> Address {
		calculate_omni_account_address(
			self.omni_account_factory_address,
			self.omni_account_implementation_address,
			omni_account.into(),
			client_id,
			root_address,
		)
	}
}

#[async_trait]
impl<
		BinanceClient: BinanceApi + 'static,
		EthereumClient: EthereumClientTrait + 'static,
		SolanaClient: SolanaClientTrait + 'static,
	> IntentExecutor for CrossChainIntentExecutor<BinanceClient, EthereumClient, SolanaClient>
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

				match scsp.to_owned().try_into()? {
					SingleChainSwapProvider::Pumpx(pumpx_config) => {
						let mut amount = pumpx_config.from_amount.clone();

						let Some(from_chain_type) =
							ChainType::from_pumpx_chain_id(pumpx_config.from_chain_id)
						else {
							error!("Unsupported from_chain_id: {}", pumpx_config.from_chain_id);
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
								error!("Could not get from_wallet from pumpx-signer: {:?}", e)
							})?;

						let mut from_address =
							::pumpx::pubkey_to_address(from_chain_type, &from_wallet)?;

						let Some(to_chain_type) =
							ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id)
						else {
							error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
							return Err(());
						};

						let to_wallet = self
							.pumpx_signer_client
							.request_wallet(
								to_chain_type,
								pumpx_config.wallet_index,
								*account_id.as_ref(),
							)
							.await
							.map_err(|e| {
								error!("Could not get to_wallet from pumpx-signer: {:?}", e)
							})?;

						let to_address = ::pumpx::pubkey_to_address(to_chain_type, &to_wallet)?;

						let storage = HeimaJwtStorage::new(self.storage_db.clone());
						let Ok(Some(access_token)) =
							storage.get(&(account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
						else {
							error!("Failed to get access token from storage");
							return Err(());
						};

						let mut instant_flow_details: Option<InstantFlowDetails> = None;

						let should_notify_parentchain =
							pumpx_config.order_type != PumpxOrderType::Limit;

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
									&pumpx_config,
									to_address.clone(),
								)
								.await
							{
								Ok((amount, address, instant_flow_details)) => {
									(amount, address, instant_flow_details)
								},
								Err(_) => {
									error!(
										"Error executing cross chain swap for intent_id: {}",
										intent_id
									);
									let body = CrossFailBody {
										request_id: intent_id,
										fail_reason: "".to_string(), // TODO: `execute_cross_chain_swap` should return concrete reasons
									};
									self.pumpx_api.cross_fail(&access_token, body).await.map_err(
										|_| {
											error!("Failed to notify pumpx-signer");
										},
									)?;
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
								&pumpx_config,
								from_address,
								to_address,
							)
							.await?;

						// deposit here and unlock assets
						if let Some(details) = instant_flow_details {
							let binance_api = self.binance_api.clone();
							let pumpx_signer = self.pumpx_signer_client.clone();
							let bsc_client = self.bsc_client.clone();
							let solana_client = self.solana_client.clone();
							let account_asset_lock = self.account_asset_lock.clone();
							let from_asset = swap_order.from_asset.clone();
							let account_id = account_id.clone();
							tokio::spawn(async move {
								if let Err(e) = Self::do_binance_deposit(
									details.omni_account,
									details.from_asset,
									details.from_amount.to_string(),
									details.from_address,
									details.wallet_index,
									false,
									binance_api,
									pumpx_signer,
									bsc_client,
									solana_client,
								)
								.await
								{
									error!("Could not deposit to binance: {:?}", e);
								} else if let Err(e) = account_asset_lock.release(
									account_id,
									from_asset,
									details.locked_amount,
								) {
									error!("Could not release asset lock: {:?}", e);
								}
							});
						}

						Ok((Some(res), should_notify_parentchain))
					},
					SingleChainSwapProvider::Omni => {
						debug!("Processing Omni single chain swap provider");

						// For omni, we generate addresses using AA contracts
						let root_address =
							self.evm_accounting_contract_client.get_signer_address().await;
						let omni_account: [u8; 32] = *account_id.as_ref();

						// Generate from and to addresses using AA contracts
						let from_address_addr = self.generate_omni_account_address(
							omni_account,
							CLIENT_ID_HEIMA.as_bytes(),
							root_address,
						);
						let from_address = from_address_addr.to_string();
						let to_address = from_address.clone(); // For single chain swaps, from and to are the same account
						let from_wallet = omni_account.to_vec();

						// Extract amount from swap order
						let amount =
							String::from_utf8(swap_order.from_amount.to_vec()).map_err(|_| {
								error!("Failed to parse from_amount from swap_order");
							})?;

						// Check if this is a cross-chain swap for omni
						let is_cross_chain = match (&swap_order.from_asset, &swap_order.to_asset) {
							(ChainAsset::Solana(_), ChainAsset::Ethereum(56, _)) => true, // SOL to BSC
							(ChainAsset::Ethereum(56, _), ChainAsset::Solana(_)) => true, // BSC to SOL
							_ => false,
						};

						let mut instant_flow_details: Option<InstantFlowDetails> = None;
						let mut final_amount = amount.clone();
						let mut final_from_address = from_address.clone();

						if is_cross_chain {
							debug!("Omni cross-chain swap detected");
							match self
								.execute_omni_cross_chain_swap(
									account_id,
									*account_id.as_ref(),
									intent_id,
									swap_order,
									from_address,
									from_wallet,
									to_address.clone(),
									amount,
								)
								.await
							{
								Ok((payout_amount, payout_address, instant_details)) => {
									final_amount = payout_amount;
									final_from_address = payout_address;
									instant_flow_details = instant_details;
								},
								Err(_) => {
									error!(
										"Omni cross-chain swap failed for intent_id: {}",
										intent_id
									);
									return Err(());
								},
							}
						}

						// For single chain swaps, call the omni single chain method
						let res = self
							.execute_omni_single_chain_swap(
								*account_id.as_ref(),
								intent_id,
								final_amount,
								final_from_address,
								to_address,
								swap_order.from_asset.clone(),
								swap_order.to_asset.clone(),
							)
							.await;

						// Handle instant flow for omni (similar to pumpx but without pumpx-specific logic)
						if let Some(details) = instant_flow_details {
							let binance_api = self.binance_api.clone();
							let bsc_client = self.bsc_client.clone();
							let solana_client = self.solana_client.clone();
							let account_asset_lock = self.account_asset_lock.clone();
							let from_asset = swap_order.from_asset.clone();
							let account_id_clone = account_id.clone();

							tokio::spawn(async move {
								// For omni, we use the same binance deposit logic but without pumpx-signer
								if let Err(e) = Self::do_omni_binance_deposit(
									details.omni_account,
									details.from_asset,
									details.from_amount.to_string(),
									details.from_address,
									false,
									binance_api,
									bsc_client,
									solana_client,
								)
								.await
								{
									error!("Could not deposit to binance for omni: {:?}", e);
								} else if let Err(e) = account_asset_lock.release(
									account_id_clone,
									from_asset,
									details.locked_amount,
								) {
									error!("Could not release asset lock for omni: {:?}", e);
								}
							});
						}

						match res {
							Ok(result) => Ok((Some(result), false)), // false = don't notify parentchain for omni
							Err(_) => {
								error!("Omni single chain swap failed");
								Err(())
							},
						}
					},
				}
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

pub struct InstantFlowDetails {
	pub omni_account: [u8; 32],
	pub from_asset: ChainAsset,
	pub from_amount: Decimal,
	pub from_address: String,
	pub wallet_index: u32,
	pub locked_amount: AmountType,
}

impl<
		BinanceClient: BinanceApi,
		EthereumClient: EthereumClientTrait,
		SolanaClient: SolanaClientTrait,
	> CrossChainIntentExecutor<BinanceClient, EthereumClient, SolanaClient>
{
	async fn do_omni_binance_deposit(
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		should_wait_for_deposit_confirm: bool,
		binance_api: Arc<BinanceClient>,
		_bsc_client: Arc<EthereumClient>,
		_solana_client: Arc<SolanaClient>,
	) -> Result<(String, U256), ()> {
		let (_deposit_address, binance_asset, from_amount_decimal, _amount_to_transfer_decimal) =
			get_binance_deposit_info(binance_api.clone(), &from_asset, &from_amount).await?;

		let binance_network = binance_asset.network;
		let binance_coin = binance_asset.coin;
		let _binance_address = binance_asset.address;

		let (trade_symbol, order_side) = determine_trade_symbol_and_order_side(
			binance_network.clone(),
			binance_coin.clone(),
			BinanceNetwork::Bsc,
		)?;

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_payout_amount(
			&binance_api,
			&trade_symbol,
			binance_coin.clone(),
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Sol.decimals())?;

		// todo: adjust and use omni-account
		let tx_id = match from_asset {
			ChainAsset::Solana(_) => {
				debug!("Omni: Simulating SOL transfer to binance deposit address");
				format!(
					"omni_sol_transfer_{}",
					omni_account.iter().map(|b| format!("{:02x}", b)).collect::<String>()
				)
			},
			ChainAsset::Ethereum(_, _) => {
				debug!("Omni: Simulating BNB transfer to binance deposit address");
				format!(
					"omni_bnb_transfer_{}",
					omni_account.iter().map(|b| format!("{:02x}", b)).collect::<String>()
				)
			},
		};

		if should_wait_for_deposit_confirm {
			let result = Self::wait_for_deposit_confirm(
				from_address,
				Some(tx_id.clone()),
				binance_network.clone(),
				binance_coin,
				trade_symbol.clone(),
				order_side,
				from_amount,
				BinanceCoin::Sol,
				binance_api.clone(),
			)
			.await?;
			payout_amount = result.0;
			payout_amount_u256 = result.1;
		};
		Ok((payout_amount, payout_amount_u256))
	}

	async fn do_binance_deposit(
		omni_account: [u8; 32],
		from_asset: ChainAsset,
		from_amount: String,
		from_address: String,
		wallet_index: u32,
		should_wait_for_deposit_confirm: bool,
		binance_api: Arc<BinanceClient>,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		bsc_client: Arc<EthereumClient>,
		solana_client: Arc<SolanaClient>,
	) -> Result<(String, U256), ()> {
		let (deposit_address, binance_asset, from_amount_decimal, amount_to_transfer_decimal) =
			get_binance_deposit_info(binance_api.clone(), &from_asset, &from_amount).await?;

		let binance_network = binance_asset.network;
		let binance_coin = binance_asset.coin;
		let binance_address = binance_asset.address;

		let (trade_symbol, order_side) = determine_trade_symbol_and_order_side(
			binance_network.clone(),
			binance_coin.clone(),
			BinanceNetwork::Bsc,
		)?;

		// init `payout_amount` with estimated-amount-to-receive
		let mut payout_amount = estimate_payout_amount(
			&binance_api,
			&trade_symbol,
			binance_coin.clone(),
			from_amount_decimal,
		)
		.await?;

		let mut payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Sol.decimals())?;

		let tx_id = match from_asset {
			ChainAsset::Solana(_) => {
				let Some(amount_to_transfer) = amount_to_transfer_decimal.to_u64() else {
					error!("Failed to convert amount to transfer to u64");
					return Err(());
				};

				let remote_signer: Box<dyn solana_sdk::signer::Signer + Send + Sync> =
					Box::new(RemoteSolanaSigner::new(
						pumpx_signer_client.clone(),
						wallet_index,
						omni_account,
						Handle::current(),
					));

				// TODO: change this when adding support for more tokens/chains
				if binance_coin.clone() == BinanceCoin::Sol {
					// Native transfer
					debug!("Transfering {:?} SOL to {:?}", amount_to_transfer, deposit_address);
					solana_client
						.transfer_sol(&deposit_address, amount_to_transfer, &remote_signer)
						.await
						.map_err(|_| {
							error!("Failed to transfer SOL");
						})?
				} else {
					debug!(
						"Transfering {:?} {:?} to {:?}",
						amount_to_transfer, binance_address, deposit_address
					);
					// SPL transfer
					solana_client
						.transfer_spl(
							&deposit_address,
							amount_to_transfer,
							&binance_address,
							&remote_signer,
						)
						.await
						.map_err(|_| {
							error!("Failed to transfer SPL");
						})?
				}
			},
			ChainAsset::Ethereum(_, _) => {
				let amount_to_transfer =
					decimal_to_u256(amount_to_transfer_decimal).map_err(|err| {
						error!("Failed to convert amount_to_transfer_decimal to U256: {:?}", err);
					})?;

				let remote_signer =
					RemoteEvmSigner::new(pumpx_signer_client.clone(), wallet_index, omni_account)
						.await
						.map_err(|err| {
							error!("Failed to create RemoteEvmSigner: {}", err);
						})?;
				let remote_signer: Box<dyn AlloyTxSigner<Signature> + Send + Sync> =
					Box::new(remote_signer);

				// Native transfer
				debug!("Transfering {:?} BNB to {:?}", amount_to_transfer, deposit_address);
				bsc_client
					.transfer(&deposit_address, amount_to_transfer, remote_signer)
					.await
					.map_err(|_| {
						error!("Failed to transfer BNB");
					})?
			},
		};

		if should_wait_for_deposit_confirm {
			let result = Self::wait_for_deposit_confirm(
				from_address,
				Some(tx_id.clone()),
				binance_network.clone(),
				binance_coin,
				trade_symbol.clone(),
				order_side,
				from_amount,
				BinanceCoin::Sol,
				binance_api.clone(),
			)
			.await?;
			payout_amount = result.0;
			payout_amount_u256 = result.1;
		};
		Ok((payout_amount, payout_amount_u256))
	}

	fn calculate_amount_decimal(
		from_amount: Decimal,
		binance_coin_name: &str,
	) -> Result<Decimal, ()> {
		let asset_decimal_multiplier = match binance_coin_name {
			"USDC" => Decimal::from(1_000_000),    // 10^6
			"USDT" => Decimal::from(1_000_000),    // 10^6
			"SOL" => Decimal::from(1_000_000_000), // 10^9  TODO: double check this
			"BNB" => Decimal::from_str("1_000_000_000_000_000_000").unwrap(), // 10^18
			_ => {
				error!("Unsupported asset: {:?}", binance_coin_name);
				return Err(());
			},
		};
		Ok(from_amount * asset_decimal_multiplier)
	}
}

fn decimal_to_u256(decimal: Decimal) -> Result<U256, ParseError> {
	let decimal_str = decimal.normalize().to_string();
	U256::from_str_radix(&decimal_str, 10).inspect_err(|err| {
		error!("Failed to convert decimal {:?} to U256: {:?}", decimal_str, err);
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decimal_to_u256() {
		// with_decimal_point
		let amount = Decimal::from_str("0.005").unwrap() * Decimal::from_str("100_000").unwrap();
		assert_eq!(amount.to_string(), "500.000");
		let value = decimal_to_u256(amount).unwrap();
		assert_eq!(value, U256::from(500));

		// without_decimal_point
		let amount = Decimal::from_str("500").unwrap();
		assert_eq!(amount.to_string(), "500");
		let value = decimal_to_u256(amount).unwrap();
		assert_eq!(value, U256::from(500));
	}
}
