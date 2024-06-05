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

use codec::Encode;

use bc_enclave_registry::{EnclaveRegistryUpdater, GLOBAL_ENCLAVE_REGISTRY};
use bc_relayer_registry::{RelayerRegistryUpdater, GLOBAL_RELAYER_REGISTRY};
use bc_signer_registry::{SignerRegistryUpdater, GLOBAL_SIGNER_REGISTRY};
use core::str::from_utf8;
pub use ita_sgx_runtime::{Balance, Index};
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itc_parentchain_indirect_calls_executor::error::Error;
use itp_stf_primitives::{traits::IndirectExecutor, types::TrustedOperation};
use itp_types::{
	parentchain::{
		AccountId, FilterEvents, HandleParentchainEvents, ParentchainEventProcessingError,
		ParentchainId,
	},
	MrEnclave, WorkerType,
};
use lc_scheduled_enclave::{ScheduledEnclaveUpdater, GLOBAL_SCHEDULED_ENCLAVE};
use litentry_hex_utils::hex_encode;
use litentry_primitives::{Address32, Identity, SidechainBlockNumber};
use log::*;
use sp_core::{blake2_256, H256};
use sp_std::vec::Vec;
use std::string::ToString;

pub struct ParentchainEventHandler {}

impl ParentchainEventHandler {
	fn shield_funds<Executor: IndirectExecutor<TrustedCallSigned, Error>>(
		executor: &Executor,
		account: &AccountId,
		amount: Balance,
	) -> Result<(), Error> {
		log::info!("shielding for {:?} amount {}", account, amount,);
		let shard = executor.get_default_shard();
		// todo: ensure this parentchain is assigned for the shard vault!
		let trusted_call = TrustedCall::balance_shield(
			executor.get_enclave_account()?.into(),
			account.clone(),
			amount,
			ParentchainId::Litentry,
		);
		let signed_trusted_call = executor.sign_call_with_self(&trusted_call, &shard)?;
		let trusted_operation =
			TrustedOperation::<TrustedCallSigned, Getter>::indirect_call(signed_trusted_call);

		let encrypted_trusted_call = executor.encrypt(&trusted_operation.encode())?;
		executor.submit_trusted_call(shard, encrypted_trusted_call);

		Ok(())
	}

	fn set_scheduled_enclave(
		worker_type: WorkerType,
		sbn: SidechainBlockNumber,
		mrenclave: MrEnclave,
	) -> Result<(), Error> {
		if worker_type != WorkerType::BitAcross {
			warn!("Ignore SetScheduledEnclave due to wrong worker_type");
			return Ok(())
		}
		GLOBAL_SCHEDULED_ENCLAVE.update(sbn, mrenclave)?;

		Ok(())
	}

	fn remove_scheduled_enclave(
		worker_type: WorkerType,
		sbn: SidechainBlockNumber,
	) -> Result<(), Error> {
		if worker_type != WorkerType::BitAcross {
			warn!("Ignore RemoveScheduledEnclave due to wrong worker_type");
			return Ok(())
		}
		GLOBAL_SCHEDULED_ENCLAVE.remove(sbn)?;

		Ok(())
	}

	fn add_relayer(account: Identity) -> Result<(), Error> {
		info!("Adding Relayer Account to Registry: {:?}", account);
		GLOBAL_RELAYER_REGISTRY.update(account).map_err(|e| {
			error!("Error adding relayer: {:?}", e);
			Error::Other("Error adding relayer".into())
		})?;

		Ok(())
	}

	fn remove_relayer(account: Identity) -> Result<(), Error> {
		info!("Remove Relayer Account from Registry: {:?}", account);
		GLOBAL_RELAYER_REGISTRY.remove(account).map_err(|e| {
			error!("Error removing relayer: {:?}", e);
			Error::Other("Error removing relayer".into())
		})?;

		Ok(())
	}

	fn add_enclave(
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
		GLOBAL_ENCLAVE_REGISTRY.update(account_id, url).map_err(|e| {
			error!("Error adding enclave: {:?}", e);
			Error::Other("Error adding enclave".into())
		})?;

		Ok(())
	}

