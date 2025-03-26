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
use executor_primitives::Intent;
use std::marker::PhantomData;
use std::sync::Arc;

use executor_primitives::AccountId;
use parentchain_api_interface::runtime_types::pallet_omni_account::pallet::CrossChainSwapProcessingEvent;
use parentchain_api_interface::runtime_types::pallet_omni_account::pallet::IntentEvent;
use parentchain_api_interface::runtime_types::pallet_omni_account::pallet::IntentProcessingEvent;
use parentchain_rpc_client::metadata::Metadata;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::CustomConfig;
use parentchain_rpc_client::SubstrateRpcClient;
use parentchain_rpc_client::SubstrateRpcClientFactory;
use parentchain_rpc_client::SubxtClient;
use parentchain_rpc_client::SubxtClientFactory;
use parentchain_rpc_client::ToSubxtType;
use parentchain_rpc_client::XtStatus;
use parentchain_signer::key_store::SubstrateKeyStore;
use parentchain_signer::TransactionSigner;

pub type ParentchainTxSigner = TransactionSigner<
	SubstrateKeyStore,
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
	) -> Result<Self, ()> {
		Ok(Self { parentchain_rpc_client_factory, transaction_signer, phantom: PhantomData })
	}
}

#[async_trait]
impl<
		Header: Send + Sync,
		RpcClient: SubstrateRpcClient<Header> + Send + Sync,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync,
	> IntentExecutor for CrossChainIntentExecutor<Header, RpcClient, RpcClientFactory>
{
	async fn execute(&self, account_id: &AccountId, intent: Intent) -> Result<(), ()> {
		match intent {
			Intent::CrossChainSwap(ref _swap_order) => {
				let Ok(mut rpc_client) = self.parentchain_rpc_client_factory.new_client().await
				else {
					log::error!("Failed to create rpc client");
					return Err(());
				};

				// 1. Check if user has enough balance on the source chain
				// 2. Lock the balance on the source chain

				let sanity_check_passed = true;

				if sanity_check_passed {
					let signer_account_id = self.transaction_signer.get_signer_account_id();
					let mut nonce = match rpc_client.get_account_nonce(&signer_account_id).await {
						Ok(n) => n,
						Err(e) => {
							log::error!("Failed to get account nonce: {:?}", e);
							return Err(());
						},
					};

					// intent requested is submited before so nonce needs to be updated
					nonce += 1;

					let intent_accepted_event_emit_call =
						parentchain_api_interface::tx().omni_account().emit_intent_event(
							account_id.to_subxt_type(),
							intent.to_subxt_type(),
							IntentEvent::Processing(IntentProcessingEvent::CrossChainSwap(
								CrossChainSwapProcessingEvent::Accepted,
							)),
						);

					let tx = self
						.transaction_signer
						.sign(intent_accepted_event_emit_call, Some(nonce))
						.await;

					match rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await {
						Ok(report) => report,
						Err(e) => {
							log::error!("Failed to submit and watch tx: {:?}", e);
							return Err(());
						},
					};
				}

				// TODO:
				// 3. Swap assets:
				//    - Call accounting contract (e.g Swap SOL to TRUMP)
				//    - Call Binance convert via binance account (e.g Swap USDC to SOL)
				// 4. Send locked balance to binance account (refill)

				todo!("CrossChainSwap is not implemented yet");
			},
			_ => {
				log::error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}
	}
}
