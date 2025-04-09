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
use executor_storage::StorageDB;
use intent_asset_lock::AmountType;
use intent_token_query::query_ethereum;
use intent_token_query::query_solana;
use intent_token_query::EthereumAddress;
use intent_token_query::SolanaPubkey;
use log::error;
use pumpx::signer_client::SignerClient;
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
			Intent::Swap(ref swap_order, ref _ccsp, ref _scsp) => {
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