	fn remove_enclave(account_id: Address32) -> Result<(), Error> {
		info!("Remove Enclave Account from Registry: {:?}", account_id);
		GLOBAL_ENCLAVE_REGISTRY.remove(account_id).map_err(|e| {
			error!("Error removing enclave: {:?}", e);
			Error::Other("Error removing enclave".into())
		})?;

		Ok(())
	}

	fn save_signer(account_id: Address32, pub_key: [u8; 33]) -> Result<(), Error> {
		info!("Saving Signer Account to Registry: {:?}", account_id);
		GLOBAL_SIGNER_REGISTRY.update(account_id, pub_key).map_err(|e| {
			error!("Error saving signer: {:?}", e);
			Error::Other("Error saving signer".into())
		})?;

		Ok(())
	}
}

impl<Executor> HandleParentchainEvents<Executor, TrustedCallSigned, Error>
	for ParentchainEventHandler
where
	Executor: IndirectExecutor<TrustedCallSigned, Error>,
{
	fn handle_events(
		executor: &Executor,
		events: impl FilterEvents,
		vault_account: Option<AccountId>,
	) -> Result<Vec<H256>, Error> {
		let mut handled_events: Vec<H256> = Vec::new();

		if let Ok(events) = events.get_transfer_events() {
			debug!(
				"Handling transfer events to shard vault account: {}",
				hex_encode(vault_account.encode().as_slice())
			);
			events
				.iter()
				.filter(|&event| {
					if let Some(vault_account) = &vault_account {
						event.to == *vault_account
					} else {
						false
					}
				})
				.try_for_each(|event| {
					info!("found transfer_event to vault account: {}", event);
					//debug!("shielding from Integritee suppressed");
					let result = Self::shield_funds(executor, &event.from, event.amount);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::ShieldFundsFailure)?;
		}

		if let Ok(events) = events.get_scheduled_enclave_set_events() {
			debug!("Handling ScheduledEnclaveSet events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found ScheduledEnclaveSet event: {:?}", event);
					let result = Self::set_scheduled_enclave(
						event.worker_type,
						event.sidechain_block_number,
						event.mrenclave,
					);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::ScheduledEnclaveSetFailure)?;
		}

		if let Ok(events) = events.get_scheduled_enclave_removed_events() {
			debug!("Handling ScheduledEnclaveRemoved events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found ScheduledEnclaveRemoved event: {:?}", event);
					let result = Self::remove_scheduled_enclave(
						event.worker_type,
						event.sidechain_block_number,
					);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::ScheduledEnclaveRemovedFailure)?;
		}

		if let Ok(events) = events.get_relayer_added_events() {
			debug!("Handling RelayerAdded events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found RelayerAdded event: {:?}", event);
					let result = Self::add_relayer(event.who.clone());
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::RelayerAddFailure)?;
		}

		if let Ok(events) = events.get_relayers_removed_events() {
			debug!("Handling RelayerRemoved events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found RelayerRemoved event: {:?}", event);
					let result = Self::remove_relayer(event.who.clone());
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::RelayerRemoveFailure)?;
		}

		if let Ok(events) = events.get_enclave_added_events() {
			debug!("Handling EnclaveAdded events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found EnclaveAdded event: {:?}", event);
					let result = Self::add_enclave(event.who, event.url.clone(), event.worker_type);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::EnclaveAddFailure)?;
		}

		if let Ok(events) = events.get_enclave_removed_events() {
			debug!("Handling EnclaveRemoved events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found EnclaveRemoved event: {:?}", event);
					let result = Self::remove_enclave(event.who);
					handled_events.push(hash_of(&event));

					result
				})
				.map_err(|_| ParentchainEventProcessingError::EnclaveRemoveFailure)?;
		}

		if let Ok(events) = events.get_btc_wallet_generated_events() {
			debug!("Handling BtcWalletGenerated events");
			events
				.iter()
				.try_for_each(|event| {
					debug!("found BtcWalletGenerated event: {:?}", event);
					let result = Self::save_signer(event.account_id.clone().into(), event.pub_key);
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
