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

use crate::metadata::{MetadataProvider, SubxtMetadataProvider};
use crate::primitives::BlockEvent;
use crate::rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use crate::transaction_signer::TransactionSigner;
use async_trait::async_trait;
use executor_core::event_handler::Error::RecoverableError;
use executor_core::event_handler::{Error, EventHandler};
use executor_core::intent_executor::IntentExecutor;
use executor_core::key_store::KeyStore;
use executor_core::primitives::Intent;
use log::error;
use std::marker::PhantomData;
use std::sync::Arc;
use subxt::ext::scale_decode;
use subxt::ext::scale_decode::DecodeAsFields;
use subxt::{Config, Metadata};
use subxt_core::config::DefaultExtrinsicParams;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_signer::sr25519::SecretKeyBytes;

pub struct IntentEventHandler<
	ChainConfig: Config,
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
	EthereumIntentExecutorT: IntentExecutor,
	SolanaIntentExecutorT: IntentExecutor,
	KeyStoreT: KeyStore<SecretKeyBytes>,
	RpcClient: SubstrateRpcClient<ChainConfig::AccountId>,
	RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, RpcClient>,
> {
	metadata_provider: Arc<MetadataProviderT>,
	ethereum_intent_executor: EthereumIntentExecutorT,
	solana_intent_executor: SolanaIntentExecutorT,
	rpc_client_factory: RpcClientFactory,
	transaction_signer: Arc<
		TransactionSigner<
			KeyStoreT,
			RpcClient,
			RpcClientFactory,
			ChainConfig,
			MetadataT,
			MetadataProviderT,
		>,
	>,
	phantom_data: PhantomData<(MetadataT, RpcClient)>,
}

impl<
		ChainConfig: Config,
		MetadataT,
		MetadataProviderT: MetadataProvider<MetadataT>,
		EthereumIntentExecutorT: IntentExecutor,
		SolanaIntentExecutorT: IntentExecutor,
		KeyStoreT: KeyStore<SecretKeyBytes>,
		RpcClient: SubstrateRpcClient<ChainConfig::AccountId>,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, RpcClient>,
	>
	IntentEventHandler<
		ChainConfig,
		MetadataT,
		MetadataProviderT,
		EthereumIntentExecutorT,
		SolanaIntentExecutorT,
		KeyStoreT,
		RpcClient,
		RpcClientFactory,
	>
{
	pub fn new(
		metadata_provider: Arc<MetadataProviderT>,
		ethereum_intent_executor: EthereumIntentExecutorT,
		solana_intent_executor: SolanaIntentExecutorT,
		rpc_client_factory: RpcClientFactory,
		transaction_signer: Arc<
			TransactionSigner<
				KeyStoreT,
				RpcClient,
				RpcClientFactory,
				ChainConfig,
				MetadataT,
				MetadataProviderT,
			>,
		>,
	) -> Self {
		Self {
			metadata_provider,
			ethereum_intent_executor,
			solana_intent_executor,
			rpc_client_factory,
			transaction_signer,
			phantom_data: Default::default(),
		}
	}
}

