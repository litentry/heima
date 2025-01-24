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

mod event_handler;
mod fetcher;
mod listener;
mod metadata;
mod sync_checkpoint;

use crate::event_handler::EventHandler;
use crate::fetcher::Fetcher;
use crate::listener::ParentchainListener;
use crate::metadata::SubxtMetadataProvider;
use executor_core::intent_executor::IntentExecutor;
use executor_core::key_store::KeyStore;
use executor_core::listener::Listener;
use executor_core::sync_checkpoint_repository::FileCheckpointRepository;
use log::{error, info};
use parentchain_api_interface::{
	runtime_types::core_primitives::teebag::types::DcapProvider,
	teebag::calls::types::register_enclave::{AttestationType, WorkerMode, WorkerType},
};
use parentchain_rpc_client::SubstrateRpcClient;
use parentchain_rpc_client::{CustomConfig, SubxtClient, SubxtClientFactory};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use std::sync::Arc;
use storage::{AccountStoreStorage, MemberOmniAccountStorage, StorageDB};
use subxt_core::utils::AccountId32;
use subxt_core::Metadata;
use subxt_signer::sr25519::Keypair;
use tokio::runtime::Handle;
use tokio::sync::oneshot::Receiver;

/// Creates parentchain listener
pub async fn create_listener<EthereumIntentExecutor, SolanaIntentExecutor>(
	id: &str,
	handle: Handle,
	ws_rpc_endpoint: &str,
	ethereum_intent_executor: EthereumIntentExecutor,
	solana_intent_executor: SolanaIntentExecutor,
	stop_signal: Receiver<()>,
	storage_db: Arc<StorageDB>,
) -> Result<
	ParentchainListener<
		SubxtClient<CustomConfig>,
		SubxtClientFactory<CustomConfig>,
		FileCheckpointRepository,
		CustomConfig,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		AccountStoreStorage,
		MemberOmniAccountStorage,
	>,
	(),
>
where
	EthereumIntentExecutor: IntentExecutor + Send + Sync,
	SolanaIntentExecutor: IntentExecutor + Send + Sync,
{
	let client_factory: Arc<SubxtClientFactory<CustomConfig>> =
		Arc::new(SubxtClientFactory::new(ws_rpc_endpoint));

	let fetcher = Fetcher::new(client_factory.clone());
	let last_processed_log_repository =
		FileCheckpointRepository::new("data/parentchain_last_log.bin");

	let metadata_provider =
		Arc::new(SubxtMetadataProvider::new(SubxtClientFactory::new(ws_rpc_endpoint)));
	let key_store = Arc::new(SubstrateKeyStore::new("data/parentchain_key.bin".to_string()));
	let secret_key_bytes = key_store
		.read()
		.map_err(|e| {
			error!("Could not unseal key: {:?}", e);
		})
		.unwrap();
	let signer = subxt_signer::sr25519::Keypair::from_secret_key(secret_key_bytes)
		.map_err(|e| {
			error!("Could not create secret key: {:?}", e);
		})
		.unwrap();

	info!("Substrate signer address: {}", AccountId32::from(signer.public_key()));

	let transaction_signer = Arc::new(TransactionSigner::new(
		metadata_provider.clone(),
		client_factory.clone(),
		key_store.clone(),
	));

	perform_attestation(client_factory, signer, &transaction_signer).await?;

	let account_store_storage = Arc::new(AccountStoreStorage::new(storage_db.clone()));
	let member_account_storage = Arc::new(MemberOmniAccountStorage::new(storage_db.clone()));

	let event_handler = EventHandler::new(
		metadata_provider,
		ethereum_intent_executor,
		solana_intent_executor,
		SubxtClientFactory::new(ws_rpc_endpoint),
		transaction_signer,
		account_store_storage,
		member_account_storage,
	);

	Listener::new(id, handle, fetcher, event_handler, stop_signal, last_processed_log_repository)
}

#[allow(unused_assignments, unused_mut, unused_variables, clippy::type_complexity)]
async fn perform_attestation(
	client_factory: Arc<SubxtClientFactory<CustomConfig>>,
	signer: Keypair,
	transaction_signer: &Arc<
		TransactionSigner<
			SubstrateKeyStore,
			SubxtClient<CustomConfig>,
			SubxtClientFactory<CustomConfig>,
			CustomConfig,
			Metadata,
			SubxtMetadataProvider<CustomConfig>,
		>,
	>,
) -> Result<(), ()> {
	let mut quote = vec![];
	let mut attestation_type = AttestationType::Dcap(DcapProvider::Intel);

	#[cfg(feature = "gramine-quote")]
	{
		use std::fs;
		use std::fs::File;
		use std::io::Write;
		let mut f = File::create("/dev/attestation/user_report_data").unwrap();
		let content = signer.public_key().0;
		f.write_all(&content).unwrap();

		quote = fs::read("/dev/attestation/quote").unwrap();
		info!("Attestation quote {:?}", quote);
	}
	#[cfg(not(feature = "gramine-quote"))]
	{
		attestation_type = AttestationType::Ignore;
	}

	let registration_call = parentchain_api_interface::tx().teebag().register_enclave(
		WorkerType::OmniExecutor,
		WorkerMode::OffChainWorker,
		quote,
		vec![],
		None,
		None,
		attestation_type,
	);

	let mut client = client_factory.new_client_until_connected().await;
	let signed_call = transaction_signer.sign(registration_call).await;
	client.submit_tx(&signed_call).await.map_err(|e| {
		error!("Error while submitting tx: {:?}", e);
	})?;
	Ok(())
}
