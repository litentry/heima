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
use crate::transaction_signer::TransactionSigner;
use async_trait::async_trait;
use executor_core::event_handler::Error::RecoverableError;
use executor_core::event_handler::{Error, EventHandler as EventHandlerTrait};
use executor_core::intent_executor::IntentExecutor;
use executor_core::key_store::KeyStore;
use executor_core::primitives::Intent;
use executor_core::storage::Storage;
use log::error;
use parentchain_api_interface::{
	omni_account::{
		calls::types::intent_executed::Result as IntentExecutionResult,
		events::{AccountStoreUpdated, IntentRequested},
		storage::types::account_store::AccountStore,
	},
	runtime_types::core_primitives::intent::Intent as RuntimeIntent,
	tx as parentchain_tx,
};
use parentchain_primitives::BlockEvent;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::marker::PhantomData;
use std::sync::Arc;
use subxt::ext::scale_decode;
use subxt::ext::scale_decode::DecodeAsFields;
use subxt::{events::StaticEvent, Config, Metadata};
use subxt_core::config::DefaultExtrinsicParams;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_signer::sr25519::SecretKeyBytes;

pub struct EventHandler<
	ChainConfig: Config,
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
	EthereumIntentExecutorT: IntentExecutor,
	SolanaIntentExecutorT: IntentExecutor,
	KeyStoreT: KeyStore<SecretKeyBytes>,
	RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header>,
	RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient>,
	AccountStoreStorage: Storage<ChainConfig::AccountId, AccountStore>,
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
	account_store_storage: Arc<AccountStoreStorage>,
	phantom_data: PhantomData<(MetadataT, RpcClient)>,
}

