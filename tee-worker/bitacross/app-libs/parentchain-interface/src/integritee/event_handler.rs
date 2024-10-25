/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

pub use ita_sgx_runtime::{Balance, Index};

use bc_enclave_registry::{EnclaveRegistry, EnclaveRegistryUpdater};
use bc_relayer_registry::{RelayerRegistry, RelayerRegistryUpdater};
use bc_signer_registry::{SignerRegistry, SignerRegistryUpdater};
use codec::Encode;
use core::str::from_utf8;
use ita_stf::TrustedCallSigned;
use itc_parentchain_indirect_calls_executor::error::Error;
use itp_stf_primitives::traits::IndirectExecutor;
use itp_types::{
	parentchain::{FilterEvents, HandleParentchainEvents, ParentchainEventProcessingError},
	WorkerType,
};
use litentry_primitives::{Address32, Identity};
use log::*;
use sp_core::{blake2_256, H256};
use sp_runtime::traits::{Block as ParentchainBlock, Header as ParentchainHeader};
use sp_std::vec::Vec;
use std::string::ToString;

pub struct ParentchainEventHandler {}

impl ParentchainEventHandler {
	fn add_relayer(relayer_registry: &RelayerRegistry, account: Identity) -> Result<(), Error> {
		info!("Adding Relayer Account to Registry: {:?}", account);
		relayer_registry.update(account).map_err(|e| {
			error!("Error adding relayer: {:?}", e);
			Error::Other("Error adding relayer".into())
		})?;

		Ok(())
	}

	fn remove_relayer(relayer_registry: &RelayerRegistry, account: Identity) -> Result<(), Error> {
		info!("Remove Relayer Account from Registry: {:?}", account);
		relayer_registry.remove(account).map_err(|e| {
			error!("Error removing relayer: {:?}", e);
			Error::Other("Error removing relayer".into())
		})?;

		Ok(())
	}

	fn add_enclave(
		enclave_registry: &EnclaveRegistry,
		account_id: Address32,
		url: Vec<u8>,
		worker_type: WorkerType,
	) -> Result<(), Error> {
		info!("Adding Enclave Account to Registry: {:?}", account_id);
		if worker_type != WorkerType::BitAcross {
			warn!("Ignore AddEnclave due to wrong worker_type");
			return Ok(())
		}

		let url = from_utf8(&url)
			.map_err(|_| Error::Other("Invalid enclave URL".into()))?
			.to_string();
		enclave_registry.update(account_id, url).map_err(|e| {
			error!("Error adding enclave: {:?}", e);
			Error::Other("Error adding enclave".into())
		})?;

		Ok(())
	}

	fn remove_enclave(
		enclave_registry: &EnclaveRegistry,
		account_id: Address32,
	) -> Result<(), Error> {
		info!("Remove Enclave Account from Registry: {:?}", account_id);
		enclave_registry.remove(account_id).map_err(|e| {
			error!("Error removing enclave: {:?}", e);
			Error::Other("Error removing enclave".into())
		})?;

		Ok(())
	}

	fn save_signer(
		signer_registry: &SignerRegistry,
		account_id: Address32,
		pub_key: [u8; 33],
	) -> Result<(), Error> {
		info!("Saving Signer Account to Registry: {:?}", account_id);
		signer_registry.update(account_id, pub_key).map_err(|e| {
			error!("Error saving signer: {:?}", e);
			Error::Other("Error saving signer".into())
		})?;

		Ok(())
	}
}

impl<Executor>
	HandleParentchainEvents<
		Executor,
		TrustedCallSigned,
		Error,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	> for ParentchainEventHandler
where
	Executor: IndirectExecutor<
		TrustedCallSigned,
		Error,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	>,
{
	type Output = Vec<H256>;

	fn handle_events<Block>(
		&self,
		executor: &Executor,
		events: impl FilterEvents,
		_block_number: <<Block as ParentchainBlock>::Header as ParentchainHeader>::Number,
	) -> Result<Vec<H256>, Error>
	where
		Block: ParentchainBlock,
	{
		let mut handled_events: Vec<H256> = Vec::new();

		if let Ok(events) = events.get_relayer_added_events() {
			debug!("Handling RelayerAdded events");
			let relayer_registry = executor.get_relayer_registry_updater();
			events
				.iter()
				.try_for_each(|event| {
					debug!("found RelayerAdded event: {:?}", event);
					let result = Self::add_relayer(relayer_registry, event.who.clone());
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::RelayerAddFailure)?;
		}

		if let Ok(events) = events.get_relayers_removed_events() {
			debug!("Handling RelayerRemoved events");
			let relayer_registry = executor.get_relayer_registry_updater();
			events
				.iter()
				.try_for_each(|event| {
					debug!("found RelayerRemoved event: {:?}", event);
					let result = Self::remove_relayer(relayer_registry, event.who.clone());
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::RelayerRemoveFailure)?;
		}

		if let Ok(events) = events.get_enclave_added_events() {
			debug!("Handling EnclaveAdded events");
			let enclave_registry = executor.get_enclave_registry_updater();
			events
				.iter()
				.try_for_each(|event| {
					debug!("found EnclaveAdded event: {:?}", event);
					let result = Self::add_enclave(
						enclave_registry,
						event.who,
						event.url.clone(),
						event.worker_type,
					);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::EnclaveAddFailure)?;
		}

		if let Ok(events) = events.get_enclave_removed_events() {
			debug!("Handling EnclaveRemoved events");
			let enclave_registry = executor.get_enclave_registry_updater();
			events
				.iter()
				.try_for_each(|event| {
					debug!("found EnclaveRemoved event: {:?}", event);
					let result = Self::remove_enclave(enclave_registry, event.who);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::EnclaveRemoveFailure)?;
		}

		if let Ok(events) = events.get_btc_wallet_generated_events() {
			debug!("Handling BtcWalletGenerated events");
			let signer_registry = executor.get_signer_registry_updater();
			events
				.iter()
				.try_for_each(|event| {
					debug!("found BtcWalletGenerated event: {:?}", event);
					let result = Self::save_signer(
						signer_registry,
						event.account_id.clone().into(),
						event.pub_key,
					);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::BtcWalletGeneratedFailure)?;
		}

		Ok(handled_events)
	}
}

fn hash_of<T: Encode>(ev: &T) -> H256 {
	blake2_256(&ev.encode()).into()
}