#[async_trait]
impl<
		ChainConfig: Config<
			ExtrinsicParams = DefaultExtrinsicParams<ChainConfig>,
			AccountId = AccountId32,
			Address = MultiAddress<AccountId32, u32>,
			Signature = MultiSignature,
		>,
		EthereumIntentExecutorT: IntentExecutor + Send + Sync,
		SolanaIntentExecutorT: IntentExecutor + Send + Sync,
		KeyStoreT: KeyStore<SecretKeyBytes> + Send + Sync,
		RpcClient: SubstrateRpcClient<ChainConfig::AccountId> + Send + Sync,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, RpcClient> + Send + Sync,
	> EventHandler<BlockEvent>
	for IntentEventHandler<
		ChainConfig,
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
		EthereumIntentExecutorT,
		SolanaIntentExecutorT,
		KeyStoreT,
		RpcClient,
		RpcClientFactory,
	>
{
	async fn handle(&self, event: BlockEvent) -> Result<(), Error> {
		log::debug!("Got event: {:?}, variant name: {}", event.id, event.variant_name);

		if event.pallet_name != "OmniAccount" || event.variant_name != "IntentRequested" {
			// we are not interested in this event
			log::debug!("Not interested in this event");
			return Ok(());
		}

		log::debug!("Got IntentRequested event: {:?}", event.id);

		let metadata = self.metadata_provider.get(Some(event.id.block_num)).await;

		let pallet = metadata.pallet_by_name(&event.pallet_name).ok_or_else(move || {
			log::error!(
				"No pallet metadata found for event {} and pallet {} ",
				event.id.block_num,
				event.pallet_name
			);
			Error::NonRecoverableError
		})?;
		let variant = pallet.event_variant_by_index(event.variant_index).ok_or_else(move || {
			log::error!(
				"No event variant metadata found for event {} and variant {}",
				event.id.block_num,
				event.variant_index
			);
			Error::NonRecoverableError
		})?;

		let mut fields = variant
			.fields
			.iter()
			.map(|f| scale_decode::Field::new(f.ty.id, f.name.as_deref()));

		let decoded =
			crate::litentry_rococo::omni_account::events::IntentRequested::decode_as_fields(
				&mut event.field_bytes.clone().as_slice(),
				&mut fields.clone(),
				metadata.types(),
			)
			.map_err(|e| {
				log::error!("Could not decode event {:?}, reason: {:?}", event.id, e);
				Error::NonRecoverableError
			})?;

		let maybe_intent = match decoded.intent {
			crate::litentry_rococo::runtime_types::core_primitives::intent::Intent::CallEthereum(call_ethereum) => {
				Some(Intent::CallEthereum(call_ethereum.address.to_fixed_bytes(), call_ethereum.input.0))
			},
			crate::litentry_rococo::runtime_types::core_primitives::intent::Intent::TransferEthereum(transfer) => {
				Some(Intent::TransferEthereum(transfer.to.to_fixed_bytes(), transfer.value))
			},
            crate::litentry_rococo::runtime_types::core_primitives::intent::Intent::TransferSolana(transfer) => {
                Some(Intent::TransferSolana(transfer.to, transfer.value))
            },
			crate::litentry_rococo::runtime_types::core_primitives::intent::Intent::SystemRemark(_) => None,
			crate::litentry_rococo::runtime_types::core_primitives::intent::Intent::TransferNative(_) => None,
		};
		let mut execution_result =
			crate::litentry_rococo::omni_account::calls::types::intent_executed::Result::Success;
		if let Some(intent) = maybe_intent {
			// to explicitly handle all intent variants
			match intent {
				Intent::CallEthereum(_, _) | Intent::TransferEthereum(_, _) => {
					if let Err(e) = self.ethereum_intent_executor.execute(intent).await {
						log::error!("Error executing intent: {:?}", e);
						execution_result = crate::litentry_rococo::omni_account::calls::types::intent_executed::Result::Failure;
					}
				},
				Intent::TransferSolana(_, _) => {
					if let Err(e) = self.solana_intent_executor.execute(intent).await {
						log::error!("Error executing intent: {:?}", e);
						execution_result = crate::litentry_rococo::omni_account::calls::types::intent_executed::Result::Failure;
					}
				},
			}

			log::debug!("Intent executed, publishing result");

			// todo: the whole signing part should be encapsulated in separate component like `TransactionSigner`
			//we need to report back to parachain intent result
			let decoded =
				crate::litentry_rococo::omni_account::events::IntentRequested::decode_as_fields(
					&mut event.field_bytes.as_slice(),
					&mut fields,
					metadata.types(),
				)
				.map_err(|_| {
					log::error!("Could not decode event {:?}", event.id);
					Error::NonRecoverableError
				})?;

			let call = crate::litentry_rococo::tx().omni_account().intent_executed(
				decoded.who,
				decoded.intent,
				execution_result,
			);

			let mut client = self.rpc_client_factory.new_client().await.map_err(|e| {
				error!("Could not create RPC client: {:?}", e);
				RecoverableError
			})?;

			let signed_call = self.transaction_signer.sign(call).await;
			client.submit_tx(&signed_call).await.map_err(|e| {
				error!("Error while submitting tx: {:?}", e);
				RecoverableError
			})?;
			log::debug!("Result published");
		}
		Ok(())
	}
}