impl<
		ChainConfig: Config,
		MetadataT,
		MetadataProviderT: MetadataProvider<MetadataT>,
		EthereumIntentExecutorT: IntentExecutor,
		SolanaIntentExecutorT: IntentExecutor,
		KeyStoreT: KeyStore<SecretKeyBytes>,
		RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header>,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient>,
		AccountStoreStorage: Storage<ChainConfig::AccountId, AccountStore>,
	>
	EventHandler<
		ChainConfig,
		MetadataT,
		MetadataProviderT,
		EthereumIntentExecutorT,
		SolanaIntentExecutorT,
		KeyStoreT,
		RpcClient,
		RpcClientFactory,
		AccountStoreStorage,
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
		account_store_storage: Arc<AccountStoreStorage>,
	) -> Self {
		Self {
			metadata_provider,
			ethereum_intent_executor,
			solana_intent_executor,
			rpc_client_factory,
			transaction_signer,
			account_store_storage,
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
		RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header> + Send + Sync,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient>
			+ Send
			+ Sync,
		AccountStoreStorage: Storage<ChainConfig::AccountId, AccountStore> + Send + Sync,
	> EventHandlerTrait<BlockEvent>
	for EventHandler<
		ChainConfig,
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
		EthereumIntentExecutorT,
		SolanaIntentExecutorT,
		KeyStoreT,
		RpcClient,
		RpcClientFactory,
		AccountStoreStorage,
	>
{
	async fn handle(&self, event: BlockEvent) -> Result<(), Error> {
		log::debug!("Got event: {:?}, variant name: {}", event.id, event.variant_name);

		if event.pallet_name != "OmniAccount" {
			// we are not interested in this event
			log::debug!("Not interested in {} events", event.pallet_name);
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

		let fields = variant
			.fields
			.iter()
			.map(|f| scale_decode::Field::new(f.ty.id, f.name.as_deref()));

		match variant.name.as_str() {
			IntentRequested::EVENT => {
				let intent_requested: IntentRequested = IntentRequested::decode_as_fields(
					&mut event.field_bytes.as_slice(),
					&mut fields.clone(),
					metadata.types(),
				)
				.map_err(|_| {
					log::error!("Could not decode event {:?}", event.id);
					Error::NonRecoverableError
				})?;

				handle_intent_requested_event(
					&self.ethereum_intent_executor,
					&self.solana_intent_executor,
					&self.rpc_client_factory,
					self.transaction_signer.clone(),
					intent_requested,
				)
				.await?;
			},
			AccountStoreUpdated::EVENT => {
				let account_store_updated: AccountStoreUpdated =
					AccountStoreUpdated::decode_as_fields(
						&mut event.field_bytes.as_slice(),
						&mut fields.clone(),
						metadata.types(),
					)
					.map_err(|_| {
						log::error!("Could not decode event {:?}", event.id);
						Error::NonRecoverableError
					})?;

				self.account_store_storage
					.insert(account_store_updated.who, account_store_updated.account_store)
					.map_err(|_| {
						log::error!("Could not insert account store into storage");
						Error::NonRecoverableError
					})?;
			},
			_ => {
				log::debug!("Not interested in {} events", event.variant_name);
			},
		}

		Ok(())
	}
}

async fn handle_intent_requested_event<
	ChainConfig: Config<
		ExtrinsicParams = DefaultExtrinsicParams<ChainConfig>,
		AccountId = AccountId32,
		Address = MultiAddress<AccountId32, u32>,
		Signature = MultiSignature,
	>,
	EthereumIntentExecutorT: IntentExecutor + Send + Sync,
	SolanaIntentExecutorT: IntentExecutor + Send + Sync,
	KeyStoreT: KeyStore<SecretKeyBytes> + Send + Sync,
	RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header> + Send + Sync,
	RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient> + Send + Sync,
>(
	ethereum_intent_executor: &EthereumIntentExecutorT,
	solana_intent_executor: &SolanaIntentExecutorT,
	rpc_client_factory: &RpcClientFactory,
	transaction_signer: Arc<
		TransactionSigner<
			KeyStoreT,
			RpcClient,
			RpcClientFactory,
			ChainConfig,
			Metadata,
			SubxtMetadataProvider<ChainConfig>,
		>,
	>,
	event: IntentRequested,
) -> Result<(), Error> {
	let maybe_intent = match event.intent {
		RuntimeIntent::CallEthereum(ref call_ethereum) => Some(Intent::CallEthereum(
			call_ethereum.address.to_fixed_bytes(),
			call_ethereum.input.0.clone(),
		)),
		RuntimeIntent::TransferEthereum(ref transfer) => {
			Some(Intent::TransferEthereum(transfer.to.to_fixed_bytes(), transfer.value))
		},
		RuntimeIntent::TransferSolana(ref transfer) => {
			Some(Intent::TransferSolana(transfer.to, transfer.value))
		},
		RuntimeIntent::SystemRemark(_) => None,
		RuntimeIntent::TransferNative(_) => None,
	};

	let mut execution_result = IntentExecutionResult::Success;
	if let Some(intent) = maybe_intent {
		// to explicitly handle all intent variants
		match intent {
			Intent::CallEthereum(_, _) | Intent::TransferEthereum(_, _) => {
				if let Err(e) = ethereum_intent_executor.execute(intent).await {
					log::error!("Error executing intent: {:?}", e);
					execution_result = IntentExecutionResult::Failure;
				}
			},
			Intent::TransferSolana(_, _) => {
				if let Err(e) = solana_intent_executor.execute(intent).await {
					log::error!("Error executing intent: {:?}", e);
					execution_result = IntentExecutionResult::Failure;
				}
			},
		}

		log::debug!("Intent executed, publishing result");

		let call = parentchain_tx().omni_account().intent_executed(
			event.who,
			event.intent,
			execution_result,
		);

		let mut client = rpc_client_factory.new_client().await.map_err(|e| {
			error!("Could not create RPC client: {:?}", e);
			RecoverableError
		})?;

		// todo: the whole signing part should be encapsulated in separate component like `TransactionSigner`
		//we need to report back to parachain intent result
		let signed_call = transaction_signer.sign(call).await;
		client.submit_tx(&signed_call).await.map_err(|e| {
			error!("Error while submitting tx: {:?}", e);
			RecoverableError
		})?;
		log::debug!("Result published");
	}

	Ok(())
}
